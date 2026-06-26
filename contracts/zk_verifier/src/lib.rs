#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracterror, panic_with_error, symbol_short, Bytes, BytesN, Env,
};

pub const MAX_PROOF_SIZE: u32 = 4096;
pub const MAX_CLAIM_SIZE: u32 = 1024;

const VERIFY_CLAIM_TOPIC: soroban_sdk::Symbol = symbol_short!("vfy_claim");

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum VerifierError {
    /// Proof bytes were empty.
    EmptyProof = 1,
    /// Claim bytes were empty.
    EmptyClaim = 2,
    /// Proof bytes exceed MAX_PROOF_SIZE.
    ProofTooLarge = 3,
    /// Claim bytes exceed MAX_CLAIM_SIZE.
    ClaimTooLarge = 4,
}

#[contract]
pub struct ZkVerifierContract;

#[contractimpl]
impl ZkVerifierContract {
    /// Verifies a zero-knowledge proof against a claim.
    ///
    /// # STUB
    /// Real ZK proof verification (e.g. Groth16, PLONK) requires a verifier
    /// circuit and cryptographic primitives not yet available as Soroban host
    /// functions. This implementation is a non-empty bytes guard that acts as
    /// a placeholder until a native ZK host function is exposed.
    ///
    /// Returns `true` when both `proof` and `claim` are non-empty and `proof`
    /// is not the known-invalid 0x00 sentinel.
    ///
    /// Emits a `vfy_claim` event with `(result, claim_hash)` on every call
    /// that passes input validation.
    pub fn verify_claim(env: Env, proof: Bytes, claim: Bytes) -> bool {
        if proof.is_empty() {
            panic_with_error!(&env, VerifierError::EmptyProof);
        }
        if proof.len() > MAX_PROOF_SIZE {
            panic_with_error!(&env, VerifierError::ProofTooLarge);
        }
        if claim.is_empty() {
            panic_with_error!(&env, VerifierError::EmptyClaim);
        }
        if claim.len() > MAX_CLAIM_SIZE {
            panic_with_error!(&env, VerifierError::ClaimTooLarge);
        }

        // STUB: a single 0x00 byte is treated as a known-invalid proof sentinel.
        // Real ZK verification would replace this with cryptographic validation.
        let result = !(proof.len() == 1 && proof.get(0) == Some(0x00));

        let claim_hash: BytesN<32> = env.crypto().sha256(&claim);
        env.events().publish((VERIFY_CLAIM_TOPIC,), (result, claim_hash));

        result
    }
}

#[cfg(test)]
mod test;
