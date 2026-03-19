import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { Teamwallet } from "../target/types/teamwallet";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";
import { expect } from "chai";
import {
  fundAccount,
  fundAccounts,
  deriveTeamWalletPda,
  deriveProposalPda,
  generateNonce,
  Actions,
  createTestMint,
  createATA,
  mintTokens,
  getTokenBalance,
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
  
  const WALLET_NAME = "test-wallet-" + Math.random().toString(36).substring(7);

  before(async () => {
    console.log("Setting up test accounts...");
    console.log("Provider wallet:", provider.wallet.publicKey.toString());
    
    // Generate keypairs
    owner = Keypair.generate();
    voter1 = Keypair.generate();
    voter2 = Keypair.generate();
    voter3 = Keypair.generate();
    contributor1 = Keypair.generate();
    recipient = Keypair.generate();

    // Fund all accounts from provider wallet (not airdrop!)
    await fundAccounts(provider, [
      owner.publicKey,
      voter1.publicKey,
      voter2.publicKey,
      voter3.publicKey,
      contributor1.publicKey,
      recipient.publicKey,
    ], 0.1 * LAMPORTS_PER_SOL);

    console.log("Accounts funded");

    // Derive team wallet PDA
    [teamWalletPda] = deriveTeamWalletPda(
      program.programId,
      owner.publicKey,
      WALLET_NAME
    );
    
    console.log("Team Wallet PDA:", teamWalletPda.toString());
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // INITIALIZATION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("Initialize Team Wallet", () => {
    it("should initialize a team wallet with owner and voters", async () => {
      const voters = [voter1.publicKey, voter2.publicKey];
      const threshold = 2;

      await program.methods
        .initializeTeamWallet(WALLET_NAME, threshold, voters)
        .accounts({
          teamWallet: teamWalletPda,
          owner: owner.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .signers([owner])
        .rpc();

      const teamWallet = await program.account.teamWallet.fetch(teamWalletPda);
      
      expect(teamWallet.owner.toString()).to.equal(owner.publicKey.toString());
      expect(teamWallet.name).to.equal(WALLET_NAME);
      expect(teamWallet.voteThreshold).to.equal(threshold);
      expect(teamWallet.voterCount).to.equal(3); // owner + 2 voters
      expect(teamWallet.voters.length).to.equal(3);
      expect(teamWallet.contributors.length).to.equal(0);
      
      console.log("✓ Team wallet initialized");
    });

    it("should fail with invalid threshold (too high)", async () => {
      const newOwner = Keypair.generate();
      await fundAccount(provider, newOwner.publicKey, 0.05 * LAMPORTS_PER_SOL);

      const [newWalletPda] = deriveTeamWalletPda(
        program.programId,
        newOwner.publicKey,
        "invalid-threshold"
      );

      try {
        await program.methods
          .initializeTeamWallet("invalid-threshold", 10, []) // threshold 10 with 1 voter
          .accounts({
            teamWallet: newWalletPda,
            owner: newOwner.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .signers([newOwner])
          .rpc();
        expect.fail("Should have thrown an error");
      } catch (err: any) {
        expect(err.message).to.include("InvalidThreshold");
        console.log("✓ Invalid threshold rejected");
      }
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // PROPOSAL CREATION TESTS
  // ═══════════════════════════════════════════════════════════════════════════

  describe("Create Proposals", () => {
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

    it("should create a ChangeThreshold proposal", async () => {
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

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.action.changeThreshold).to.not.be.undefined;
      
      console.log("✓ ChangeThreshold proposal created");
    });

    it("should fail if non-voter tries to create proposal", async () => {
      const nonVoter = Keypair.generate();
      await fundAccount(provider, nonVoter.publicKey, 0.05 * LAMPORTS_PER_SOL);

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

  describe("Vote on Proposals", () => {
    let proposalPda: PublicKey;
    let nonce: Keypair;

    beforeEach(async () => {
      nonce = generateNonce();
      [proposalPda] = deriveProposalPda(
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
    });

    it("should allow voter to vote FOR", async () => {
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.votesFor).to.equal(2); // owner + voter1
      
      console.log("✓ Vote FOR recorded");
    });

    it("should allow voter to vote AGAINST", async () => {
      await program.methods
        .voteProposal(false)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.votesFor).to.equal(1);
      expect(proposal.votesAgainst).to.equal(1);
      
      console.log("✓ Vote AGAINST recorded");
    });

    it("should fail if voter already voted", async () => {
      await program.methods
        .voteProposal(true)
        .accounts({
          proposal: proposalPda,
          teamWallet: teamWalletPda,
          voter: voter1.publicKey,
        })
        .signers([voter1])
        .rpc();

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

  describe("Execute Proposals", () => {
    it("should execute TransferSol proposal", async () => {
      // Fund team wallet
      await fundAccount(provider, teamWalletPda, 0.1 * LAMPORTS_PER_SOL);

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
      
      console.log("✓ TransferSol executed");
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

      // Vote
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
      
      console.log("✓ ChangeThreshold executed");
    });

    it("should execute AddVoter proposal", async () => {
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

      // With threshold=1, auto-approved
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

  describe("Cancel Proposals", () => {
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
      
      console.log("✓ Proposer cancelled");
    });

    it("should allow owner to cancel any proposal", async () => {
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
          canceller: owner.publicKey,
        })
        .signers([owner])
        .rpc();

      const proposal = await program.account.proposal.fetch(proposalPda);
      expect(proposal.cancelled).to.equal(true);
      
      console.log("✓ Owner cancelled");
    });

    it("should fail to execute cancelled proposal", async () => {
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

  describe("Contributor Permissions", () => {
    it("contributor can create proposals but not vote", async () => {
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
      
      console.log("✓ Contributor created proposal");

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
