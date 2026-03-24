#![no_std]

mod errors;
mod events;
mod storage;
mod types;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, Address, BytesN, Env};

pub use crate::errors::ContractError;
pub use crate::events::{MerkleRootUpdated, UsernameRegistered};
pub use crate::types::{Proof, PublicSignals};

#[contract]
pub struct CoreContract;

#[contractclient(name = "VerifierContractClient")]
pub trait VerifierContract {
    fn verify_proof(env: Env, proof: Proof, public_signals: PublicSignals) -> bool;
}

fn current_merkle_root(env: &Env) -> BytesN<32> {
    storage::get_merkle_root(env)
        .unwrap_or_else(|| panic_with_error!(env, ContractError::NotInitialized))
}

fn update_merkle_root(env: &Env, old_root: BytesN<32>, new_root: BytesN<32>) {
    storage::set_merkle_root(env, &new_root);

    MerkleRootUpdated { old_root, new_root }.publish(env);
}

#[contractimpl]
impl CoreContract {
    pub fn init(env: Env, verifier: Address, root: BytesN<32>) {
        if storage::is_initialized(&env) {
            panic_with_error!(&env, ContractError::AlreadyInitialized);
        }

        storage::set_verifier(&env, &verifier);
        storage::set_merkle_root(&env, &root);
    }

    pub fn submit_proof(env: Env, proof: Proof, public_signals: PublicSignals) {
        let current_root = current_merkle_root(&env);

        if current_root != public_signals.old_root.clone() {
            panic_with_error!(&env, ContractError::RootMismatch);
        }

        if storage::has_commitment(&env, &public_signals.commitment) {
            panic_with_error!(&env, ContractError::DuplicateCommitment);
        }

        let verifier = storage::get_verifier(&env)
            .unwrap_or_else(|| panic_with_error!(&env, ContractError::NotInitialized));
        let verifier_client = VerifierContractClient::new(&env, &verifier);
        let is_valid = verifier_client.verify_proof(&proof, &public_signals);
        if !is_valid {
            panic_with_error!(&env, ContractError::InvalidProof);
        }

        storage::store_commitment(&env, &public_signals.commitment);
        update_merkle_root(&env, current_root, public_signals.new_root.clone());

        UsernameRegistered {
            commitment: public_signals.commitment,
        }
        .publish(&env);
    }

    pub fn get_merkle_root(env: Env) -> BytesN<32> {
        current_merkle_root(&env)
    }

    pub fn get_verifier(env: Env) -> Option<Address> {
        storage::get_verifier(&env)
    }

    pub fn has_commitment(env: Env, commitment: BytesN<32>) -> bool {
        storage::has_commitment(&env, &commitment)
    }
}
