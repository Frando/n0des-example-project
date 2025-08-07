use anyhow::{Result, ensure};
use example_project::ExampleNode;
use iroh_n0des::simulation::{Builder, RoundContext, Spawn};
use rand::seq::IteratorRandom;

const EVENT_ID: &str = "ping";

#[iroh_n0des::sim]
async fn test_ping() -> Result<Builder> {
    async fn round(node: &mut ExampleNode, ctx: &RoundContext<'_>) -> Result<bool> {
        let me = node.node_id();

        let target = ctx.all_other_nodes(me).choose(&mut rand::rng()).unwrap();
        node.ping.ping(node.endpoint(), target.clone()).await?;

        // record event for simulation visualization.
        iroh_n0des::simulation::events::event_start(
            me.fmt_short(),
            target.node_id.fmt_short(),
            format!("send ping (round {})", ctx.round()),
            Some(EVENT_ID.to_string()),
        );
        Ok(true)
    }

    fn check(node: &ExampleNode, ctx: &RoundContext<'_>) -> Result<()> {
        let metrics = node.ping.metrics();
        ensure!(metrics.pings_sent.get() == ctx.round() as u64 + 1);
        Ok(())
    }

    let sim = Builder::new()
        .spawn(4, ExampleNode::builder(round).check(check))
        .rounds(8);
    Ok(sim)
}
