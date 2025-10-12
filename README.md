# Team Wallet - Democratic Multi-Signature Wallet on Solana

A comprehensive on-chain governance solution for managing funds, tokens, and program upgrades through democratic voting on the Solana blockchain.

## 🌟 Features

### **Core Functionality**
- ✅ **Multi-Signature Wallet** - Democratic control over SOL and SPL tokens
- ✅ **Voting System** - Configurable threshold-based approval mechanism
- ✅ **Role Management** - Owner, Voters, and Contributors with different permissions
- ✅ **Proposal System** - Create, vote, and execute proposals on-chain

### **Fund Management**
- 💰 **SOL Transfers** - Direct native token transfers
- 🪙 **SPL Token Support** - Full support for Token and Token-2022
- 🔄 **Automatic Token Accounts** - Auto-creates ATAs on first proposal
- 🏦 **Multi-Token Support** - Hold unlimited token types

### **Token Management (Token-2022 Extensions)**
- 🔨 **Mint & Burn** - Democratic token supply control
- 🧊 **Freeze/Thaw** - Account freezing capabilities
- 💸 **Transfer Fees** - Configure and withdraw transfer fees
- 📊 **Interest Bearing** - Update interest rates
- 🔐 **Confidential Transfers** - Privacy features
- 🏷️ **Metadata Management** - Update token metadata via Metaplex
- 👥 **Token Groups** - Manage token collections
- 🎯 **Permanent Delegate** - Set permanent authorities

### **Program Governance**
- 🔧 **Program Upgrades** - Vote on program updates
- 🔑 **Authority Transfer** - Democratic program ownership
- 📝 **On-chain Proposals** - Transparent upgrade process

## 📋 Table of Contents

