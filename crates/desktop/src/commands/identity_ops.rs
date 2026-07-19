use common::NodeId;
use crypto::KeyPair;

pub struct IdentityOps {
    keypair: KeyPair,
}

impl IdentityOps {
    pub fn new(keypair: KeyPair) -> Self {
        Self { keypair }
    }

    pub fn generate() -> Self {
        Self::new(KeyPair::generate())
    }

    pub fn get_node_id(&self) -> NodeId {
        self.keypair.node_id()
    }

    pub fn get_public_key(&self) -> Vec<u8> {
        self.keypair.public_key().to_bytes().to_vec()
    }

    pub fn sign_challenge(&self, data: &[u8]) -> Vec<u8> {
        self.keypair.sign(data)
    }

    pub fn verify_signature(&self, data: &[u8], signature: &[u8]) -> bool {
        self.keypair.verify(data, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_node_id() {
        let ops = IdentityOps::generate();
        let node_id = ops.get_node_id();
        assert!(!node_id.0.is_empty());
    }

    #[test]
    fn test_get_public_key() {
        let ops = IdentityOps::generate();
        let pk = ops.get_public_key();
        assert_eq!(pk.len(), 32);
    }

    #[test]
    fn test_sign_and_verify_challenge() {
        let ops = IdentityOps::generate();
        let challenge = b"peerbox-auth-challenge";
        let sig = ops.sign_challenge(challenge);
        assert_eq!(sig.len(), 64);
        assert!(ops.verify_signature(challenge, &sig));
    }

    #[test]
    fn test_verify_fails_with_wrong_data() {
        let ops = IdentityOps::generate();
        let sig = ops.sign_challenge(b"challenge");
        assert!(!ops.verify_signature(b"wrong", &sig));
    }

    #[test]
    fn test_identity_ops_deterministic_with_seed() {
        let seed = [42u8; 32];
        let kp1 = KeyPair::from_seed(&seed);
        let kp2 = KeyPair::from_seed(&seed);
        let ops1 = IdentityOps::new(kp1);
        let ops2 = IdentityOps::new(kp2);

        assert_eq!(ops1.get_node_id(), ops2.get_node_id());
        assert_eq!(ops1.get_public_key(), ops2.get_public_key());

        let sig1 = ops1.sign_challenge(b"test");
        let sig2 = ops2.sign_challenge(b"test");
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_different_keys_produce_different_ids() {
        let ops1 = IdentityOps::generate();
        let ops2 = IdentityOps::generate();
        assert_ne!(ops1.get_node_id(), ops2.get_node_id());
    }
}
