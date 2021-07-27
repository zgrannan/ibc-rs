use flex_error::{define_error, TraceError};
use prost::DecodeError;

define_error! {
    #[derive(Clone)]
    Error {
        InvalidRawMerkleProof
            [ TraceError<DecodeError> ]
            |_| { "invalid raw merkle proof" },

        CommitmentProofDecodingFailed
            [ TraceError<DecodeError> ]
            |_| { "failed to decode commitment proof" },
    }
}