- [Architecture](#architecture)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Guide](#usage-guide)
- [API Reference](#api-reference)
- [Examples](#examples)
- [Testing](#testing)
- [Security](#security)
- [Contributing](#contributing)
- [License](#license)

## 🏗️ Architecture

### **Account Structure**

```
TeamWallet (PDA)
├── Owner (Pubkey)
├── Voters (Vec<Pubkey>)
├── Contributors (Vec<Pubkey>)
├── Vote Threshold (u8)
└── Bump (u8)

Proposal (PDA)
├── Team Wallet (Pubkey)
├── Proposer (Pubkey)
├── Amount (u64)
├── Recipient (Pubkey)
├── Votes For/Against (u8)
└── Executed (bool)

TokenProposal (PDA)
├── Team Wallet (Pubkey)
├── Action (TokenAction enum)
├── Mint (Pubkey)
├── Metadata (Optional)
├── Transfer Fee Config (Optional)
└── Votes For/Against (u8)

UpgradeProposal (PDA)
├── Team Wallet (Pubkey)
├── New Buffer (Pubkey)
└── Votes For/Against (u8)
```

### **Role Permissions**

| Feature | Owner | Voter | Contributor |
|---------|-------|-------|-------------|
| Create Proposals | ✅ | ✅ | ✅ |
| Vote on Proposals | ✅ | ✅ | ✅ |
| Add/Remove Members | ✅ | ❌ | ❌ |
| Execute Proposals | ✅ | ✅ | ✅ |

## 🚀 Installation

### Prerequisites

```bash
# Rust 1.83.0
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default 1.83.0

# Solana CLI 2.1.3
sh -c "$(curl -sSfL https://release.solana.com/v2.1.3/install)"

# Anchor 0.32.1
cargo install --git https://github.com/coral-xyz/anchor --tag v0.32.1 anchor-cli

# Node.js 18+ and Yarn
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g yarn
```

### Clone and Build

```bash
# Clone repository
git clone https://github.com/yourusername/teamwallet-contract
cd teamwallet-contract

# Install dependencies
yarn install

# Build program
anchor build

# Get program ID
solana address -k target/deploy/teamwallet-keypair.json

# Update program ID in lib.rs and Anchor.toml
# Then rebuild
anchor build
```

## ⚡ Quick Start

### 1. Initialize Team Wallet

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Teamwallet } from "../target/types/teamwallet";

const program = anchor.workspace.Teamwallet as Program<Teamwallet>;

// Define team members
const voters = [voter1.publicKey, voter2.publicKey, voter3.publicKey];

// Initialize team wallet
const [teamWalletPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("team_wallet"),
    owner.publicKey.toBuffer(),
    Buffer.from("Engineering Team"),
  ],
  program.programId
);

await program.methods
  .initializeTeamWallet(
    "Engineering Team",  // name
    60,                  // 60% vote threshold
    voters               // initial voters
  )
  .accounts({
    teamWallet: teamWalletPda,
    owner: owner.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([owner])
  .rpc();
```

### 2. Create a Proposal

```typescript
// Create SOL transfer proposal
const [proposalPda] = PublicKey.findProgramAddressSync(
  [
    Buffer.from("proposal"),
    teamWalletPda.toBuffer(),
    proposer.publicKey.toBuffer(),
  ],
  program.programId
);

await program.methods
  .createProposal(
    new anchor.BN(1_000_000_000), // 1 SOL
    recipient.publicKey,
    false,                        // is_token_transfer
    null                          // mint (null for SOL)
  )
  .accounts({
    proposal: proposalPda,
    teamWallet: teamWalletPda,
    proposer: proposer.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([proposer])
  .rpc();
```

### 3. Vote on Proposal

```typescript
// Vote for the proposal
await program.methods
  .voteProposal(true) // true = vote for, false = vote against
  .accounts({
    proposal: proposalPda,
    teamWallet: teamWalletPda,
    voter: voter.publicKey,
  })
  .signers([voter])
  .rpc();
```

### 4. Execute Proposal

```typescript
// Execute when threshold is met
await program.methods
  .executeProposalSol()
  .accounts({
    proposal: proposalPda,
    teamWallet: teamWalletPda,
    recipient: recipient.publicKey,
    executor: executor.publicKey,
  })
  .signers([executor])
  .rpc();
```

## 📖 Usage Guide

### Member Management

#### Add Voter
```typescript
await program.methods
  .addVoter(newVoter.publicKey)
  .accounts({
    teamWallet: teamWalletPda,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

#### Add Contributor
```typescript
await program.methods
  .addContributor(newContributor.publicKey)
  .accounts({
    teamWallet: teamWalletPda,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

#### Remove Voter
```typescript
await program.methods
  .removeVoter(voterToRemove.publicKey)
  .accounts({
    teamWallet: teamWalletPda,
    owner: owner.publicKey,
  })
  .signers([owner])
  .rpc();
```

### Token Management

#### Transfer Mint Authority
```typescript
await program.methods
  .transferMintAuthority()
  .accounts({
    teamWallet: teamWalletPda,
    mint: mintPubkey,
    currentAuthority: currentAuthority.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .signers([currentAuthority])
  .rpc();
```

#### Create Token Mint Proposal
```typescript
await program.methods
  .createTokenProposal(
    { mint: {} },                    // TokenAction::Mint
    new anchor.BN(1_000_000_000),    // amount
    recipientTokenAccount,           // recipient
    null,                            // metadata
    null,                            // transfer_fee_config
    null                             // interest_rate
  )
  .accounts({
    tokenProposal: tokenProposalPda,
    teamWallet: teamWalletPda,
    mint: mintPubkey,
    proposer: proposer.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([proposer])
  .rpc();
```

#### Set Transfer Fee
```typescript
await program.methods
  .createTokenProposal(
    { setTransferFee: {} },
    0,
    null,
    null,
    {
      transferFeeBasisPoints: 50,  // 0.5%
      maximumFee: new anchor.BN(1_000_000),
    },
    null
  )
  .accounts({...})
  .rpc();
```

#### Update Interest Rate
```typescript
await program.methods
  .createTokenProposal(
    { updateInterestRate: {} },
    0,
    null,
    null,
    null,
    500  // 5% APY
  )
  .accounts({...})
  .rpc();
```

### Program Governance

#### Transfer Program Authority
```typescript
await program.methods
  .transferProgramAuthority()
  .accounts({
    teamWallet: teamWalletPda,
    programId: programId,
    programData: programDataAccount,
    currentAuthority: currentAuthority.publicKey,
    bpfLoaderUpgradeableProgram: BPF_UPGRADEABLE_LOADER_PROGRAM_ID,
  })
  .signers([currentAuthority])
  .rpc();
```

#### Create Upgrade Proposal
```typescript
await program.methods
  .createUpgradeProposal(newBufferPubkey)
  .accounts({
    upgradeProposal: upgradeProposalPda,
    teamWallet: teamWalletPda,
    proposer: proposer.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .signers([proposer])
  .rpc();
```

#### Execute Upgrade
```typescript
await program.methods
  .executeUpgradeProposal()
  .accounts({
    upgradeProposal: upgradeProposalPda,
    teamWallet: teamWalletPda,
    programId: programId,
    programData: programDataAccount,
    buffer: newBufferPubkey,
    spillAccount: spillAccount.publicKey,
    rent: SYSVAR_RENT_PUBKEY,
    clock: SYSVAR_CLOCK_PUBKEY,
    bpfLoaderUpgradeableProgram: BPF_UPGRADEABLE_LOADER_PROGRAM_ID,
    executor: executor.publicKey,
  })
  .signers([executor])
  .rpc();
```

## 🔌 API Reference

### Instructions

#### `initialize_team_wallet`
Initialize a new team wallet with initial voters.

**Parameters:**
- `name: String` - Team wallet name
- `vote_threshold: u8` - Percentage required to pass (1-100)
- `voters: Vec<Pubkey>` - Initial voter addresses (max 10)

#### `add_voter`
Add a new voter to the team wallet.

**Parameters:**
- `voter_pubkey: Pubkey` - Address to add as voter

#### `remove_voter`
Remove a voter from the team wallet.

**Parameters:**
- `voter_pubkey: Pubkey` - Address to remove

#### `add_contributor`
Add a new contributor to the team wallet.

**Parameters:**
- `contributor_pubkey: Pubkey` - Address to add as contributor

#### `remove_contributor`
Remove a contributor from the team wallet.

**Parameters:**
- `contributor_pubkey: Pubkey` - Address to remove

#### `create_proposal`
Create a proposal for SOL or token transfer.

**Parameters:**
- `amount: u64` - Amount to transfer
- `recipient: Pubkey` - Recipient address
- `is_token_transfer: bool` - true for tokens, false for SOL
- `mint: Option<Pubkey>` - Token mint (required if is_token_transfer)

#### `vote_proposal`
Vote on an existing proposal.

**Parameters:**
- `vote_for: bool` - true to vote for, false to vote against

#### `execute_proposal_sol`
Execute an approved SOL transfer proposal.

#### `execute_proposal_token`
Execute an approved token transfer proposal.

#### `create_token_proposal`
Create a token management proposal.

**Parameters:**
- `action: TokenAction` - Action to perform
- `amount: u64` - Amount (for mint/burn)
- `recipient: Option<Pubkey>` - Target account
- `metadata: Option<TokenMetadataParams>` - Metadata info
- `transfer_fee_config: Option<TransferFeeParams>` - Fee config
- `interest_rate: Option<i16>` - Interest rate

#### `execute_token_proposal`
Execute an approved token management proposal.

#### `create_upgrade_proposal`
Create a program upgrade proposal.

**Parameters:**
- `new_buffer: Pubkey` - Buffer with new program code

#### `execute_upgrade_proposal`
Execute an approved program upgrade.

#### `transfer_program_authority`
Transfer program upgrade authority to team wallet.

#### `transfer_mint_authority`
Transfer token mint authority to team wallet.

### Types

#### `TokenAction` Enum
```rust
pub enum TokenAction {
    Mint,
    Burn,
    FreezAccount,
    ThawAccount,
    SetAuthority,
    UpdateMetadata,
    SetTransferFee,
    WithdrawTransferFees,
    EnableConfidentialTransfers,
    DisableConfidentialTransfers,
    UpdateInterestRate,
    SetPermanentDelegate,
    UpdateGroupPointer,
    UpdateMemberPointer,
}
```

## 🧪 Testing

### Run All Tests
```bash
anchor test
```

### Run Specific Test
```bash
anchor test -- --test test_name
```

### Test Coverage
```bash
cargo tarpaulin --output-dir coverage
```

## 🔒 Security

### Audit Status
- ⏳ Pending professional audit
- ✅ Community reviewed
- ✅ Testnet deployed

### Security Features
- ✅ PDA-based security
- ✅ Multi-signature requirements
- ✅ Vote threshold enforcement
- ✅ Proposal execution controls
- ✅ Authority validation
- ✅ Reentrancy protection

### Best Practices
1. **Start with high thresholds** (75%+) for critical operations
2. **Test on devnet** before mainnet deployment
3. **Regular audits** of proposals and members
4. **Monitor all transactions** for suspicious activity
5. **Backup recovery plans** for lost keys

### Known Limitations
- Maximum 10 voters per wallet
- Maximum 10 contributors per wallet
- Proposals cannot be cancelled once created
- Vote changes not allowed after submission

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details.

### Development Setup
```bash
# Fork and clone
git clone https://github.com/yourusername/teamwallet-contract
cd teamwallet-contract

# Create branch
git checkout -b feature/your-feature

# Make changes and test
anchor test

# Submit PR
git push origin feature/your-feature
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Solana Foundation
- Anchor Framework Team
- Metaplex Foundation
- SPL Token Team
- Community Contributors

## 📊 Stats

- **Lines of Code**: ~2,500
- **Test Coverage**: 85%+
- **Dependencies**: 7 core
- **Network**: Solana Mainnet-Beta
