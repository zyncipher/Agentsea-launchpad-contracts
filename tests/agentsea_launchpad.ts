import * as anchor from "@project-serum/anchor";
import { Program } from "@project-serum/anchor";
import { AgentseaLaunchpad } from "../target/types/agentsea_launchpad";
import { PublicKey, SystemProgram } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo } from "@solana/spl-token";

describe("agentsea_launchpad", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.AgentseaLaunchpad as Program<AgentseaLaunchpad>;
  const provider = anchor.AnchorProvider.env();

  let agentsTokenMint: PublicKey;
  let launchpadPda: PublicKey;

  it("Initialize launchpad", async () => {
    // Create $AGENTS token mint
    agentsTokenMint = await createMint(
      provider.connection,
      (provider.wallet as any).payer,
      provider.wallet.publicKey,
      null,
      9
    );

    // Derive launchpad PDA
    [launchpadPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("launchpad")],
      program.programId
    );

    // Initialize launchpad
    const tx = await program.methods
      .initializeLaunchpad(new anchor.BN(1_000_000_000)) // 1 token minimum stake
      .accounts({
        launchpad: launchpadPda,
        authority: provider.wallet.publicKey,
        agentsTokenMint: agentsTokenMint,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Launchpad initialized:", tx);

    // Fetch launchpad state
    const launchpadAccount = await program.account.launchpad.fetch(launchpadPda);
    console.log("Launchpad state:", launchpadAccount);
  });

  it("Register an agent", async () => {
    const agentCount = (await program.account.launchpad.fetch(launchpadPda)).agentCount;

    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), agentCount.toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const tx = await program.methods
      .registerAgent(
        "AI Trading Bot",
        "https://ipfs.io/ipfs/QmExample",
        "An AI agent that trades cryptocurrencies"
      )
      .accounts({
        agent: agentPda,
        launchpad: launchpadPda,
        owner: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Agent registered:", tx);

    const agent = await program.account.agent.fetch(agentPda);
    console.log("Agent:", agent);
  });

  it("Stake tokens to agent", async () => {
    const agentCount = (await program.account.launchpad.fetch(launchpadPda)).agentCount;

    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), new anchor.BN(agentCount.toNumber() - 1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [stakeAccountPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake"), agentPda.toBuffer(), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const [stakeVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("stake_vault"), agentPda.toBuffer()],
      program.programId
    );

    // Create staker token account and mint tokens
    const stakerTokenAccount = await createAccount(
      provider.connection,
      (provider.wallet as any).payer,
      agentsTokenMint,
      provider.wallet.publicKey
    );

    await mintTo(
      provider.connection,
      (provider.wallet as any).payer,
      agentsTokenMint,
      stakerTokenAccount,
      provider.wallet.publicKey,
      10_000_000_000 // 10 tokens
    );

    const tx = await program.methods
      .stakeToAgent(new anchor.BN(5_000_000_000)) // Stake 5 tokens
      .accounts({
        agent: agentPda,
        stakeAccount: stakeAccountPda,
        launchpad: launchpadPda,
        stakeVault: stakeVaultPda,
        staker: provider.wallet.publicKey,
        stakerTokenAccount: stakerTokenAccount,
        agentsTokenMint: agentsTokenMint,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Tokens staked:", tx);

    const stakeAccount = await program.account.stakeAccount.fetch(stakeAccountPda);
    console.log("Stake account:", stakeAccount);
  });

  it("Give feedback to agent", async () => {
    const agentCount = (await program.account.launchpad.fetch(launchpadPda)).agentCount;

    const [agentPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("agent"), new anchor.BN(agentCount.toNumber() - 1).toArrayLike(Buffer, "le", 8)],
      program.programId
    );

    const [feedbackPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("feedback"), agentPda.toBuffer(), provider.wallet.publicKey.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .giveFeedback(85, "https://feedback.example.com/review1")
      .accounts({
        agent: agentPda,
        feedback: feedbackPda,
        reviewer: provider.wallet.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Feedback given:", tx);

    const agent = await program.account.agent.fetch(agentPda);
    console.log("Agent reputation score:", agent.reputationScore);

    const feedback = await program.account.feedback.fetch(feedbackPda);
    console.log("Feedback:", feedback);
  });
});
