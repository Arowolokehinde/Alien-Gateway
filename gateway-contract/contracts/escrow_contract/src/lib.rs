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
use crate::storage::{
    increment_payment_id, read_auto_pay, read_vault, write_auto_pay, write_scheduled_payment,
    write_vault,
};
use crate::types::{AutoPay, ScheduledPayment};
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

    /// Stores recurring payment settings for a source vault.
    pub fn setup_auto_pay(
        env: Env,
        from: BytesN<32>,
        to: BytesN<32>,
        amount: i128,
        interval: u64,
    ) -> Result<(), EscrowError> {
        if amount <= 0 || interval == 0 {
            return Err(EscrowError::InvalidAmount);
        }

        let vault = read_vault(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        vault.owner.require_auth();

        let auto_pay = AutoPay {
            to,
            amount,
            interval,
            last_paid: 0,
        };
        write_auto_pay(&env, &from, &auto_pay);
        Ok(())
    }

    /// Triggers one recurring payment cycle if interval has elapsed.
    pub fn trigger_auto_pay(env: Env, from: BytesN<32>) -> Result<(), EscrowError> {
        let mut auto_pay = read_auto_pay(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        let mut from_vault = read_vault(&env, &from).ok_or(EscrowError::VaultNotFound)?;
        let mut to_vault = read_vault(&env, &auto_pay.to).ok_or(EscrowError::VaultNotFound)?;

        let now = env.ledger().timestamp();
        let next_allowed = if auto_pay.last_paid == 0 {
            auto_pay.interval
        } else {
            auto_pay.last_paid.saturating_add(auto_pay.interval)
        };
        if now < next_allowed {
            return Err(EscrowError::PastReleaseTime);
        }

        if from_vault.balance < auto_pay.amount {
            return Err(EscrowError::InsufficientBalance);
        }

        from_vault.balance -= auto_pay.amount;
        to_vault.balance += auto_pay.amount;
        auto_pay.last_paid = now;

        write_vault(&env, &from, &from_vault);
        write_vault(&env, &auto_pay.to, &to_vault);
        write_auto_pay(&env, &from, &auto_pay);
        Ok(())
    }
}
