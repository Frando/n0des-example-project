use iroh::{Endpoint, NodeId, protocol::Router};
use iroh_blobs::api::downloader::Downloader;
use iroh_gossip::{
    api::{GossipReceiver, GossipSender},
    net::Gossip,
};
use iroh_n0des::Registry;
use iroh_ping::Ping;

#[derive(Debug)]
pub struct ExampleNode {
    pub router: Router,
    pub ping: Ping,
    pub gossip: Gossip,
    pub blobs: iroh_blobs::api::Store,
    pub downloader: Downloader,
    pub gossip_sender: Option<GossipSender>,
    pub gossip_receiver: Option<GossipReceiver>,
}

impl ExampleNode {
    pub fn spawn(endpoint: Endpoint, registry: &mut Registry) -> Self {
        let ping = Ping::default();
        let blobs = iroh_blobs::store::mem::MemStore::default();
        let gossip = Gossip::builder().spawn(endpoint.clone());
        let blobs_protocol = iroh_blobs::BlobsProtocol::new(&blobs, endpoint.clone(), None);
        let downloader = Downloader::new(&blobs, &endpoint);

        registry.register(ping.metrics().clone());
        registry.register(gossip.metrics().clone());

        let router = Router::builder(endpoint)
            .accept(iroh_ping::ALPN, ping.clone())
            .accept(iroh_gossip::ALPN, gossip.clone())
            .accept(iroh_blobs::ALPN, blobs_protocol)
            .spawn();

        Self {
            router,
            ping,
            gossip,
            blobs: blobs.as_ref().clone(),
            downloader,
            gossip_receiver: None,
            gossip_sender: None,
        }
    }

    pub fn endpoint(&self) -> &Endpoint {
        self.router.endpoint()
    }

    pub fn node_id(&self) -> NodeId {
        self.endpoint().node_id()
    }
}
