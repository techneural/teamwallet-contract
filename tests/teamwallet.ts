import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Teamwallet } from "../target/types/teamwallet";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { expect } from "chai";
import {
  fundAccount,
  fundAccounts,
  deriveTeamWalletPda,
  deriveProposalPda,
  generateNonce,
  Actions,
} from "./utils";

describe("TeamWallet Unified", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.Teamwallet as Program<Teamwallet>;
  
  // Test accounts
  let owner: Keypair;
  let voter1: Keypair;
  let voter2: Keypair;
  let voter3: Keypair;
  let contributor1: Keypair;
  let recipient: Keypair;
  
  // PDAs
  let teamWalletPda: PublicKey;
  
  // Use unique wallet name for each test run
  const WALLET_NAME = "tw-" + Date.now().toString(36);

  before(async () => {
    console.log("\n═══════════════════════════════════════════════════════");
    console.log("Setting up test accounts...");
    console.log("Provider wallet:", provider.wallet.publicKey.toString());
    
    // Generate keypairs
    owner = Keypair.generate();
    voter1 = Keypair.generate();
    voter2 = Keypair.generate();
    voter3 = Keypair.generate();
    contributor1 = Keypair.generate();
    recipient = Keypair.generate();

    // Fund all accounts with MORE SOL (1 SOL for owner, 0.2 for others)
    await fundAccount(provider, owner.publicKey, 1 * LAMPORTS_PER_SOL);
    await fundAccounts(provider, [
      voter1.publicKey,
      voter2.publicKey,
      voter3.publicKey,
      contributor1.publicKey,
      recipient.publicKey,
    ], 0.2 * LAMPORTS_PER_SOL);

    console.log("✓ Owner funded with 1 SOL, others with 0.2 SOL each");

    // Derive team wallet PDA
    [teamWalletPda] = deriveTeamWalletPda(
      program.programId,
      owner.publicKey,
      WALLET_NAME
    );
    
    console.log("Team Wallet PDA:", teamWalletPda.toString());
    console.log("Wallet Name:", WALLET_NAME);
    console.log("═══════════════════════════════════════════════════════\n");
  });
  
  // Helper to refund owner if needed
  async function ensureOwnerFunded(minBalance: number = 0.3 * LAMPORTS_PER_SOL) {
    const balance = await provider.connection.getBalance(owner.publicKey);
    if (balance < minBalance) {
      console.log(`  Refunding owner (balance: ${balance / LAMPORTS_PER_SOL} SOL)`);
      await fundAccount(provider, owner.publicKey, 0.5 * LAMPORTS_PER_SOL);
    }
  }

  // ═══════════════════════════════════════════════════════════════════════════
  // INITIALIZATION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("1. Initialize Team Wallet", () => {
    it("should initialize a team wallet with owner and voters", async () => {
      const voters = [voter1.publicKey, voter2.publicKey];
      const threshold = 2;

      const tx = await program.methods
        .initializeTeamWallet(WALLET_NAME, threshold, voters)
        .accounts({
          teamWallet: teamWalletPda,
          owner: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();
      
      console.log("Init TX:", tx);

      // Wait a bit for confirmation
      await new Promise(r => setTimeout(r, 1000));

      const teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      
      expect(teamWallet.owner.toString()).to.equal(owner.publicKey.toString());
      expect(teamWallet.name).to.equal(WALLET_NAME);
      expect(teamWallet.voteThreshold).to.equal(threshold);
      expect(teamWallet.voterCount).to.equal(3); // owner + 2 voters
      expect(teamWallet.voters.length).to.equal(3);
      expect(teamWallet.contributors.length).to.equal(0);
      
      console.log("✓ Team wallet initialized successfully");
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // PROPOSAL CREATION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("2. Create Proposals", () => {
    it("should create a TransferSol proposal", async () => {
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      const amount = new anchor.BN(0.01 * LAMPORTS_PER_SOL);

      await program.methods
        .createProposal(Actions.transferSol(amount, recipient.publicKey), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      
      expect(proposal.votesFor).to.equal(1); // Proposer auto-votes
      expect(proposal.votesAgainst).to.equal(0);
      expect(proposal.executed).to.equal(false);
      expect(proposal.cancelled).to.equal(false);
      
      console.log("✓ TransferSol proposal created");
    });

    it("should fail if non-voter tries to create proposal", async () => {
      const nonVoter = Keypair.generate();
      await fundAccount(provider, nonVoter.publicKey, 0.1 * LAMPORTS_PER_SOL);

      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      try {
        await program.methods
          .createProposal(
            Actions.transferSol(new anchor.BN(LAMPORTS_PER_SOL), recipient.publicKey),
            nonce.publicKey
          )
          .accounts({
            proposal: proposalPda,
            teamWallet: teamWalletPda,
            proposer: nonVoter.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([nonVoter])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("NotAVoterOrContributor");
        console.log("✓ Non-voter rejected");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // VOTING TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("3. Vote on Proposals", () => {
    it("should allow voter to vote and reach threshold", async () => {
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      // Create proposal
      await program.methods
        .createProposal(
          Actions.transferSol(new anchor.BN(0.001 * LAMPORTS_PER_SOL), recipient.publicKey),
          nonce.publicKey
        )
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      // Check initial state - owner auto-voted
      let proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.votesFor).to.equal(1);
      console.log("✓ Owner auto-voted (1 vote)");

      // Voter1 votes FOR
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.votesFor).to.equal(2);
      expect(proposal.approved).to.equal(true); // Threshold reached
      console.log("✓ Voter1 voted (2 votes - threshold reached)");
    });

    it("should prevent double voting", async () => {
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(
          Actions.transferSol(new anchor.BN(0.001 * LAMPORTS_PER_SOL), recipient.publicKey),
          nonce.publicKey
        )
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      // Voter1 votes
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      // Voter1 tries to vote again
      try {
        await program.methods
          .voteProposal(true)
          .accounts({
            proposal: proposalPda,
            teamWallet: teamWalletPda,
            voter: voter1.publicKey,
          })
          .signers([voter1])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("AlreadyVoted");
        console.log("✓ Double voting prevented");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // EXECUTION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("4. Execute Proposals", () => {
    it("should execute TransferSol proposal", async () => {
      // Fund team wallet PDA
      await fundAccount(provider, teamWalletPda, 0.5 * LAMPORTS_PER_SOL);
      console.log("✓ Team wallet funded");

      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      const transferAmount = new anchor.BN(0.01 * LAMPORTS_PER_SOL);

      // Create proposal
      await program.methods
        .createProposal(Actions.transferSol(transferAmount, recipient.publicKey), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      // Vote to reach threshold
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      const recipientBalanceBefore = await provider.connection.getBalance(recipient.publicKey);

      // Execute
      await program.methods
        .executeProposal(null)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          executor: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .remainingAccounts([
          { pubkey: recipient.publicKey, isSigner: false, isWritable: true },
        ])
        .signers([owner])
        .rpc();

      const recipientBalanceAfter = await provider.connection.getBalance(recipient.publicKey);
      const proposal = await program.account.proposal.fetch(proposalPda);

      expect(proposal.executed).to.equal(true);
      expect(recipientBalanceAfter - recipientBalanceBefore).to.equal(0.01 * LAMPORTS_PER_SOL);
      
      console.log("✓ TransferSol executed successfully");
    });

    it("should execute ChangeThreshold proposal", async () => {
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(Actions.changeThreshold(1), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      // Vote to reach threshold (currently 2)
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      // Execute
      await program.methods
        .executeProposal(null)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          executor: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      expect(teamWallet.voteThreshold).to.equal(1);
      
      console.log("✓ ChangeThreshold executed (now threshold = 1)");
    });

    it("should execute AddVoter proposal", async () => {
      await ensureOwnerFunded();
      
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      const teamWalletBefore = await program.account.teamWallet.fetch(teamWalletPda);

      await program.methods
        .createProposal(Actions.addVoter(voter3.publicKey), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      // With threshold=1, auto-approved and can execute immediately
      await program.methods
        .executeProposal(null)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          executor: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      expect(teamWallet.voterCount).to.equal(teamWalletBefore.voterCount + 1);
      
      console.log("✓ AddVoter executed");
    });

    it("should execute AddContributor proposal", async () => {
      await ensureOwnerFunded();
      
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(Actions.addContributor(contributor1.publicKey), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      await program.methods
        .executeProposal(null)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          executor: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      expect(teamWallet.contributors.map(c => c.toString())).to.include(contributor1.publicKey.toString());
      
      console.log("✓ AddContributor executed");
    });

    it("should fail to remove owner", async () => {
      await ensureOwnerFunded();
      
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(Actions.removeVoter(owner.publicKey), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      try {
        await program.methods
          .executeProposal(null)
          .accounts({
            proposal: proposalPda,
            teamWallet: teamWalletPda,
            executor: owner.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([owner])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("CannotRemoveOwner");
        console.log("✓ Owner removal prevented");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // CANCELLATION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("5. Cancel Proposals", () => {
    it("should allow proposer to cancel", async () => {
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(
          Actions.transferSol(new anchor.BN(0.001 * LAMPORTS_PER_SOL), recipient.publicKey),
          nonce.publicKey
        )
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: voter1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([voter1])
        .rpc();

      await program.methods
        .cancelProposal()
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          canceller: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.cancelled).to.equal(true);
      
      console.log("✓ Proposer cancelled successfully");
    });

    it("should fail to execute cancelled proposal", async () => {
      await ensureOwnerFunded();
      
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      await program.methods
        .createProposal(Actions.changeThreshold(2), nonce.publicKey)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      await program.methods
        .cancelProposal()
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          canceller: owner.publicKey,
        })
        .signers([owner])
        .rpc();

      try {
        await program.methods
          .executeProposal(null)
          .accounts({
            proposal: proposalPda,
            teamWallet: teamWalletPda,
            executor: owner.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([owner])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("ProposalAlreadyCancelled");
        console.log("✓ Cancelled proposal execution prevented");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // CONTRIBUTOR TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("6. Contributor Permissions", () => {
    it("contributor can create proposals but not vote", async () => {
      // First ensure contributor1 is actually a contributor
      let teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      const isContributor = teamWallet.contributors.some(
        c => c.toString() === contributor1.publicKey.toString()
      );
      
      if (!isContributor) {
        console.log("  Adding contributor1 first...");
        await ensureOwnerFunded();
        
        const addNonce = generateNonce();
        const [addProposalPda] = deriveProposalPda(
          program.programId,
          teamWalletPda,
          addNonce.publicKey
        );
        
        await program.methods
          .createProposal(Actions.addContributor(contributor1.publicKey), addNonce.publicKey)
          .accounts({
            proposal: addProposalPda,
            teamWallet: teamWalletPda,
            proposer: owner.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([owner])
          .rpc();
        
        await program.methods
          .executeProposal(null)
          .accounts({
            proposal: addProposalPda,
            teamWallet: teamWalletPda,
            executor: owner.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([owner])
          .rpc();
        
        console.log("  ✓ Contributor added");
      }
      
      const nonce = generateNonce();
      const [proposalPda] = deriveProposalPda(
        program.programId,
        teamWalletPda,
        nonce.publicKey
      );

      // Contributor creates proposal
      await program.methods
        .createProposal(
          Actions.transferSol(new anchor.BN(0.001 * LAMPORTS_PER_SOL), recipient.publicKey),
          nonce.publicKey
        )
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          proposer: contributor1.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([contributor1])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.proposer.toString()).to.equal(contributor1.publicKey.toString());
      expect(proposal.votesFor).to.equal(0); // Contributors don't auto-vote
      
      console.log("✓ Contributor created proposal (no auto-vote)");

      // Contributor cannot vote
      try {
        await program.methods
          .voteProposal(true)
          .accounts({
            proposal: proposalPda,
            teamWallet: teamWalletPda,
            voter: contributor1.publicKey,
          })
          .signers([contributor1])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("NotAuthorizedToVote");
        console.log("✓ Contributor voting prevented");
      }
    });
  });
});
