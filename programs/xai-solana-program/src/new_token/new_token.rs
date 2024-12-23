use anchor_lang::prelude::Accounts;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    metadata::{
        create_metadata_accounts_v3, mpl_token_metadata::types::DataV2, CreateMetadataAccountsV3,
        Metadata as Metaplex,
    },
    token::{self, mint_to, Mint, Token, TokenAccount},
};
use spl_token::instruction::AuthorityType;

use crate::instructions::*;
use serde::Serialize;

pub fn init_token(ctx: &Context<InitToken>, metadata: InitTokenParams) -> Result<()> {
    let identifier_account = ctx.accounts.identifier_account.key();
    let mint_seeds = &[
        "mint".as_bytes(),
        identifier_account.as_ref(),
        &[ctx.bumps.mint]
    ];
    let program_signer_seeds = &[
        "program_signer".as_bytes(),
        &[ctx.bumps.program_signer]
    ];
    let signer = [&mint_seeds[..], &program_signer_seeds[..]];

    let token_data: DataV2 = DataV2 {
        name                   : metadata.name,
        symbol                 : metadata.symbol,
        uri                    : metadata.uri,
        seller_fee_basis_points: 0,
        creators               : None,
        collection             : None,
        uses                   : None,
    };

    let metadata_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_metadata_program.to_account_info(),
        CreateMetadataAccountsV3 {
            payer           : ctx.accounts.user.to_account_info(),
            update_authority: ctx.accounts.program_signer.to_account_info(),
            mint            : ctx.accounts.mint.to_account_info(),
            metadata        : ctx.accounts.metadata.to_account_info(),
            mint_authority  : ctx.accounts.mint.to_account_info(),
            system_program  : ctx.accounts.system_program.to_account_info(),
            rent            : ctx.accounts.rent.to_account_info(),
        },
        &signer,
    );

    create_metadata_accounts_v3(metadata_ctx, token_data, false, true, None)?;

    msg!("Token mint created successfully.");

    Ok(())
}

pub fn mint_tokens(ctx: &Context<InitToken>, quantity: u64) -> Result<()> {
    let identifier_account = ctx.accounts.identifier_account.key();
    let mint_seeds = &[
        "mint".as_bytes(),
        identifier_account.as_ref(),
        &[ctx.bumps.mint]
    ];
    let signer = [&mint_seeds[..]];

    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::MintTo {
                authority: ctx.accounts.mint.to_account_info(),
                to       : ctx.accounts.vault.to_account_info(),
                mint     : ctx.accounts.mint.to_account_info(),
            },
            &signer,
        ),
        quantity,
    )?;

    Ok(())
}

pub fn drop_mint_authority(ctx: &Context<InitToken>, identifier: &String) -> Result<()> {
    let mint_seeds = &[
        "mint".as_bytes(),
        identifier.as_bytes(),
        &[ctx.bumps.mint]
    ];
    let signer = [&mint_seeds[..]];

    token::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::SetAuthority {
                current_authority: ctx.accounts.mint.to_account_info(),
                account_or_mint  : ctx.accounts.mint.to_account_info(),
            },
            &signer,
        ),
        AuthorityType::MintTokens,
        None
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    params: InitTokenParams,
)]
pub struct InitCreateTokenAccount<'info> {
    /// CHECK: Safe.
    pub identifier_account: AccountInfo<'info>,

    #[account(
        init,
        seeds = [b"mint", identifier_account.key().as_ref()],
        bump,
        payer = user,
        mint::decimals = 9,
        mint::authority = mint,
    )]
    /// CHECK: This is a new token mint
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        seeds = [b"curve_config", mint.key().as_ref()],
        bump,
        payer = user,
        space = ANCHOR_DISCRIMINATOR + CurveConfig::INIT_SPACE,
    )]
    pub curve_config: Account<'info, CurveConfig>,

    /// CHECK: sign to send token or SOL
    #[account(seeds = [b"program_signer"], bump)]
    pub program_signer: AccountInfo<'info>,

    #[account(
        init,
        payer = user,
        associated_token::mint = mint,
        associated_token::authority = program_signer,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user: Signer<'info>,
    // pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>   
}

#[derive(Accounts)]
#[instruction(
    params: InitTokenParams,
)]
pub struct InitToken<'info> {
    /// CHECK: New Metaplex Account being created
    #[account(mut)]
    pub metadata: UncheckedAccount<'info>,

    /// CHECK: Safe.
    pub identifier_account: AccountInfo<'info>,

    #[account(
        seeds = [b"mint", identifier_account.key().as_ref()],
        bump,
    )]
    /// CHECK: This is a new token mint
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"curve_config", mint.key().as_ref()],
        bump,
    )]
    pub curve_config: Account<'info, CurveConfig>,

    /// CHECK: sign to send token or SOL
    #[account(seeds = [b"program_signer"], bump)]
    pub program_signer: AccountInfo<'info>,

    #[account(
        seeds = [b"fee_config"],
        bump,
    )]
    pub fee_config: Account<'info, FeeConfig>,

    /// CHECK: safe. only used to receive SOL
    #[account(
        mut,
        constraint = fee_receiver_account.key() == fee_config.fee_receiver_account
    )]
    pub fee_receiver_account: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = program_signer,
    )]
    pub vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [b"init_token_config"],
        bump,
    )]
    pub init_token_config: Account<'info, InitTokenConfig>,

    #[account(mut)]
    pub user: Signer<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub token_metadata_program: Program<'info, Metaplex>,
    pub associated_token_program: Program<'info, AssociatedToken>   
}

// 5. Define the init token params
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone)]
pub struct InitTokenParams {
    pub name    : String,
    pub symbol  : String,
    pub uri     : String,
    pub decimals: u8,
}

#[derive(Debug, Serialize)]
pub struct CreateTokenEvent {
    pub id  : String,   // identifier
    pub u   : String,   // user
    pub mint: String,   // token_mint
    pub aa  : u64,      // airdrop_amount
    pub ms  : u128,     // max_supply
    pub ts  : u64,      // total_supply
    pub rs  : u128,     // init_virtual_sol_reserve
    pub rt  : u128,     // init_virtual_token_reserve
}