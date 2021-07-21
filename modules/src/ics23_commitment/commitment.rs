use prusti_contracts::*;

use crate::ics23_commitment::error::Error;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use serde::{Deserialize, Serialize};
use std::{convert::TryFrom, fmt};
use subtle_encoding::{Encoding, Hex};

#[derive(Clone, PartialEq, Eq)]
// #[serde(transparent)]
pub struct CommitmentRoot {
//     #[serde(serialize_with = "crate::serializers::ser_hex_upper")]
    bytes: Vec<u8>,
}

impl fmt::Debug for CommitmentRoot {
#[trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex = Hex::upper_case().encode_to_string(&self.bytes).unwrap();
        f.debug_tuple("CommitmentRoot").field(&hex).finish()
    }
}

impl CommitmentRoot {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes),
        }
    }

#[trusted]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

impl From<Vec<u8>> for CommitmentRoot {
#[trusted]
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

#[derive(Clone, PartialEq)]
pub struct CommitmentPath;

#[derive(Clone, PartialEq, Eq)]
// #[serde(transparent)]
pub struct CommitmentProofBytes {
//     #[serde(serialize_with = "crate::serializers::ser_hex_upper")]
    bytes: Vec<u8>,
}

impl CommitmentProofBytes {
    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }
}

impl From<Vec<u8>> for CommitmentProofBytes {
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl From<CommitmentProofBytes> for Vec<u8> {
    fn from(p: CommitmentProofBytes) -> Vec<u8> {
        p.bytes
    }
}

// impl From<MerkleProof> for CommitmentProofBytes {
//     fn from(proof: MerkleProof) -> Self {
//         let raw_proof: RawMerkleProof = proof.into();
//         raw_proof.into()
//     }
// }

impl From<RawMerkleProof> for CommitmentProofBytes {
    #[trusted]
    fn from(_proof: RawMerkleProof) -> Self {
        // let mut buf = Vec::new();
        // prost::Message::encode(&proof, &mut buf).unwrap();
        // buf.into()
        panic!("no")
    }
}

impl TryFrom<CommitmentProofBytes> for RawMerkleProof {
    type Error = Error;

    #[trusted]
    fn try_from(_value: CommitmentProofBytes) -> Result<Self, Self::Error> {
        panic!("No")
        // let value: Vec<u8> = value.into();
        // let res: RawMerkleProof =
        //     prost::Message::decode(value.as_ref()).map_err(Error::InvalidRawMerkleProof)?;
        // Ok(res)
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Default)]
pub struct CommitmentPrefix {
    bytes: Vec<u8>,
}

impl CommitmentPrefix {
    pub fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            bytes: Vec::from(bytes),
        }
    }

#[trusted]
    pub fn is_empty(&self) -> bool {
        self.bytes.len() == 0
    }

#[trusted]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.bytes
    }
}

impl From<Vec<u8>> for CommitmentPrefix {
#[trusted]
    fn from(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }
}

impl fmt::Debug for CommitmentPrefix {
#[trusted]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let converted = std::str::from_utf8(self.as_bytes());
        match converted {
            Ok(s) => write!(f, "{}", s),
            Err(_e) => write!(f, "<not valid UTF8: {:?}>", self.as_bytes()),
        }
    }
}

impl Serialize for CommitmentPrefix {
#[trusted]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!("{:?}", self).serialize(serializer)
    }
}

#[cfg(test)]
pub mod test_util {
    use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;

    /// Returns a dummy `RawMerkleProof`, for testing only!
    pub fn get_dummy_merkle_proof() -> RawMerkleProof {
        let parsed = ibc_proto::ics23::CommitmentProof { proof: None };
        let mproofs: Vec<ibc_proto::ics23::CommitmentProof> = vec![parsed];
        RawMerkleProof { proofs: mproofs }
    }
}
