use anchor_lang::prelude::*;

use crate::ProgramSystemAccount;

use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    /// CHECK: sign to send token or SOL
    #[account(seeds = [b"program_signer"], bump)]
    pub program_signer: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::mint = token_mint,
        associated_token::authority = program_signer,
    )]
    pub vault_token: Account<'info, TokenAccount>,

    /// CHECK: receive SOL
    #[account(
        mut,
        seeds = [b"curve_config", token_mint.key().as_ref()],
        bump,
    )]
    pub vault_sol: UncheckedAccount<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        seeds = [b"program_system_account"],
        bump,
        owner = crate::ID,
    )]
    pub program_system_account: Account<'info, ProgramSystemAccount>,

    #[account(
        mut,
        constraint = migration.key() == program_system_account.migration_account
    )]
    pub migration: Signer<'info>,

    #[account(
        init_if_needed,
        payer = migration,
        associated_token::mint = token_mint,
        associated_token::authority = migration,
        constraint = migration.key() == migration_ata.owner,
    )]
    pub migration_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
