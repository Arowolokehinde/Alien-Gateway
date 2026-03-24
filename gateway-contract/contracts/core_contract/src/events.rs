use soroban_sdk::{contractevent, BytesN};

#[contractevent]
pub struct UsernameRegistered {
    pub commitment: BytesN<32>,
}

#[contractevent]
pub struct MerkleRootUpdated {
    pub old_root: BytesN<32>,
    pub new_root: BytesN<32>,
}
