use std::collections::HashMap;

use pingora::{proxy::Session, upstreams::peer::HttpPeer};

pub type GeneratePeerKey = dyn Fn(&Session) -> String;

pub struct UpstreamPeerConnector {
    generate_peer_key: Box<GeneratePeerKey>,
    peers: HashMap<String, Box<HttpPeer>>,
}

unsafe impl Send for UpstreamPeerConnector {}
unsafe impl Sync for UpstreamPeerConnector {}

impl UpstreamPeerConnector {
    pub fn new(generate_peer_key: Box<GeneratePeerKey>) -> Self {
        Self {
            generate_peer_key,
            peers: HashMap::new(),
        }
    }

    pub fn register_peer(&mut self, key: String, peer: Box<HttpPeer>) {
        self.peers.insert(key, peer);
    }

    pub fn get_peer(&self, session: &Session) -> Option<Box<HttpPeer>> {
        let key = (self.generate_peer_key)(session);
        self.peers.get(&key).cloned()
    }
}
