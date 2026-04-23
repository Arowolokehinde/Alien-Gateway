use soroban_sdk::{contractevent, Address, BytesN, Env};

use crate::types::{UsernameRecord, UsernameTransferredPayload};

#[contractevent(topics = ["USERNAME_DEPLOYED"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameDeployedEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    #[topic]
    pub owner: Address,
    pub record: UsernameRecord,
}

pub fn emit_username_deployed(env: &Env, record: &UsernameRecord) {
    UsernameDeployedEvent {
        username_hash: record.username_hash.clone(),
        owner: record.owner.clone(),
        record: record.clone(),
    }
    .publish(env);
}

#[contractevent(topics = ["USERNAME_TRANSFERRED"], data_format = "single-value")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UsernameTransferredEvent {
    #[topic]
    pub username_hash: BytesN<32>,
    pub payload: UsernameTransferredPayload,
}

pub fn emit_username_transferred(
    env: &Env,
    username_hash: BytesN<32>,
    old_owner: Address,
    new_owner: Address,
) {
    UsernameTransferredEvent {
        username_hash,
        payload: UsernameTransferredPayload {
            old_owner,
            new_owner,
        },
    }
    .publish(env);
}
