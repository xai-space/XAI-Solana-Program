use anchor_lang::prelude::*;
use crate::fee::FeeRecommendReward;

pub const ANCHOR_DISCRIMINATOR: usize = 8;

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct ProgramSystemAccount {
    pub owner            : Pubkey,
    pub admin            : Pubkey,
    pub migration_account: Pubkey,
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct FeeConfig {
    pub trade_fee_numerator  : u64,
    pub trade_fee_denominator: u64,
    pub creation_fee         : u64,
    pub fee_receiver_account : Pubkey,
    pub recommend_award_list : [u16; 5],
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct InitTokenConfig {
    pub init_virtual_token_reserve: u128,
    pub init_virtual_sol_reserve  : u128,
    pub mint_amount               : u64,
    pub token_total_supply        : u128,
    pub token_max_supply          : u128,
    pub sol_aim                   : u128,
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct CurveConfig {
    pub virtual_token_reserve: u128,
    pub virtual_sol_reserve  : u128,
    pub token_reserve        : u128,
    pub token_max_supply     : u128,
    pub sol_reserve          : u128,
    pub sol_aim              : u128,
    pub k                    : u128,
    pub graduated            : bool,
}

///   Initialize
#[derive(Accounts)]
pub struct ProgramInitialize1<'info> {
    /// CHECK: safe.
    #[account(
        init, 
        payer = owner,
        space = ANCHOR_DISCRIMINATOR + ProgramSystemAccount::INIT_SPACE,
        seeds = [b"program_system_account"],
        bump
    )]
    pub program_system_account: Account<'info, ProgramSystemAccount>,

    /// CHECK: use to sign the transfer of token and SOL 
    #[account(
        init, 
        payer = owner, 
        space = 0,
        seeds = [b"program_signer"],
        bump
    )]
    pub program_signer: AccountInfo<'info>,

    #[account(
        init,
        seeds = [b"fee_config"],
        bump,
        payer = owner,
        space = ANCHOR_DISCRIMINATOR + FeeConfig::INIT_SPACE,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// CHECK: safe. Read only.
    pub migration_account: AccountInfo<'info>,

    /// CHECK: use to receive fee
    pub fee_receiver_account: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

///   Initialize
#[derive(Accounts)]
pub struct ProgramInitialize2<'info> {
    #[account(
        init,
        seeds = [b"init_token_config"],
        bump,
        payer = owner,
        space = ANCHOR_DISCRIMINATOR + InitTokenConfig::INIT_SPACE,
    )]
    pub init_token_config: Account<'info, InitTokenConfig>,

    #[account(
        init,
        seeds = [b"program_config"],
        bump,
        payer = owner,
        space = ANCHOR_DISCRIMINATOR + ProgramConfig::INIT_SPACE,
    )]
    pub program_config: Account<'info, ProgramConfig>,

    /// CHECK: only used to receive SOL
    #[account(
        init,
        seeds = [b"recommend_reward_vault"],
        bump,
        payer = owner,
        space = 0,
    )]
    pub recommend_reward_vault: AccountInfo<'info>,

    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeFeeRecommendReward<'info> {
    #[account(
        init_if_needed,
        seeds = [b"fee_recommend_reward", recommender.key().as_ref()],
        bump,
        payer = user,
        space = ANCHOR_DISCRIMINATOR + FeeRecommendReward::INIT_SPACE,
    )]
    pub fee_recommend_reward: Account<'info, FeeRecommendReward>,

    /// CHECK: Only used in seeds
    pub recommender: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct ProgramConfig {
    pub version: [u8; 8],
}

/// Set owner
#[derive(Accounts)]
pub struct SetOwner<'info> {
    #[account(
        mut,
        seeds = [b"program_system_account"],
        bump,
        owner = crate::ID,
    )]
    pub program_system_account: Account<'info, ProgramSystemAccount>,

    /// CHECK: safe. Read only.
    pub new_owner: AccountInfo<'info>,

    #[account(
        mut,
        constraint = owner.key() == program_system_account.owner
    )]
    pub owner: Signer<'info>,
}

/// Set admin
#[derive(Accounts)]
pub struct SetAdmin<'info> {
    #[account(
        mut,
        seeds = [b"program_system_account"],
        bump,
        owner = crate::ID,
    )]
    pub program_system_account: Account<'info, ProgramSystemAccount>,

    /// CHECK: safe. Read only.
    pub new_admin: AccountInfo<'info>,

    #[account(
        mut,
        constraint = owner.key() == program_system_account.owner
    )]
    pub owner: Signer<'info>,
}

/// Set fee
#[derive(Accounts)]
pub struct SetFeeReceiverAccount<'info> {
    #[account(
        mut,
        seeds = [b"program_system_account"],
        bump,
        owner = crate::ID,
    )]
    pub program_system_account: Account<'info, ProgramSystemAccount>,

    #[account(
        mut,
        seeds = [b"fee_config"],
        bump,
        owner = crate::ID,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// CHECK: safe. Read only.
    pub new_fee_receiver_account: AccountInfo<'info>,

    #[account(
        mut,
        constraint = owner.key() == program_system_account.owner
    )]
    pub owner: Signer<'info>,
}

