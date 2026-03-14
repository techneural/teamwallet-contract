use anchor_lang::prelude::*;

#[account]
pub struct UpgradeProposal {
    pub team_wallet: Pubkey,       // 32
    pub proposer: Pubkey,          // 32
    pub new_buffer: Pubkey,        // 32
    pub spill_account: Pubkey,     // 32  ← SOL refund destination on execute/delete
    pub votes_for: u8,             // 1
    pub votes_against: u8,         // 1
    pub voters_voted: Vec<Pubkey>, // 4 + (32 * 15) = 484
    pub executed: bool,            // 1
    pub bump: u8,                  // 1
}

impl UpgradeProposal {
    // discriminator(8) + team_wallet(32) + proposer(32) + new_buffer(32)
    // + spill_account(32) + votes_for(1) + votes_against(1)
    // + voters_voted vec (4 prefix + 32*15 max voters)
    // + executed(1) + bump(1)
    pub const MAX_SIZE: usize = 8 + 32 + 32 + 32 + 32 + 1 + 1 + (4 + 32 * 15) + 1 + 1;
}