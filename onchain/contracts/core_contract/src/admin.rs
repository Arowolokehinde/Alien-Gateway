use soroban_sdk::{panic_with_error, Address, BytesN, Env};

use crate::errors::CoreError;
use crate::events::INIT_EVENT;
use crate::{smt_root, storage};

pub struct Admin;

impl Admin {
    pub fn initialize(env: Env, owner: Address) {
        owner.require_auth();
        if storage::is_initialized(&env) {
            panic_with_error!(&env, CoreError::AlreadyInitialized);
        }
        storage::set_owner(&env, &owner);
        #[allow(deprecated)]
        env.events().publish((INIT_EVENT,), (owner,));
    }

    pub fn get_contract_owner(env: Env) -> Address {
        storage::get_owner(&env).unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound))
    }

    pub fn get_smt_root(env: Env) -> BytesN<32> {
        smt_root::SmtRoot::get_root(env.clone())
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::RootNotSet))
    }

    pub fn update_smt_root(env: Env, new_root: BytesN<32>) {
        let owner = storage::get_owner(&env)
            .unwrap_or_else(|| panic_with_error!(&env, CoreError::NotFound));
        owner.require_auth();

        if let Some(current) = env
            .storage()
            .instance()
            .get::<_, soroban_sdk::BytesN<32>>(&storage::DataKey::SmtRoot)
        {
            if current == new_root {
                panic_with_error!(&env, CoreError::RootUnchanged);
            }
        }

        smt_root::SmtRoot::update_root(&env, new_root);
    }
}
