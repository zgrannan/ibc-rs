use flex_error::{define_error, TraceError};
use prost::DecodeError;
use prusti_contracts::*;

define_error! {
    Error {
        InvalidRawMerkleProof
            [ TraceError<DecodeError> ]
            |_| { "invalid raw merkle proof" },

        CommitmentProofDecodingFailed
            [ TraceError<DecodeError> ]
            |_| { "failed to decode commitment proof" },
    }
}
