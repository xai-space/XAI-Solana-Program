use anchor_lang::error_code;

#[error_code]
pub enum SwapError {
    #[msg("The output amount is less than the minimum required")]
    InsufficientOutputAmount,
    #[msg("This token have graduated")]
    TokenGraduated,
    #[msg("Invalid amount in")]
    InvalidAmountIn,
    #[msg("This token still swap in inner pool")]
    GraduateNotAllowed,
    #[msg("FeeRecommendReward PDA is not initialized")]
    FeeRecommendRewardUninitialized,
    #[msg("FeeRecommendReward PDA error")]
    FeeRecommendRewardError,
}

