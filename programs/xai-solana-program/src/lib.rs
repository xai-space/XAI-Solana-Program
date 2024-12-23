pub mod bonding_curve;
pub mod common;
pub mod error;
pub mod fee;
pub mod instructions;
pub mod new_token;
pub mod swap;

use crate::{
    fee::*,
    error::SwapError
};
use crate::common::{Sol, SplToken};
use anchor_lang::prelude::*;
use bonding_curve::*;
use instructions::*;
use new_token::*;
use serde_json::json;
use swap::*;

declare_id!("65tLehMbGRJUYJDNP5V2nCy3oVRBQW315gtLxuCSJ88b");

#[program]
pub mod contract_solana_xai {
    use super::*;

    pub fn initialize1(ctx: Context<ProgramInitialize1>) -> Result<()> {
        let trade_fee_numerator = 1;
        let trade_fee_denominator = 100;
        let fee_receiver_account = &ctx.accounts.fee_receiver_account;
        
        ctx.accounts.fee_config.trade_fee_numerator   = trade_fee_numerator;
        ctx.accounts.fee_config.trade_fee_denominator = trade_fee_denominator;
        ctx.accounts.fee_config.creation_fee          = 20000000; // 0.02 SOL
        ctx.accounts.fee_config.fee_receiver_account  = fee_receiver_account.key();
        ctx.accounts.fee_config.recommend_award_list  = [2000, 1000, 0 , 0, 0];

        ctx.accounts.program_system_account.owner = ctx.accounts.owner.key();
        ctx.accounts.program_system_account.admin = ctx.accounts.owner.key();
        ctx.accounts.program_system_account.migration_account = ctx.accounts.migration_account.key();
        
        Ok(())
    }

    pub fn initialize2(ctx: Context<ProgramInitialize2>) -> Result<()> {
        ctx.accounts.init_token_config.init_virtual_token_reserve = 1_038_000_000_000000000;
        ctx.accounts.init_token_config.init_virtual_sol_reserve   = 21_000000000;
        ctx.accounts.init_token_config.mint_amount                = 1_000_000_000_000000000;
        ctx.accounts.init_token_config.token_total_supply         = 1_000_000_000_000000000;
        ctx.accounts.init_token_config.token_max_supply           = 745_100_000_000000000;
        ctx.accounts.init_token_config.sol_aim                    = 53_420000000;

        let version_str = "0.1.0";

        ctx.accounts.program_config.version = std::array::from_fn(|i| {
            version_str.as_bytes().get(i).cloned().unwrap_or(b'\0')
        });

        Ok(())
    }
    
    pub fn init_create_token_account(_ctx: Context<InitCreateTokenAccount>, params: InitTokenParams) -> Result<()> {
        Ok(())
    }

    pub fn create_token(ctx: Context<InitToken>, params: InitTokenParams) -> Result<()> {
        let identifier = ctx.accounts.identifier_account.key().to_string();

        msg!("create_token identifier: {:?}", identifier);

        // send creation fee
        let sol = Sol;

        sol.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.fee_receiver_account,
            ctx.accounts.fee_config.creation_fee,
            &ctx.accounts.system_program
        )?;

        // create token
        let init_virtual_token_reserve: u128 = ctx.accounts.init_token_config.init_virtual_token_reserve;
        let init_virtual_sol_reserve   = ctx.accounts.init_token_config.init_virtual_sol_reserve;
        let mint_amount                = ctx.accounts.init_token_config.mint_amount;

        ctx.accounts.curve_config.virtual_token_reserve = init_virtual_token_reserve;
        ctx.accounts.curve_config.virtual_sol_reserve   = init_virtual_sol_reserve;
        ctx.accounts.curve_config.token_reserve         = ctx.accounts.init_token_config.token_max_supply;
        ctx.accounts.curve_config.token_max_supply      = ctx.accounts.init_token_config.token_max_supply;
        ctx.accounts.curve_config.sol_aim               = ctx.accounts.init_token_config.sol_aim;
        ctx.accounts.curve_config.k                     = init_virtual_token_reserve
            .checked_mul(init_virtual_sol_reserve)
            .unwrap();

        init_token(&ctx, params).unwrap();
        mint_tokens(&ctx, mint_amount).unwrap();

        // drop_mint_authority(&ctx, &identifier).unwrap();

        let create_token_event = CreateTokenEvent {
            id  : identifier.clone(),
            u   : ctx.accounts.user.key().to_string(),
            mint: ctx.accounts.mint.key().to_string(),
            aa  : 0,
            ms  : ctx.accounts.init_token_config.token_max_supply,
            ts  : mint_amount,
            rs  : ctx.accounts.init_token_config.init_virtual_sol_reserve,
            rt  : ctx.accounts.init_token_config.init_virtual_token_reserve,
        };

        msg!("$CreateTokenEvent: {}", json!(create_token_event));

