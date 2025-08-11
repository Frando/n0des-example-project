use anyhow::Result;
use iroh_n0des::simulation::{Builder, RoundContext, Spawn};

#[derive(Debug, Clone)]
struct Node;
impl iroh_n0des::simulation::Node for Node {}

impl Spawn for Node {
    async fn spawn(
        _ctx: &mut iroh_n0des::simulation::SpawnContext<'_, ()>,
    ) -> anyhow::Result<Self> {
        Ok(Node)
    }
}

#[iroh_n0des::sim]
async fn test_ping() -> Result<Builder> {
    async fn round(_node: &mut Node, _ctx: &RoundContext<'_>) -> Result<bool> {
        Ok(true)
    }

    fn check(_node: &Node, _ctx: &RoundContext<'_>) -> Result<()> {
        Ok(())
    }

    let sim = Builder::new()
        .spawn(2, Node::builder(round).check(check))
        .rounds(2);
    Ok(sim)
}
