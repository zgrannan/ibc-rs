use std::convert::TryFrom;

use ics23::ProofSpec;
use serde::{Serialize, Serializer};

use ibc_proto::ics23::ProofSpec as ProtoProofSpec;

use crate::ics23_commitment::error::Error;

/// An array of proof specifications.
///
/// This type encapsulates different types of proof specifications, mostly predefined, e.g., for
/// Cosmos-SDK.
/// Additionally, this type also aids in the conversion from `ProofSpec` types from crate `ics23`
/// into proof specifications as represented in the `ibc_proto` type; see the
/// `From` trait(s) below.
#[derive(Clone, Debug, PartialEq)]
pub struct ProofSpecs {
    specs: Vec<ProofSpec>,
}

impl ProofSpecs {
    /// Returns the specification for Cosmos-SDK proofs
    pub fn cosmos() -> Self {
        Self {
            specs: vec![
                ics23::iavl_spec(),       // Format of proofs-iavl (iavl merkle proofs)
                ics23::tendermint_spec(), // Format of proofs-tendermint (crypto/ merkle SimpleProof)
            ],
        }
    }
}

/// Converts from the domain type (which is represented as a vector of `ics23::ProofSpec`
/// to the corresponding proto type (vector of `ibc_proto::ProofSpec`).
/// TODO: fix with https://github.com/informalsystems/ibc-rs/issues/853
impl From<ProofSpecs> for Vec<ProtoProofSpec> {
    fn from(domain_specs: ProofSpecs) -> Self {
        let mut raw_specs = vec![];
        for ds in domain_specs.specs.iter() {
            // Both `ProofSpec` types implement trait `prost::Message`. Convert by encoding, then
            // decoding into the destination type.
            // Safety note: the source and target data structures are identical, hence the
            // encode/decode conversion here should be infallible.
            let mut encoded = Vec::new();
            prost::Message::encode(ds, &mut encoded).unwrap();
            let decoded: ProtoProofSpec = prost::Message::decode(&*encoded).unwrap();
            raw_specs.push(decoded);
        }
        raw_specs
    }
}

impl TryFrom<Vec<ProtoProofSpec>> for ProofSpecs {
    type Error = Error;

    fn try_from(_proto_value: Vec<ProtoProofSpec>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl Eq for ProofSpecs {}

impl Serialize for ProofSpecs {
    fn serialize<S>(
        &self,
        _serializer: S,
    ) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        todo!()
    }
}
