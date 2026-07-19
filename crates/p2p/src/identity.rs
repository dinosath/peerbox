use crate::types::PeerId;
use crypto::KeyPair;

pub struct NodeIdentity {
    pub peer_id: PeerId,
    pub keypair: KeyPair,
}

impl NodeIdentity {
    pub fn generate() -> Self {
        let keypair = KeyPair::generate();
        let peer_id = PeerId(keypair.node_id().0.clone());
        Self { peer_id, keypair }
    }

    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        self.keypair.sign(data)
    }

    pub fn verify(&self, data: &[u8], sig: &[u8]) -> bool {
        self.keypair.verify(data, sig)
    }
}
