use anchor_lang::prelude::*;
use anchor_spl::token::{self, Transfer as SplTransfer};

#[derive(Clone, Debug, Default)]
pub struct SplToken;

impl SplToken {
    pub fn transfer_from<'info>(
        &self,
        from         : &Signer<'info>,
        from_ata     : &AccountInfo<'info>,
        to_ata       : &AccountInfo<'info>,
        amount       : u64,
        token_program: &AccountInfo<'info>,
    ) -> Result<bool> {
        let cpi_accounts = SplTransfer {
            from     : from_ata.to_account_info().clone(),
            to       : to_ata.to_account_info().clone(),
            authority: from.to_account_info().clone(),
        };

        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        Ok(true)
    }

    /// 
    /// let spl_token = SplToken {};
    /// 
    /// let program_signer_seeds = &[
    ///     "program_signer".as_bytes(),
    ///     &[ctx.bumps.program_signer]
    /// ];
    /// 
    /// spl_token.transfer_from_pda(
    ///     program_signer_seeds,
    ///     &ctx.accounts.vault.to_account_info(),
    ///     &ctx.accounts.program_signer,
    ///     &ctx.accounts.receiver_ata.to_account_info(),
    ///     amount_out.try_into().unwrap(),
    ///     &ctx.accounts.token_program,
    /// )?;
    /// 
    pub fn transfer_from_pda<'info>(
        &self,
        from_seeds_bump: &[&[u8]],
        from_ata       : &AccountInfo<'info>,
        authority      : &AccountInfo<'info>,
        to_ata         : &AccountInfo<'info>,
        amount         : u64,
        token_program  : &AccountInfo<'info>,
    ) -> Result<bool> {
        let signer_seeds = [&from_seeds_bump[..]];

        let cpi_accounts = SplTransfer {
            from     : from_ata.to_account_info().clone(),
            to       : to_ata.to_account_info().clone(),
            authority: authority.to_account_info().clone(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, &signer_seeds);
        token::transfer(cpi_ctx, amount)?;

        Ok(true)
    }
}
