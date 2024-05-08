use anchor_lang::prelude::*;

/// Custom error codes
#[error_code]
pub enum ErrorCode {
    /// Initialize error
    #[msg("Error: Initialized already")]
    ErrorInitializedAready,
    /// Invalid timestamp
    #[msg("Error: Invalid timestamp")]
    ErrorInvalidTimestamp,
}