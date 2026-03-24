//! The Escrow contract handles scheduled payments between vaults.
//! This implementation focuses on security, identity commitment, and host-level authentication.

#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use crate::errors::EscrowError;
use crate::events::Events;
use crate::storage::{increment_payment_id, read_vault, write_scheduled_payment, write_vault};
use crate::types::ScheduledPayment;
use soroban_sdk::{contract, contractimpl, BytesN, Env};

#[contract]
pub struct EscrowContract;

#[contractimpl]
impl EscrowContract {
    /// Schedules a payment from one vault to another.
    ///
    /// Funds are reserved in the source vault immediately upon scheduling.
    /// The payment can be executed at or after the `release_at` timestamp.
    ///
    /// ### Arguments
    /// - `from`: The commitment ID of the source vault.
    /// - `to`: The commitment ID of the destination vault.
    /// - `amount`: The amount of tokens to schedule. Must be > 0.
    /// - `release_at`: The ledger timestamp (u64) for release. Must be > current time.
    ///
    /// ### Returns
    /// - `u32`: The unique payment ID assigned to this schedule.
    ///
    /// ### Errors
    /// - `VaultNotFound`: If the `from` vault does not exist.
    /// - `InvalidAmount`: If `amount <= 0`.
    /// - `InsufficientBalance`: If the vault has less than `amount`.
    /// - `PastReleaseTime`: If `release_at` is not in the future.
    /// - `PaymentCounterOverflow`: If the global ID counter overflows.
    pub fn schedule_payment(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        release_at: u64,
    ) -> Result<u32, EscrowError> {
        // 1. Validate Input
        if amount <= 0 {
            return Err(EscrowError::InvalidAmount);
        }

        if release_at <= env.ledger().timestamp() {
            return Err(EscrowError::PastReleaseTime);
        }

        // 2. Read Vault
        let mut vault = read_vault(&env, &from).ok_or(EscrowError::VaultNotFound)?;

        // 3. Authenticate caller as owner of from vault
        // Host-level authentication. Panics with host error if unauthorized.
        vault.owner.require_auth();

        // 4. Validate Balance
        if vault.balance < amount {
            return Err(EscrowError::InsufficientBalance);
        }

        // 5. Reserve Funds
        vault.balance -= amount;
        write_vault(&env, &from, &vault);

        // 6. Generate Payment ID
        let payment_id = increment_payment_id(&env)?;

        // 7. Store Scheduled Payment
        let payment = ScheduledPayment {
            from,
            to,
            token: vault.token.clone(),
            amount,
            release_at,
            executed: false,
        };
        write_scheduled_payment(&env, payment_id, &payment);

        // 8. Emit Event
        Events::schedule_pay(
            &env,
            payment_id,
            payment.from,
            payment.to,
            payment.amount,
            payment.release_at,
        );

        Ok(payment_id)
    }
}
