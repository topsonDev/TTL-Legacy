#![cfg(test)]

use super::*;
use soroban_sdk::{bytes, testutils::Events as _, Bytes, Env};

fn setup() -> (Env, ZkVerifierContractClient<'static>) {
    let env = Env::default();
    let id = env.register_contract(None, ZkVerifierContract);
    let client = ZkVerifierContractClient::new(&env, &id);
    (env, client)
}

// ── Existing correctness tests ────────────────────────────────────────────────

/// Valid proof and claim — must return true.
#[test]
fn test_valid_proof_returns_true() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0xdeadbeef);
    let claim = bytes!(&env, 0xcafebabe);
    assert!(client.verify_claim(&proof, &claim));
}

/// Invalid proof (0x00 sentinel) — must return false.
#[test]
fn test_invalid_proof_returns_false() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0x00); // known-invalid sentinel
    let claim = bytes!(&env, 0xcafebabe);
    assert!(!client.verify_claim(&proof, &claim));
}

/// Malformed input: empty proof — must panic with EmptyProof (#1).
#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_malformed_empty_proof_panics() {
    let (env, client) = setup();
    let proof = bytes!(&env,);
    let claim = bytes!(&env, 0xcafebabe);
    client.verify_claim(&proof, &claim);
}

/// Malformed input: empty claim — must panic with EmptyClaim (#2).
#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_malformed_empty_claim_panics() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0xdeadbeef);
    let claim = bytes!(&env,);
    client.verify_claim(&proof, &claim);
}

// ── #817: Input size limit tests ──────────────────────────────────────────────

/// Proof at exactly MAX_PROOF_SIZE — must succeed.
#[test]
fn test_proof_at_max_size_succeeds() {
    let (env, client) = setup();
    let data = [0xffu8; MAX_PROOF_SIZE as usize];
    let proof = Bytes::from_slice(&env, &data);
    let claim = bytes!(&env, 0xcafebabe);
    assert!(client.verify_claim(&proof, &claim));
}

/// Proof one byte over MAX_PROOF_SIZE — must panic with ProofTooLarge (#3).
#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_proof_exceeds_max_size_panics() {
    let (env, client) = setup();
    let data = [0xffu8; MAX_PROOF_SIZE as usize + 1];
    let proof = Bytes::from_slice(&env, &data);
    let claim = bytes!(&env, 0xcafebabe);
    client.verify_claim(&proof, &claim);
}

/// Claim at exactly MAX_CLAIM_SIZE — must succeed.
#[test]
fn test_claim_at_max_size_succeeds() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0xdeadbeef);
    let data = [0xaau8; MAX_CLAIM_SIZE as usize];
    let claim = Bytes::from_slice(&env, &data);
    assert!(client.verify_claim(&proof, &claim));
}

/// Claim one byte over MAX_CLAIM_SIZE — must panic with ClaimTooLarge (#4).
#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_claim_exceeds_max_size_panics() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0xdeadbeef);
    let data = [0xaau8; MAX_CLAIM_SIZE as usize + 1];
    let claim = Bytes::from_slice(&env, &data);
    client.verify_claim(&proof, &claim);
}

// ── #818: Event emission tests ────────────────────────────────────────────────

/// verify_claim with a valid proof must emit exactly one vfy_claim event.
#[test]
fn test_verify_claim_emits_event_on_true_result() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0xdeadbeef);
    let claim = bytes!(&env, 0xcafebabe);
    let result = client.verify_claim(&proof, &claim);
    assert!(result);
    assert_eq!(env.events().all().len(), 1);
}

/// verify_claim with the 0x00 sentinel must emit exactly one vfy_claim event
/// even when the result is false.
#[test]
fn test_verify_claim_emits_event_on_false_result() {
    let (env, client) = setup();
    let proof = bytes!(&env, 0x00); // known-invalid sentinel → result = false
    let claim = bytes!(&env, 0xcafebabe);
    let result = client.verify_claim(&proof, &claim);
    assert!(!result);
    assert_eq!(env.events().all().len(), 1);
}