        Ok(())
    }

    pub fn initialize_fee_recommend_reward(_ctx: Context<InitializeFeeRecommendReward>) -> Result<()> {
        Ok(())
    }

    pub fn buy(ctx: Context<BuyToken>, amount_in: u128, amount_out_min: u128) -> Result<()> {
        msg!("buy amount_in: {}", amount_in);
        require!(amount_in > 0, SwapError::InvalidAmountIn);

        let curve_config = &mut ctx.accounts.curve_config;
        require!(
            curve_config.token_reserve > 0 
            && curve_config.sol_aim > 0 
            && curve_config.graduated == false, 
            SwapError::TokenGraduated
        );

        let fees = Fees {
            trade_fee_numerator  : ctx.accounts.fee_config.trade_fee_numerator,
            trade_fee_denominator: ctx.accounts.fee_config.trade_fee_denominator,
            fee_receiver_account : ctx.accounts.fee_config.fee_receiver_account,
        };

        let trading_fee: u128;
        let amount_in_without_fee: u128;
        if amount_in > curve_config.sol_aim {
            trading_fee = fees.calc_trading_fee_reverse(curve_config.sol_aim).unwrap();
            amount_in_without_fee = curve_config.sol_aim;
        } else {
            trading_fee = fees.calc_trading_fee(amount_in).unwrap();
            amount_in_without_fee = amount_in.checked_sub(trading_fee).unwrap();
        }

        let (mut amount_out, new_virtual_sol_reserve, mut new_virtual_token_reserve) =
            calculator::buy(&curve_config, amount_in_without_fee)?;

        curve_config.sol_aim = curve_config.sol_aim.checked_sub(amount_in_without_fee).unwrap();
        curve_config.sol_reserve += amount_in_without_fee;

        let mut graduated = false;
        if curve_config.sol_aim == 0 {
            amount_out = curve_config.token_reserve;
            new_virtual_token_reserve = curve_config
                .virtual_token_reserve
                .checked_sub(amount_out)
                .unwrap();

            graduated = true;
        }
        msg!("buy amount_out: {:?}", amount_out);

        require!(amount_out >= amount_out_min, SwapError::InsufficientOutputAmount);

        let vsr = curve_config.virtual_sol_reserve;
        let vtr = curve_config.virtual_token_reserve;

        curve_config.virtual_sol_reserve    = new_virtual_sol_reserve;
        curve_config.virtual_token_reserve  = new_virtual_token_reserve;
        curve_config.token_reserve         -= amount_out;

        // send fee
        let sol = Sol;

        let (residue_fee, total_reward_fee) = fees.reward_recommend(
            u64::try_from(trading_fee).unwrap(),
            ctx.remaining_accounts,
            &ctx.accounts.fee_config.recommend_award_list,
            ctx.program_id,
        )?;
        
        sol.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.recommend_reward_vault,
            total_reward_fee,
            &ctx.accounts.system_program
        )?;

        sol.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.fee_receiver_account,
            residue_fee,
            &ctx.accounts.system_program
        )?;

        // receive SOL to vault_sol PDA
        sol.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.vault_sol,
            amount_in_without_fee.try_into().unwrap(),
            &ctx.accounts.system_program
        )?;
        
        msg!("receive SOL successfully.");

        // transfer token
        let spl_token = SplToken;

        let program_signer_seeds = &[
            "program_signer".as_bytes(),
            &[ctx.bumps.program_signer]
        ];

        spl_token.transfer_from_pda(
            program_signer_seeds,
            &ctx.accounts.vault.to_account_info(),
            &ctx.accounts.program_signer,
            &ctx.accounts.receiver_ata.to_account_info(),
            amount_out.try_into().unwrap(),
            &ctx.accounts.token_program,
        )?;

        let timestamp = Clock::get()?.unix_timestamp;
        let timestamp: u64 = timestamp.try_into().unwrap();

        msg!("$BuyEvent: {}", json!(BuyEvent {
            u   : ctx.accounts.receiver.key().to_string(),
            ua  : ctx.accounts.receiver_ata.key().to_string(),
            mint: ctx.accounts.token_mint.key().to_string(),
            vsr,
            vtr,
            nvsr: new_virtual_sol_reserve,
            nvtr: new_virtual_token_reserve,
            f   : trading_fee,
            i   : trading_fee.checked_add(amount_in_without_fee).unwrap(),
            o   : amount_out,
            t   : timestamp,
        }));

        if graduated {
            ctx.accounts.curve_config.graduated = true;
            msg!("$TokenGraduatedEvent: {}", json!(TokenGraduated {
                mint: ctx.accounts.token_mint.key().to_string(),
            }));

        }

        Ok(())
    }

    pub fn sell(ctx: Context<SellToken>, amount_in: u128, amount_out_min: u128) -> Result<()> {
        require!(amount_in > 0, SwapError::InvalidAmountIn);
        require!(ctx.accounts.curve_config.token_reserve > 0 && ctx.accounts.curve_config.graduated == false, SwapError::TokenGraduated);

        let (amount_out, new_virtual_sol_reserve, new_virtual_token_reserve) =
            calculator::sell(&ctx.accounts.curve_config, amount_in)?;
        msg!("sell amount_out: {:?}", amount_out);

        let fees = Fees {
            trade_fee_numerator  : ctx.accounts.fee_config.trade_fee_numerator,
            trade_fee_denominator: ctx.accounts.fee_config.trade_fee_denominator,
            fee_receiver_account : ctx.accounts.fee_config.fee_receiver_account,
        };

        let trading_fee = fees.calc_trading_fee(amount_out).unwrap();

        // receive token
        let spl_token = SplToken;
        spl_token.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.user_token_ata.to_account_info(),
            &ctx.accounts.vault.to_account_info(),
            amount_in.try_into().unwrap(),
            &ctx.accounts.token_program,
        )?;

        msg!("receive token successfully.");

        let sol = Sol;
        // send recommend reward
        let (residue_fee, total_reward_fee) = fees.reward_recommend(
            u64::try_from(trading_fee).unwrap(),
            ctx.remaining_accounts,
            &ctx.accounts.fee_config.recommend_award_list,
            ctx.program_id,
        )?;

        sol.transfer_from(
            &ctx.accounts.user,
            &ctx.accounts.recommend_reward_vault,
            total_reward_fee,
            &ctx.accounts.system_program
        )?;

        // send fee to developers
        sol.transfer_from_pda(
            &ctx.accounts.vault_sol,
            &ctx.accounts.fee_receiver_account,
            residue_fee,
        )?;

        // send the rest of SOL to user
        let amount_out_without_fee = amount_out.checked_sub(trading_fee).unwrap();
        require!(amount_out_without_fee >= amount_out_min, SwapError::InsufficientOutputAmount); 

        sol.transfer_from_pda(
            &ctx.accounts.vault_sol,
            &ctx.accounts.receiver,
            amount_out_without_fee.try_into().unwrap(),
        )?;

        let vsr = ctx.accounts.curve_config.virtual_sol_reserve;
        let vtr = ctx.accounts.curve_config.virtual_token_reserve;

        // update curve_config
        ctx.accounts.curve_config.virtual_sol_reserve    = new_virtual_sol_reserve;
        ctx.accounts.curve_config.virtual_token_reserve  = new_virtual_token_reserve;
        ctx.accounts.curve_config.token_reserve         += amount_in;
        ctx.accounts.curve_config.sol_reserve           -= amount_out;
        ctx.accounts.curve_config.sol_aim               += amount_out;

        let timestamp = Clock::get()?.unix_timestamp;
        let timestamp: u64 = timestamp.try_into().unwrap();

        msg!("$SellEvent: {}", json!(SellEvent {
            u   : ctx.accounts.user.key().to_string(),
            ua  : ctx.accounts.user_token_ata.key().to_string(),
            mint: ctx.accounts.token_mint.key().to_string(),
            vsr,
            vtr,
            nvsr: new_virtual_sol_reserve,
            nvtr: new_virtual_token_reserve,
            f   : trading_fee,
            i   : amount_in,
            o   : amount_out_without_fee,
            t   : timestamp,
        }));

        Ok(())
    }

    pub fn recommender_claim_sol(ctx: Context<RecommenderClaimSol>) -> Result<()> {
        let sol = Sol;

        let claim_amount = ctx.accounts.fee_recommend_reward.unclaimed_sol;
        ctx.accounts.fee_recommend_reward.unclaimed_sol = 0;

        sol.transfer_from_pda(
            &ctx.accounts.recommend_reward_vault,
            &ctx.accounts.recommender,
            claim_amount,
        )?;

        emit_cpi!(RecommenderClaimSolEvent {
            recommender: ctx.accounts.recommender.key(),
            claim_amount,
        });

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, withdraw_sol_amount: u64, withdraw_token_amount: u64) -> Result<()> {
        // withdraw SOL
        let sol = Sol;
        sol.transfer_from_pda(
            &ctx.accounts.vault_sol,
            &ctx.accounts.migration.to_account_info(),
            withdraw_sol_amount,
        )?;

        // withdraw token
        let spl_token = SplToken;

        let program_signer_seeds = &[
            "program_signer".as_bytes(),
            &[ctx.bumps.program_signer]
        ];

        spl_token.transfer_from_pda(
            program_signer_seeds,
            &ctx.accounts.vault_token.to_account_info(),
            &ctx.accounts.program_signer,
            &ctx.accounts.migration_ata.to_account_info(),
            withdraw_token_amount,
            &ctx.accounts.token_program,
        )?;

        Ok(())
    }

}
