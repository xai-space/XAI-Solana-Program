use anchor_lang::prelude::*;
use crate::fee::FeeRecommendReward;

#[derive(Accounts)]
pub struct RecommenderClaimSol<'info> {
    #[account(
        mut,
        seeds = [b"fee_recommend_reward", recommender.key().as_ref()],
        bump,
    )]
    pub fee_recommend_reward: Account<'info, FeeRecommendReward>,

    /// CHECK: Send SOL to recommender
    #[account(
        mut,
        seeds = [b"recommend_reward_vault"],
        bump,
    )]
    pub recommend_reward_vault: AccountInfo<'info>,

    #[account(mut)]
    pub recommender: Signer<'info>,
    pub system_program: Program<'info, System>,
}
