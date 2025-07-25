use anyhow::Result;
use example_project::ExampleNode;
use iroh_n0des::simulation::{Context, Simulation, SimulationBuilder};
use rand::seq::IteratorRandom;

const EVENT_ID: &str = "ping";

#[iroh_n0des::sim]
async fn test_simulation() -> Result<SimulationBuilder<ExampleNode>> {
    async fn tick(ctx: &Context, node: &mut ExampleNode) -> Result<bool> {
        let me = ctx.self_addr().node_id;

        if let Some(last_target) = node.last_target.take() {
            // end previous event
            iroh_n0des::simulation::events::event_end(
                node.endpoint().node_id().fmt_short(),
                last_target.fmt_short(),
                format!("ping done (round {})", ctx.round - 1),
                EVENT_ID.to_string(),
            );
        }

        let target = ctx.all_other_nodes(me).choose(&mut rand::rng()).unwrap();
        node.ping(target.clone()).await?;

        // record event for simulation visualization.
        iroh_n0des::simulation::events::event_start(
            me.fmt_short(),
            target.node_id.fmt_short(),
            format!("send ping (round {})", ctx.round),
            Some(EVENT_ID.to_string()),
        );
        node.last_target = Some(target.node_id);
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
