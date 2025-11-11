# AgentSea Launchpad - Solana

A Solana implementation of the EIP-8004 protocol for discovering, registering, and building trust in autonomous AI agents.

## Devnet Deployment

| Component | Address |
|-----------|---------|
| **Program ID** | `DZxHnHSzHfgoYd5qpjvxY67BRmS4Du6kjAZD5v4TPyWT` |
| **$AGENTS Token Mint** | `6SJDrLFSxkvnMQzKCjPgMU5wL73k61W2NQ2remvjGRa4` |
| **Launchpad PDA** | `49Y94xbg8G45bpgbbcLBG3ioNnmjD6fsnGJjA2QANJR4` |
| **Deployer Wallet** | `3yYVTf6Fpoey6VwzUcf4wyQW3EETFTha8oC4qkCWwi1v` |
| **Network** | Solana Devnet |
| **RPC URL** | `https://api.devnet.solana.com` |

**Devnet Explorer Links:**
- [Program](https://explorer.solana.com/address/DZxHnHSzHfgoYd5qpjvxY67BRmS4Du6kjAZD5v4TPyWT?cluster=devnet)
- [$AGENTS Token](https://explorer.solana.com/address/6SJDrLFSxkvnMQzKCjPgMU5wL73k61W2NQ2remvjGRa4?cluster=devnet)
- [Launchpad Account](https://explorer.solana.com/address/49Y94xbg8G45bpgbbcLBG3ioNnmjD6fsnGJjA2QANJR4?cluster=devnet)

## Overview

AgentSea Launchpad is a decentralized marketplace for AI agents where:
- Developers can register and launch AI agents
- Users can discover agents by browsing registries
- Community can stake $AGENTS tokens to back trusted agents
- Reputation is built through on-chain feedback and ratings

## Architecture

```
┌───────────────────────────────────────────────────────────┐
│                   AgentSea Launchpad                      │
├───────────────────────────────────────────────────────────┤
│                                                           │
│   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐    │
│   │    Agent     │  │   Staking    │  │  Reputation  │    │
│   │   Registry   │  │   System     │  │   System     │    │
│   └──────────────┘  └──────────────┘  └──────────────┘    │
│                                                           │
│   • Register AI agents with metadata                      │
│   • Stake $AGENTS tokens to support agents                │
│   • Give feedback & build reputation (0-100 scores)       │
│   • Discover agents by reputation & stake                 │
│                                                           │
└───────────────────────────────────────────────────────────┘
```

## What is EIP-8004?

**EIP-8004: Non-Fungible Trustless Agents** is a protocol standard originally designed for Ethereum that enables:

1. **Agent Discovery** - Find and identify AI agents across networks
2. **Trust Building** - Establish confidence in agents without centralized intermediaries
3. **Reputation Tracking** - On-chain feedback and validation systems

### Key Concepts

- **Agent** - An autonomous AI program (trading bot, data analyzer, automation tool, etc.)
- **$AGENTS Token** - Utility token used for staking and governance
- **Staking** - Lock tokens to show support/trust in specific agents
- **Reputation** - On-chain rating system (0-100 scores)

## Program Features

### 1. Agent Registration
Register AI agents with rich metadata:
- **Unique Agent ID** - Auto-incremented identifier
- **Name** - Agent name (max 50 chars)
- **Metadata URI** - IPFS/HTTPS link to full metadata
- **Description** - What the agent does (max 500 chars)
- **Owner** - Agent creator/maintainer

### 2. Token Staking
Stake $AGENTS tokens to support agents:
- **Minimum Stake** - Configurable minimum amount
- **Stake Tracking** - Per-user stake amounts
- **Total Staked** - Aggregate stake per agent
- **Unstaking** - Withdraw staked tokens anytime

### 3. Reputation System
Build trust through on-chain feedback:
- **Score** - 0-100 rating scale
- **Feedback URI** - Link to detailed review
- **Average Reputation** - Calculated on-chain
- **Feedback Count** - Track total reviews

### 4. On-Chain Events
All actions emit events for indexing:
- `AgentRegistered` - New agent added
- `TokensStaked` - Tokens staked to agent
- `TokensUnstaked` - Tokens withdrawn
- `FeedbackGiven` - New rating submitted

## Program Instructions

### Initialize Launchpad
```rust
initialize_launchpad(min_stake_amount: u64)
```
Sets up the global launchpad with minimum staking requirements.

**Accounts:**
- `launchpad` - PDA for launchpad state
- `authority` - Admin/deployer
- `agents_token_mint` - $AGENTS token mint address

### Register Agent
```rust
register_agent(
    name: String,          // Max 50 chars
    metadata_uri: String,  // Max 200 chars
    description: String    // Max 500 chars
)
```
Registers a new AI agent.

**Accounts:**
- `agent` - PDA for agent state
- `launchpad` - Global launchpad state
- `owner` - Agent creator

**Example Metadata URI JSON:**
```json
{
  "type": "agentsea-agent-v1",
  "name": "AI Trading Bot",
  "description": "Automated crypto trading agent",
  "image": "https://ipfs.io/ipfs/QmXXX",
  "version": "1.0.0",
  "endpoints": [
    {
      "type": "mcp",
      "url": "https://agent.example.com/mcp"
    }
  ],
  "capabilities": ["trading", "analysis", "risk-management"],
  "pricing": {
    "model": "subscription",
    "amount": 10,
    "currency": "USDC"
  }
}
```

### Stake to Agent
```rust
stake_to_agent(amount: u64)
```
Stake $AGENTS tokens to support an agent.

**Accounts:**
- `agent` - Agent to stake to
- `stake_account` - PDA tracking user's stake
- `stake_vault` - PDA holding staked tokens
- `staker` - User staking tokens
- `staker_token_account` - User's $AGENTS token account

### Unstake from Agent
```rust
unstake_from_agent(amount: u64)
```
Withdraw previously staked tokens.

**Accounts:**
- `agent` - Agent staked to
- `stake_account` - User's stake tracking
- `stake_vault` - Vault holding tokens
- `staker` - User unstaking

### Give Feedback
```rust
give_feedback(
    score: u8,            // 0-100
    comment_uri: String   // Max 200 chars
)
```
Rate an agent and provide feedback.

**Accounts:**
- `agent` - Agent being reviewed
- `feedback` - PDA for feedback record
- `reviewer` - User giving feedback

### Update Agent Metadata
```rust
update_agent_metadata(
    new_metadata_uri: String,
    new_description: String
)
```
Update agent information (owner only).

**Accounts:**
- `agent` - Agent to update
- `owner` - Agent owner (must match)

## Account Structures

### Launchpad
```rust
pub struct Launchpad {
    pub authority: Pubkey,           // Admin
    pub agent_count: u64,            // Total agents
    pub min_stake_amount: u64,       // Min stake required
    pub agents_token_mint: Pubkey,   // $AGENTS mint
}
```
**Space:** 8 + 32 + 8 + 8 + 32 = 88 bytes

### Agent
```rust
pub struct Agent {
    pub agent_id: u64,              // Unique ID
    pub owner: Pubkey,              // Creator
    pub name: String,               // Agent name
    pub metadata_uri: String,       // IPFS/HTTPS link
    pub description: String,        // Description
    pub total_staked: u64,          // Total $AGENTS staked
    pub reputation_score: u8,       // Average score (0-100)
    pub feedback_count: u32,        // Total reviews
    pub is_active: bool,            // Active status
    pub created_at: i64,            // Timestamp
}
```
**Space:** 8 + 8 + 32 + (4+50) + (4+200) + (4+500) + 8 + 1 + 4 + 1 + 8 = 822 bytes

### StakeAccount
```rust
pub struct StakeAccount {
    pub agent: Pubkey,              // Agent staked to
    pub staker: Pubkey,             // Staker address
    pub amount: u64,                // Amount staked
    pub staked_at: i64,             // Stake timestamp
}
```
**Space:** 8 + 32 + 32 + 8 + 8 = 88 bytes

### Feedback
```rust
pub struct Feedback {
    pub agent: Pubkey,              // Agent reviewed
    pub reviewer: Pubkey,           // Reviewer
    pub score: u8,                  // Score (0-100)
    pub comment_uri: String,        // Comment link
    pub timestamp: i64,             // Review time
}
```
**Space:** 8 + 32 + 32 + 1 + (4+200) + 8 = 285 bytes

## PDA Seeds

| Account | Seeds |
|---------|-------|
| Launchpad | `["launchpad"]` |
| Agent | `["agent", agent_count.to_le_bytes()]` |
| StakeAccount | `["stake", agent_pubkey, staker_pubkey]` |
| StakeVault | `["stake_vault", agent_pubkey]` |
| Feedback | `["feedback", agent_pubkey, reviewer_pubkey]` |

## Error Codes

| Code | Message |
|------|---------|
| `NameTooLong` | Name exceeds 50 characters |
| `UriTooLong` | URI exceeds 200 characters |
| `DescriptionTooLong` | Description exceeds 500 characters |
| `InvalidAmount` | Amount must be greater than 0 |
| `StakeTooLow` | Stake below minimum required |
| `InsufficientStake` | Not enough stake in account |
| `InvalidScore` | Score must be 0-100 |
| `Unauthorized` | Operation not allowed |
| `MathOverflow` | Arithmetic overflow |

## Setup & Deployment

### Prerequisites
- Rust 1.75+
- Solana CLI 1.18.17+
- Anchor 0.28.0
- Node.js 16+

### Build
```bash
anchor build
```

### Test
```bash
# Start local validator
solana-test-validator

# Run tests
anchor test --skip-local-validator
```

### Deploy to Devnet
```bash
# Set to devnet
solana config set --url devnet

# Airdrop SOL for deployment
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet
```

### Create $AGENTS Token
```bash
# Create token mint
spl-token create-token --decimals 9

# Create your token account
spl-token create-account <MINT_ADDRESS>

# Mint initial supply
spl-token mint <MINT_ADDRESS> 1000000000
```

## Usage Example (TypeScript)

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { AgentseaLaunchpad } from "./target/types/agentsea_launchpad";

const program = anchor.workspace.AgentseaLaunchpad as Program<AgentseaLaunchpad>;
const provider = anchor.AnchorProvider.env();

// 1. Initialize Launchpad
const [launchpadPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("launchpad")],
  program.programId
);

await program.methods
  .initializeLaunchpad(new anchor.BN(1_000_000_000)) // 1 token min
  .accounts({
    launchpad: launchpadPda,
    authority: provider.wallet.publicKey,
    agentsTokenMint: agentsTokenMint,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// 2. Register Agent
const agentCount = 0; // First agent
const [agentPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("agent"), new anchor.BN(agentCount).toArrayLike(Buffer, "le", 8)],
  program.programId
);

await program.methods
  .registerAgent(
    "AI Trading Bot",
    "https://ipfs.io/ipfs/QmXXX",
    "Autonomous crypto trading agent with risk management"
  )
  .accounts({
    agent: agentPda,
    launchpad: launchpadPda,
    owner: provider.wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();

// 3. Stake Tokens
const [stakeAccountPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("stake"), agentPda.toBuffer(), provider.wallet.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .stakeToAgent(new anchor.BN(5_000_000_000)) // 5 tokens
  .accounts({
    agent: agentPda,
    stakeAccount: stakeAccountPda,
    // ... other accounts
  })
  .rpc();

// 4. Give Feedback
const [feedbackPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("feedback"), agentPda.toBuffer(), provider.wallet.publicKey.toBuffer()],
  program.programId
);

await program.methods
  .giveFeedback(95, "https://reviews.example.com/agent1")
  .accounts({
    agent: agentPda,
    feedback: feedbackPda,
    reviewer: provider.wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```
## Use Cases

### 1. AI Trading Bots
- Register automated trading strategies
- Users stake to back profitable bots
- Rate bots based on performance
- Discover top-performing agents by stake/reputation

### 2. Data Analysis Agents
- Register data processing agents
- Stake to support accurate analyzers
- Review data quality and insights
- Find specialized analysis agents

### 3. Automation Tools
- List workflow automation agents
- Community backs reliable tools
- Rate based on reliability
- Discover automation solutions

### 4. Content Creation Agents
- Register AI writers, designers, etc.
- Stake on quality creators
- Review output quality
- Find creative AI assistants

## Security Considerations

### ✅ Implemented Protections
- **PDA Authority** - Only launchpad PDA can transfer staked tokens
- **Owner-Only Updates** - Only agent owner can update metadata
- **Checked Math** - All arithmetic uses checked operations
- **Input Validation** - String lengths and scores validated
- **Proper Seeds** - All PDAs use proper seed derivation

### ⚠️ Limitations (Hackathon Scope)
- No slashing mechanism for bad actors
- No governance for parameter changes
- Simplified reputation (simple average)
- No validator staking system
- No reward distribution

## Future Enhancements

- [ ] Validator staking with slashing
- [ ] Advanced reputation algorithms (weighted, time-decay)
- [ ] Governance for launchpad parameters
- [ ] Reward distribution for stakers
- [ ] Agent categories/tags
- [ ] Search and filter functionality
- [ ] Integration with Solana Name Service
- [ ] Mobile SDK
- [ ] Agent marketplace fees
- [ ] Dispute resolution system

## License

MIT

## Resources

- [EIP-8004 Specification](../ERC8004SPEC.md)
- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [SPL Token Program](https://spl.solana.com/token)

## Support

For issues or questions:
- GitHub Issues: [Create an issue]
- Discord: [AgentSea Community]

---

**Built for hackathons** 🚀 **Powered by Solana & Anchor** ⚡
