use prost::DecodeError;
use thiserror::Error;
use prusti_contracts::*;

impl std::fmt::Display for Error {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}
impl std::fmt::Debug for Error {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        panic!("No")
    }
}


#[derive(Clone, Error, PartialEq, Eq)]
pub enum Error {
//     #[error("invalid raw merkle proof")]
    InvalidRawMerkleProof(DecodeError),

//     #[error("failed to decode commitment proof")]
    CommitmentProofDecodingFailed(DecodeError),
}
