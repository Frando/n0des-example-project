use anyhow::Result;
use iroh::{Endpoint, NodeAddr, NodeId, protocol::Router};
use iroh_n0des::{N0de, Registry};
use iroh_ping::{ALPN as PingALPN, Ping};

pub struct ExampleNode {
    pub ping: Ping,
    pub router: Router,
    pub last_target: Option<NodeId>,
}

// needing to have this exported type is very irritating.
// pub type N0de = Node;

impl ExampleNode {
    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }

    pub async fn ping(&self, addr: impl Into<NodeAddr>) -> anyhow::Result<std::time::Duration> {
        self.ping.ping(self.endpoint(), addr.into()).await
    }
}

impl N0de for ExampleNode {
    async fn spawn(ep: Endpoint, metrics: &mut Registry) -> Result<Self> {
        let ping = Ping::new();
        metrics.register(ping.metrics().clone());

        let router = iroh::protocol::Router::builder(ep)
            .accept(PingALPN, ping.clone())
            .spawn();

        Ok(Self {
            ping,
            router,
            last_target: None,
        })
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     #[iroh_n0des::sim]
//     async fn test_simulation() -> Result<SimulationBuilder> {
//         let tick = |ctx: &Context, node: &ExampleN0de| -> BoxFuture<Result<bool>> {
//             let me = node.router.endpoint().node_id();
//             let other_nodes = ctx.all_other_nodes(me);
//             let ping = node.ping.clone();
//             let endpoint = node.router.endpoint().clone();
//             let node_index = ctx.node_index;

//             Box::pin(async move {
//                 if node_index % 2 == 0 {
//                     for other in other_nodes.iter() {
//                         println!("Sending message:\n\tfrom: {me}\n\t to:   {}", other.node_id);
//                         ping.ping(&endpoint, (other.clone()).into()).await?;
//                     }
//                 }
//                 Ok(true)
//             })
//         };

//         let check = |ctx: &Context, node: &ExampleN0de| -> Result<()> {
//             let metrics = node.ping.metrics();
//             let node_count = ctx.addrs.len() as u64;
//             match ctx.node_index % 2 {
//                 0 => assert_eq!(metrics.pings_sent.get(), node_count / 2),
//                 _ => assert_eq!(metrics.pings_recv.get(), node_count / 2),
//             }
//             Ok(())
//         };

//         let sim: Simulation<ExampleN0de> = Simulation::builder(tick)
//             .max_ticks(1) // currently stuck on one tick, because more ends with an "endpoint closing" error
//             .check(check);
//         Ok(sim)
//     }

//     #[iroh_n0des::sim]
//     async fn unimpl_simulation() -> Result<SimulationBuilder> {
//         todo!()
//     }
// }
