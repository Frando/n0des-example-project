use std::time::Duration;

use anyhow::{Context, Result, ensure};
use bytes::Bytes;
use example_project::ExampleNode;
use iroh::{Endpoint, NodeId};
use iroh_gossip::{api::Event, proto::TopicId};
use iroh_n0des::simulation::{Builder, Node, RoundContext, Spawn};
use n0_future::StreamExt;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, derive_more::Deref, derive_more::DerefMut)]
struct SimNode {
    #[deref]
    #[deref_mut]
    node: ExampleNode,
    last_messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Setup {
    topic_id: TopicId,
}

impl Spawn<Setup> for SimNode {
    async fn spawn(context: &mut iroh_n0des::simulation::SpawnContext<'_, Setup>) -> Result<Self> {
        let endpoint = context.bind_endpoint().await?;
        let node = ExampleNode::spawn(endpoint, context.metrics_registry());
        Ok(SimNode {
            node,
            last_messages: vec![],
        })
    }
}

impl Node for SimNode {
    fn endpoint(&self) -> Option<&Endpoint> {
        Some(self.node.endpoint())
    }

    async fn shutdown(&mut self) -> anyhow::Result<()> {
        self.node.router.shutdown().await?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    sender: NodeId,
    round: u32,
}

impl Message {
    fn to_bytes(&self) -> Bytes {
        Bytes::from(postcard::to_stdvec(&self).unwrap())
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self> {
        Ok(postcard::from_bytes(bytes)?)
    }
}

#[iroh_n0des::sim]
async fn test_gossip() -> Result<Builder<Setup>> {
    async fn round(node: &mut SimNode, ctx: &RoundContext<'_, Setup>) -> Result<bool> {
        let topic_id = ctx.setup_data().topic_id;
        // In round 0, all nodes join the gossip topic.
        if ctx.round() == 0 {
            let topic = if ctx.node_index() == 0 {
                node.gossip.subscribe_and_join(topic_id, vec![]).await?
            } else {
                let remote_addr = ctx.addr(0)?;
                let remote_id = remote_addr.node_id;
                node.node.endpoint().add_node_addr(remote_addr)?;
                node.gossip
                    .subscribe_and_join(topic_id, vec![remote_id])
                    .await?
            };
            let (sender, receiver) = topic.split();
            node.gossip_sender = Some(sender);
            node.gossip_receiver = Some(receiver);
            // give the swarm some time to stabilize
            tokio::time::sleep(Duration::from_secs(1)).await;
        // In all other rounds, each node broadcasts a single message, and waits until it received all other messages.
        } else {
            let sender = node
                .gossip_sender
                .as_ref()
                .context("expected gossip sender")?;
            let message = Message {
                sender: node.node_id(),
                round: ctx.round(),
            };
            // We add a random delay before broadcasting
            let delay = rand::random_range(10..100);
            tokio::time::sleep(Duration::from_millis(delay)).await;
            sender.broadcast(message.to_bytes()).await?;

            // Now wait for all messages to arrive.
            let receiver = node
                .gossip_receiver
                .as_mut()
                .context("expected gossip receiver")?;
            let mut messages = vec![];
            let expected_count = ctx.node_count() - 1;
            while let Some(event) = receiver.try_next().await? {
                match event {
                    Event::NeighborUp(_) => {}
                    Event::NeighborDown(_) => {}
                    Event::Received(message) => {
                        let message = Message::from_bytes(&message.content)?;
                        info!("received message {} of {}", messages.len(), expected_count);
                        messages.push(message);
                        if messages.len() == expected_count {
                            break;
                        }
                    }
                    Event::Lagged => {}
                }
            }
            node.last_messages = messages;
        }
        Ok(true)
    }

    fn check(node: &SimNode, ctx: &RoundContext<'_, Setup>) -> Result<()> {
        if ctx.round() == 0 {
            return Ok(());
        }
        ensure!(node.last_messages.len() == ctx.node_count() - 1);
        ensure!(
            node.last_messages
                .iter()
                .all(|msg| msg.round == ctx.round())
        );
        let mut received_node_ids: Vec<_> =
            node.last_messages.iter().map(|msg| msg.sender).collect();
        let mut expected_node_ids: Vec<_> = ctx
            .all_other_nodes(node.node_id())
            .map(|addr| addr.node_id)
            .collect();
        received_node_ids.sort();
        expected_node_ids.sort();
        ensure!(received_node_ids == expected_node_ids);
        Ok(())
    }

    let builder = Builder::with_setup(async || {
        let setup = Setup {
            topic_id: TopicId::from_bytes([3u8; 32]),
        };
        Ok(setup)
    })
    .spawn(8, SimNode::builder(round).check(check))
    .rounds(20);

    Ok(builder)
}
