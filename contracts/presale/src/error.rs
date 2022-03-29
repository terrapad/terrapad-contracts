use cosmwasm_std::StdError;
use hex::FromHexError;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Hex(#[from] FromHexError),

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("Invalid input")]
    InvalidInput {},

    #[error("Public Not In Progress")]
    PublicNotInProgress {},

    #[error("Private Not In Progress")]
    PrivateNotInProgress {},

    #[error("Still In Progress")]
    StillInProgress {},

    #[error("Not Whitelisted")]
    NotWhitelisted {},

    #[error("Exceed Allocation")]
    ExceedAllocation {},

    #[error("Wrong length")]
    WrongLength {},

    #[error("Verification failed")]
    VerificationFailed {},
}
