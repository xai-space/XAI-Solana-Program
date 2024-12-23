use anchor_lang::prelude::*;
use solana_program::system_instruction;

#[derive(Clone, Debug, Default)]
pub struct Sol;

impl Sol {
    pub fn transfer_from<'info>(
        &self,
        from          : &Signer<'info>,
        to            : &AccountInfo<'info>,
        lamports      : u64,
        system_program: &Program<'info, System>,
    ) -> Result<bool> {
        let ix = system_instruction::transfer(
            from.key, 
            to.key, 
            lamports
        );
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                from.to_account_info(),
                to.to_account_info(),
                system_program.to_account_info(),
            ],
        )?;

        Ok(true)
    }

    pub fn transfer_from_pda<'info>(
        &self,
        from    : &AccountInfo<'info>,
        to      : &AccountInfo<'info>,
        lamports: u64,
    ) -> Result<bool> {
        **from.try_borrow_mut_lamports()? -= lamports;
        **to.try_borrow_mut_lamports()? += lamports;

        Ok(true)
    }
}
