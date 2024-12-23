use crate::{CurveConfig, FeeConfig};
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use serde::Serialize;

#[derive(Accounts)]
pub struct BuyToken<'info> {
    #[account(
        mut,
        seeds = [b"curve_config", token_mint.key().as_ref()],
        bump
    )]
    pub curve_config: Account<'info, CurveConfig>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = program_signer,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: receive SOL
    #[account(
        mut,
        seeds = [b"curve_config", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_sol: UncheckedAccount<'info>,

    /// CHECK: sign to send token or SOL
    #[account(seeds = [b"program_signer"], bump)]
    pub program_signer: UncheckedAccount<'info>,

    #[account(
        seeds = [b"fee_config"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// CHECK: only used to receive SOL
    #[account(
        mut,
        constraint = fee_receiver_account.key() == fee_config.fee_receiver_account
    )]
    pub fee_receiver_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,
    /// CHECK: This is the account only used to receive tokens
    pub receiver: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = user,
        associated_token::mint = token_mint,
        associated_token::authority = receiver,
    )]
    pub receiver_ata: Account<'info, TokenAccount>,

    /// CHECK: only used to receive SOL
    #[account(
        mut,
        seeds = [b"recommend_reward_vault"],
        bump,
    )]
    pub recommend_reward_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SellToken<'info> {
    #[account(
        mut,
        seeds = [b"curve_config", token_mint.key().as_ref()],
        bump
    )]
    pub curve_config: Account<'info, CurveConfig>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = program_signer,
    )]
    pub vault: Account<'info, TokenAccount>,

    /// CHECK: send SOL
    #[account(
        mut,
        seeds = [b"curve_config", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_sol: UncheckedAccount<'info>,

    /// CHECK: sign to send token or SOL
    #[account(seeds = [b"program_signer"], bump)]
    pub program_signer: UncheckedAccount<'info>,

    #[account(
        seeds = [b"fee_config"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// CHECK: only used to receive SOL
    #[account(
        mut,
        constraint = fee_receiver_account.key() == fee_config.fee_receiver_account
    )]
    pub fee_receiver_account: UncheckedAccount<'info>,

    #[account(
        mut,
        constraint = user.key() == user_token_ata.owner,
    )]
    pub user_token_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub token_mint: Account<'info, Mint>,

    /// CHECK: Safe. This account only used to receive SOL
    #[account(mut)]
    pub receiver: UncheckedAccount<'info>,

    /// CHECK: only used to receive SOL
    #[account(
        mut,
        seeds = [b"recommend_reward_vault"],
        bump,
    )]
    pub recommend_reward_vault: AccountInfo<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

#[derive(Debug, Serialize)]
pub struct BuyEvent {
    pub u   : String,   // user                     
    pub ua  : String,   // user's token account
    pub mint: String,   // token_mint               
    pub vsr : u128,     // virtual_sol_reserve  
    pub vtr : u128,     // virtual_token_reserve
    pub nvsr: u128,     // new_virtual_sol_reserve  
    pub nvtr: u128,     // new_virtual_token_reserve
    pub f   : u128,     // fee
    pub i   : u128,     // amount_in                
    pub o   : u128,     // amount_out               
    pub t   : u64,      // timestamp                
}

#[derive(Debug, Serialize)]
pub struct TokenGraduated {
    pub mint: String,
}

#[derive(Debug, Serialize)]
pub struct SellEvent {
    pub u   : String,   // user                     
    pub ua  : String,   // user's token account
    pub mint: String,   // token_mint               
    pub vsr : u128,     // virtual_sol_reserve  
    pub vtr : u128,     // virtual_token_reserve
    pub nvsr: u128,     // new_virtual_sol_reserve  
    pub nvtr: u128,     // new_virtual_token_reserve
    pub f   : u128,     // fee
    pub i   : u128,     // amount_in                
    pub o   : u128,     // amount_out               
    pub t   : u64,      // timestamp                
}