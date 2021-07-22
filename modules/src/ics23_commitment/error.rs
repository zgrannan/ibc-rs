use prost::DecodeError;
use thiserror::Error;
use prusti_contracts::*;

impl std::fmt::Display for Error {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}
impl std::fmt::Debug for Error {
#[trusted]
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}


#[derive(Clone, Error)]
pub enum Error {
//     #[error("invalid raw merkle proof")]
    InvalidRawMerkleProof(DecodeError),

//     #[error("failed to decode commitment proof")]
    CommitmentProofDecodingFailed(DecodeError),
}
