use anyhow::Result;
use example_project::ExampleNode;
use iroh_n0des::simulation::{Context, Simulation, SimulationBuilder};
use rand::seq::IteratorRandom;

#[iroh_n0des::sim]
async fn test_simulation() -> Result<SimulationBuilder<ExampleNode>> {
    async fn tick(ctx: &Context, node: &mut ExampleNode) -> Result<bool> {
        let me = ctx.self_addr().node_id;
        let target = ctx.all_other_nodes(me).choose(&mut rand::rng()).unwrap();
        node.ping(target.clone()).await?;
        // record event for simulation visualization.
        iroh_n0des::simulation::events::event(
            node.endpoint().node_id().fmt_short(),
            target.node_id.fmt_short(),
            format!("send ping (round {})", ctx.round),
        );
        Ok(true)
    }

    fn check(ctx: &Context, node: &ExampleNode) -> Result<()> {
        let metrics = node.ping.metrics();
        assert_eq!(metrics.pings_sent.get(), ctx.round + 1);
        Ok(())
    }

    let sim = Simulation::builder(tick)
        .check(check)
        .max_rounds(4)
        .node_count(8);
    Ok(sim)
}
