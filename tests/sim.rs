use anyhow::Result;
use example_project::ExampleNode;
use iroh_n0des::simulation::{Context, Simulation, SimulationBuilder};

#[iroh_n0des::sim]
async fn test_simulation() -> Result<SimulationBuilder<ExampleNode>> {
    async fn tick(ctx: &Context, node: &mut ExampleNode) -> Result<bool> {
        if ctx.node_index != 0 {
            let target = ctx.addr(0).unwrap().clone();
            // record event for simulation visualization.
            iroh_n0des::simulation::events::event(
                node.endpoint().node_id().fmt_short(),
                target.node_id.fmt_short(),
                format!("send ping (round {})", ctx.round),
            );
            node.ping(target).await?;
        }
        Ok(true)
    }

    fn check(ctx: &Context, node: &ExampleNode) -> Result<()> {
        let metrics = node.ping.metrics();
        let node_count = ctx.addrs.len() as u64;
        println!(
            "round {} node {}: sent {} recv {}",
            ctx.round,
            ctx.node_index,
            metrics.pings_sent.get(),
            metrics.pings_recv.get()
        );
        match ctx.node_index {
            0 => assert_eq!(metrics.pings_recv.get(), (node_count - 1) * (ctx.round + 1)),
            _ => assert_eq!(metrics.pings_sent.get(), ctx.round + 1),
        }
        Ok(())
    }

    let sim = Simulation::builder(tick)
        .check(check)
        .max_rounds(4)
        .node_count(8);
    Ok(sim)
}
