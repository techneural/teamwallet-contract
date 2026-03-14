use anchor_lang::prelude::*;

#[account]
pub struct DeleteProposal {
    pub team_wallet: Pubkey,       // 32
    pub proposer: Pubkey,          // 32
    pub program_id: Pubkey,        // 32  ← program to close
    pub spill_account: Pubkey,     // 32  ← receives all SOL after close
    pub votes_for: u8,             // 1
    pub votes_against: u8,         // 1
    pub voters_voted: Vec<Pubkey>, // 4 + (32 * 15)
    pub executed: bool,            // 1
    pub bump: u8,                  // 1
}

impl DeleteProposal {
    pub const MAX_SIZE: usize =
        8 + 32 + 32 + 32 + 32 + 1 + 1 + (4 + 32 * 15) + 1 + 1;
}