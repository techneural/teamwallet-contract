import * as anchor from "@coral-xyz/anchor";
import { PublicKey, Keypair, LAMPORTS_PER_SOL, SystemProgram, Transaction } from "@solana/web3.js";
import {
  TOKEN_2022_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  createMint,
  mintTo,
  getAssociatedTokenAddressSync,
  createAssociatedTokenAccountInstruction,
  getAccount,
} from "@solana/spl-token";

/**
 * Fund an account by transferring SOL from the provider wallet
 * (Use this instead of airdrop on devnet to avoid rate limits)
 */
export async function fundAccount(
  provider: anchor.AnchorProvider,
  destination: PublicKey,
  amount: number = LAMPORTS_PER_SOL
): Promise<void> {
  const tx = new Transaction().add(
    SystemProgram.transfer({
      fromPubkey: provider.wallet.publicKey,
      toPubkey: destination,
      lamports: amount,
    })
  );
  await provider.sendAndConfirm(tx);
}

/**
 * Fund multiple accounts in a single transaction
 */
export async function fundAccounts(
  provider: anchor.AnchorProvider,
  destinations: PublicKey[],
  amount: number = LAMPORTS_PER_SOL
): Promise<void> {
  const tx = new Transaction();
  for (const dest of destinations) {
    tx.add(
      SystemProgram.transfer({
        fromPubkey: provider.wallet.publicKey,
        toPubkey: dest,
        lamports: amount,
      })
    );
  }
  await provider.sendAndConfirm(tx);
}

export function deriveTeamWalletPda(
  programId: PublicKey,
  owner: PublicKey,
  name: string
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("team_wallet"), owner.toBuffer(), Buffer.from(name)],
    programId
  );
}

export function deriveProposalPda(
  programId: PublicKey,
  teamWallet: PublicKey,
  nonce: PublicKey
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("proposal"), teamWallet.toBuffer(), nonce.toBuffer()],
    programId
  );
}

export async function createTestMint(
  provider: anchor.AnchorProvider,
  mintAuthority: Keypair,
  decimals: number = 9
): Promise<PublicKey> {
  return await createMint(
    provider.connection,
    mintAuthority,
    mintAuthority.publicKey,
    mintAuthority.publicKey,
    decimals,
    undefined,
    undefined,
    TOKEN_PROGRAM_ID
  );
}

export async function mintTokens(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  destination: PublicKey,
  authority: Keypair,
  amount: number,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID
): Promise<void> {
  await mintTo(
    provider.connection,
    authority,
    mint,
    destination,
    authority,
    amount,
    [],
    undefined,
    tokenProgram
  );
}

export async function createATA(
  provider: anchor.AnchorProvider,
  mint: PublicKey,
  owner: PublicKey,
  payer: Keypair,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID
): Promise<PublicKey> {
  const ata = getAssociatedTokenAddressSync(mint, owner, true, tokenProgram);

  const ix = createAssociatedTokenAccountInstruction(
    payer.publicKey,
    ata,
    owner,
    mint,
    tokenProgram
  );

  const tx = new anchor.web3.Transaction().add(ix);
  await provider.sendAndConfirm(tx, [payer]);

  return ata;
}

export async function getTokenBalance(
  provider: anchor.AnchorProvider,
  tokenAccount: PublicKey,
  tokenProgram: PublicKey = TOKEN_PROGRAM_ID
): Promise<bigint> {
  const account = await getAccount(
    provider.connection,
    tokenAccount,
    undefined,
    tokenProgram
  );
  return account.amount;
}

export async function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export function generateNonce(): Keypair {
  return Keypair.generate();
}

// Action builders for cleaner test code
export const Actions = {
  transferSol: (amount: anchor.BN, recipient: PublicKey) => ({
    transferSol: { amount, recipient },
  }),

  transferToken: (amount: anchor.BN, recipient: PublicKey, mint: PublicKey) => ({
    transferToken: { amount, recipient, mint },
  }),

  swap: (
    inputMint: PublicKey,
    outputMint: PublicKey,
    amountIn: anchor.BN,
    minAmountOut: anchor.BN,
    slippageBps: number
  ) => ({
    swap: { inputMint, outputMint, amountIn, minAmountOut, slippageBps },
  }),

  changeThreshold: (newThreshold: number) => ({
    changeThreshold: { newThreshold },
  }),

  addVoter: (voter: PublicKey) => ({
    addVoter: { voter },
  }),

  removeVoter: (voter: PublicKey) => ({
    removeVoter: { voter },
  }),

  addContributor: (contributor: PublicKey) => ({
    addContributor: { contributor },
  }),

  removeContributor: (contributor: PublicKey) => ({
    removeContributor: { contributor },
  }),

  tokenMint: (mint: PublicKey, amount: anchor.BN, recipient: PublicKey) => ({
    tokenMint: { mint, amount, recipient },
  }),

  tokenBurn: (mint: PublicKey, amount: anchor.BN) => ({
    tokenBurn: { mint, amount },
  }),

  tokenFreeze: (mint: PublicKey, account: PublicKey) => ({
    tokenFreeze: { mint, account },
  }),

  tokenThaw: (mint: PublicKey, account: PublicKey) => ({
    tokenThaw: { mint, account },
  }),

  tokenSetMintAuthority: (mint: PublicKey, newAuthority: PublicKey | null) => ({
    tokenSetMintAuthority: { mint, newAuthority },
  }),

  tokenSetFreezeAuthority: (mint: PublicKey, newAuthority: PublicKey | null) => ({
    tokenSetFreezeAuthority: { mint, newAuthority },
  }),
};
