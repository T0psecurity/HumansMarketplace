use cosmwasm_std::StdError;
use cw_utils::PaymentError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid roylties")]
    InvalidRoyalties {},

    #[error("No roylities exist for token_id")]
    NoRoyaltiesForTokenId {},

    #[error("Funds sent don't match bid amount")]
    IncorrectBidFunds {},

    #[error("InvalidExpiration")]
    InvalidExpiration {},

    #[error("AskExpired")]
    AskExpired {},

    #[error("BidExpired")]
    BidExpired {},

    #[error("Bid not found")]
    BidNotFound {},

    #[error("Contract needs approval")]
    NeedsApproval {},

    #[error("{0}")]
    BidPaymentError(#[from] PaymentError),
}
