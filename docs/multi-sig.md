# Multi-Sig Vault Configuration

Vault owners can require multiple approvals before sensitive operations execute. This protects against single-key compromise and is suitable for shared custody or high-value vaults.

## How It Works

Multi-sig uses a **propose → approve → execute** flow:

```
Owner                    Co-Signers
  │
  ├─ configure_multisig(signers, threshold)
  │
  ├─ propose_multisig(operation, payload)  ← owner auto-approves
  │
  │                    ├─ approve_multisig(proposal_id)
  │                    └─ approve_multisig(proposal_id)  ← threshold reached
  │
  └─ execute_multisig(proposal_id)         ← operation runs
```

## Setup

```rust
// 2-of-3 multi-sig (owner + 2 co-signers, threshold = 2)
configure_multisig(vault_id, owner, [signer1, signer2], 2)
```

- `signers` — co-signer addresses (must not include the owner)
- `threshold` — total approvals needed (1 ≤ threshold ≤ signers.len() + 1)
- The owner always counts as one approver

## Operations Requiring Multi-Sig

Once configured, these operations require a proposal:

| Operation | Payload |
|---|---|
| `Withdraw` | `encode_i128_payload(amount)` |
| `UpdateBeneficiary` | `address_payload = Some(new_beneficiary)` |
| `CancelVault` | empty `Bytes` |
| `TransferOwnership` | `address_payload = Some(new_owner)` |
| `UpdateCheckInInterval` | `encode_u64_payload(new_interval)` |

## API Reference

### Configure
```rust
configure_multisig(vault_id, caller, signers, threshold) -> Result<(), ContractError>
remove_multisig(vault_id, caller) -> Result<(), ContractError>
get_multisig_config(vault_id) -> Option<MultiSigConfig>
has_multisig(vault_id) -> bool
```

### Propose
```rust
propose_multisig(vault_id, caller, operation, payload, address_payload) -> Result<u64, ContractError>
// Returns proposal_id. Owner is auto-approved.
```

### Approve / Reject
```rust
approve_multisig(vault_id, proposal_id, caller) -> Result<(), ContractError>
reject_multisig(vault_id, proposal_id, caller) -> Result<(), ContractError>
```

### Execute
```rust
execute_multisig(vault_id, proposal_id, caller) -> Result<(), ContractError>
// Proposal must be in Approved status.
```

### Query
```rust
get_multisig_proposal(vault_id, proposal_id) -> Option<MultiSigProposal>
get_multisig_proposal_count(vault_id) -> u64
```

### Payload Helpers
```rust
encode_i128_payload(value: i128) -> Bytes   // for Withdraw
encode_u64_payload(value: u64) -> Bytes     // for UpdateCheckInInterval
// For address operations, pass address_payload = Some(address)
```

## Proposal Lifecycle

| Status | Meaning |
|---|---|
| `Pending` | Created, collecting approvals |
| `Approved` | Threshold reached, ready to execute |
| `Executed` | Operation completed |
| `Rejected` | Owner rejected the proposal |
| `Expired` | Not executed within 7 days |

Proposals expire **7 days** after creation. Expired proposals cannot be approved or executed.

## Events

| Event | Topic | Data |
|---|---|---|
| Configured | `ms_cfg` | `threshold` |
| Proposed | `ms_prop` | `(proposal_id, operation, expires_at)` |
| Approved | `ms_appr` | `(proposal_id, approver, approval_count)` |
| Executed | `ms_exec` | `proposal_id` |
| Rejected | `ms_rej` | `proposal_id` |

## Error Codes

| Code | Constant | Meaning |
|---|---|---|
| `#34` | `MultiSigRequired` | Vault has no multi-sig config |
| `#35` | `AlreadyApproved` | Caller already approved this proposal |
| `#36` | `ProposalNotFound` | Proposal does not exist or is not Pending |
| `#37` | `ProposalExpired` | Proposal passed its 7-day expiry |
| `#38` | `ProposalNotApproved` | Proposal has not reached threshold yet |
| `#39` | `NotASigner` | Caller is not the owner or a configured co-signer |
| `#40` | `InvalidThreshold` | Threshold is 0 or exceeds total signers |

## Example: 2-of-3 Withdraw

```rust
// 1. Configure
configure_multisig(vault_id, owner, [alice, bob], 2);

// 2. Owner proposes a 500-stroop withdrawal
let payload = encode_i128_payload(500);
let pid = propose_multisig(vault_id, owner, Withdraw, payload, None);
// → owner auto-approved (1/2)

// 3. Alice approves → threshold reached (2/2)
approve_multisig(vault_id, pid, alice);

// 4. Owner executes
execute_multisig(vault_id, pid, owner);
// → 500 stroops transferred to owner
```
