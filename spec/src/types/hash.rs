use std::{fmt, str::FromStr};

use generic_array::typenum::U32;
use serde::{Deserialize, Serialize};
use sha3::Digest;
use sha3::{
    digest::generic_array::{self, GenericArray},
    Sha3_256,
};
use thiserror::Error;

#[derive(Error, PartialEq, Eq, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum ConsensusHashError {
    #[error("Invalid format")]
    InvalidFormat,

    #[error("Invalid length")]
    InvalidLength,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct ConsensusHash([u8; 32]);

impl ConsensusHash {
    pub fn digest<T>(value: &T) -> Self
    where
        T: ?Sized + serde::Serialize,
    {
        let encoded: Vec<u8> = bincode::serialize(&value).unwrap();
        let sha3_256_digest: GenericArray<u8, U32> = Sha3_256::digest(encoded);
        ConsensusHash(sha3_256_digest.into())
    }

    pub fn leading_zeros(&self) -> u32 {
        let mut count = 0;
        for byte in self.0 {
            let byte_leading_zeros = byte.leading_zeros();
            count += byte_leading_zeros;
            if byte_leading_zeros < 8 {
                break;
            }
        }

        count
    }
}

impl TryFrom<Vec<u8>> for ConsensusHash {
    type Error = ConsensusHashError;

    fn try_from(vec: Vec<u8>) -> Result<Self, ConsensusHashError> {
        let slice = vec.as_slice();
        match slice.try_into() {
            Ok(byte_array) => Ok(ConsensusHash(byte_array)),
            Err(_) => Err(ConsensusHashError::InvalidLength),
        }
    }
}

impl TryFrom<String> for ConsensusHash {
    type Error = ConsensusHashError;

    fn try_from(s: String) -> Result<Self, ConsensusHashError> {
        match hex::decode(s) {
            Ok(decoded_vec) => decoded_vec.try_into(),
            Err(_) => Err(ConsensusHashError::InvalidFormat),
        }
    }
}

impl FromStr for ConsensusHash {
    type Err = ConsensusHashError;

    fn from_str(s: &str) -> Result<Self, ConsensusHashError> {
        ConsensusHash::try_from(s.to_string())
    }
}

impl From<ConsensusHash> for String {
    fn from(hash: ConsensusHash) -> Self {
        hash.to_string()
    }
}

impl fmt::Display for ConsensusHash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

pub trait ConsensusHashable {
    fn consensus_hash(&self) -> ConsensusHash;
}

impl<T: ?Sized + Serialize> ConsensusHashable for T {
    fn consensus_hash(&self) -> ConsensusHash {
        ConsensusHash::digest(self)
    }
}