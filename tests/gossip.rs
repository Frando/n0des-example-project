use anyhow::Result;
use iroh::{Endpoint, protocol::Router};
use iroh_gossip::{
    api::{Event, GossipTopic},
    net::Gossip,
    proto::TopicId,
};
use iroh_n0des::{
    N0de, Registry,
    simulation::{Context, Simulation, SimulationBuilder},
};
use n0_future::StreamExt;

struct GossipNode {
    gossip: Gossip,
    router: Router,
    topic: Option<GossipTopic>,
}

impl GossipNode {
    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }
}

impl N0de for GossipNode {
    async fn spawn(ep: Endpoint, metrics: &mut Registry) -> Result<Self> {
        let gossip = Gossip::builder().spawn(ep.clone());
        metrics.register(gossip.metrics().clone());

        let router = iroh::protocol::Router::builder(ep)
            .accept(iroh_gossip::ALPN, gossip.clone())
            .spawn();

        Ok(Self {
            gossip,
            router,
            topic: None,
        })
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.router.shutdown().await?;
        Ok(())
    }
}

const TOPIC: TopicId = TopicId::from_bytes([0u8; 32]);

impl GossipNode {
    async fn tick_bootstrap(&mut self, ctx: &Context) -> Result<bool> {
        let addrs = if ctx.node_index == 0 {
            vec![]
        } else {
            let addr = ctx.addr(0)?;
            self.endpoint().add_node_addr(addr.clone())?;
            vec![addr.node_id]
        };
        let topic = self.gossip.subscribe_and_join(TOPIC, addrs).await?;
        self.topic = Some(topic);
        Ok(true)
    }

    async fn tick_broadcast(&mut self, ctx: &Context) -> Result<bool> {
        let sender = ctx.round as usize % ctx.node_count();
        if ctx.node_index == sender {
            let me = self.endpoint().node_id().fmt_short();
            let topic = self.topic.as_mut().expect("topic is initialized");
            let message = format!("in round {} the sender is {me}", ctx.round,);
            topic.broadcast(message.as_bytes().to_vec().into()).await?;
            Ok(true)
        } else {
            let topic = self.topic.as_mut().expect("topic is initialized");
            while let Some(event) = topic.try_next().await? {
                match event {
                    Event::NeighborUp(_node_id) => {}
                    Event::NeighborDown(_node_id) => {}
                    Event::Received(message) => {
                        let message = String::from_utf8(message.content.to_vec())?;
                        tracing::info!("received: {message}");
                        return Ok(true);
                    }
                    Event::Lagged => return Err(anyhow::anyhow!("node lagged")),
                }
            }
            unreachable!()
        }
    }

    fn check(&self, _ctx: &Context) -> Result<()> {
        Ok(())
    }
}

#[iroh_n0des::sim]
async fn gossip_smoke() -> Result<SimulationBuilder<GossipNode>> {
    async fn tick(ctx: &Context, node: &mut GossipNode) -> Result<bool> {
        match ctx.round {
            0 => node.tick_bootstrap(ctx).await,
            _ => node.tick_broadcast(ctx).await,
        }
    }
    fn check(ctx: &Context, node: &GossipNode) -> Result<()> {
        node.check(ctx)
    }

    let sim = Simulation::builder(tick)
        .check(check)
        .max_rounds(4)
        .node_count(8);
    Ok(sim)
}
