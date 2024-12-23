use anchor_lang::prelude::*;
use crate::SwapError;

#[derive(Clone, Debug, Default)]
pub struct Fees {
    /// Trade fee numerator
    pub trade_fee_numerator: u64,
    /// Trade fee denominator
    pub trade_fee_denominator: u64,
    pub fee_receiver_account: Pubkey,
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct FeeRecommendReward {
    pub unclaimed_sol: u64,
    pub total_reward : u64,
}

impl Fees {
    /// Calculate the trading fee in trading tokens
    pub fn calc_trading_fee(&self, trading_tokens: u128) -> Option<u128> {
        let token_amount    = trading_tokens;
        let fee_numerator   = u128::try_from(self.trade_fee_numerator).ok()?;
        let fee_denominator = u128::try_from(self.trade_fee_denominator).ok()?;

        if fee_numerator == 0 || fee_denominator == 0 || token_amount == 0 {
            Some(0)
        } else {
            token_amount.checked_mul(fee_numerator)?.checked_div(fee_denominator)
        }
    }

    pub fn calc_trading_fee_reverse(&self, trading_tokens: u128) -> Option<u128> {
        let token_amount    = trading_tokens;
        let fee_numerator   = u128::try_from(self.trade_fee_numerator).ok()?;
        let fee_denominator = u128::try_from(self.trade_fee_denominator).ok()?;

        if fee_numerator == 0 || fee_denominator == 0 || token_amount == 0 {
            Some(0)
        } else {
            token_amount.checked_mul(fee_denominator)?.checked_div(fee_denominator - fee_numerator)?.checked_sub(token_amount)
        }
    }

    pub fn reward_recommend(
        &self, 
        trading_fee: u64, 
        remaining_accounts: &[AccountInfo], 
        recommend_reward_list: &[u16; 5],
        program_id: &Pubkey,
    ) -> Result<(u64, u64)> {
        let mut total_reward_fee: u64 = 0;
        let recommend_len = remaining_accounts.len() / 2;

        let (recommend_accounts, recommend_pdas) = remaining_accounts.split_at(recommend_len);

        for i in 0..recommend_len {
            let recommend_account = &recommend_accounts[i];
            let recommend_account_pda = &recommend_pdas[i];

            let recommend_reward = *recommend_reward_list.get(i).unwrap();
            let reward_fee = trading_fee
                .checked_mul(u64::try_from(recommend_reward).ok().unwrap()).unwrap()
                .checked_div(10000).unwrap();

            total_reward_fee = total_reward_fee.checked_add(reward_fee).unwrap();

            let (expect_fee_recommend_reward_pda, _) = Pubkey::find_program_address(
                &[b"fee_recommend_reward", recommend_account.key().as_ref()],
                program_id,
            );

            require!(expect_fee_recommend_reward_pda.key() == recommend_account_pda.key(), SwapError::FeeRecommendRewardError);
            require!(recommend_account_pda.data_len() > 0, SwapError::FeeRecommendRewardUninitialized);
            
            let mut fee_recommend_reward_data = recommend_account_pda.try_borrow_mut_data()?;

            let fee_recommend_reward_data = &mut fee_recommend_reward_data[8..];
            
            let unclaimed_sol = u64::from_le_bytes(fee_recommend_reward_data[..8].try_into().unwrap());
            let total_reward = u64::from_le_bytes(fee_recommend_reward_data[8..16].try_into().unwrap());
            
            let new_unclaimed_sol = unclaimed_sol.checked_add(reward_fee).unwrap();
            let new_total_reward = total_reward.checked_add(reward_fee).unwrap();
            
            fee_recommend_reward_data[..8].copy_from_slice(&new_unclaimed_sol.to_le_bytes());
            fee_recommend_reward_data[8..16].copy_from_slice(&new_total_reward.to_le_bytes());
        }

        Ok((
            trading_fee.checked_sub(total_reward_fee).unwrap(),
            total_reward_fee,
        ))
    }

}

