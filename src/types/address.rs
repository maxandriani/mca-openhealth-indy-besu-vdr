// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{
    error::VdrError,
    types::{ContractOutput, ContractParam},
    DID,
};

use crate::contracts::types::did::ParsedDid;
use ethereum_types::Address as Address_;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

const PREFIX: &str = "0x";
const NULL_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Wrapper structure for Ethereum account address
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address(String);

impl Address {
    pub(crate) fn to_filter(&self) -> String {
        format!("000000000000000000000000{}", &self.0[2..])
    }

    pub fn as_blockchain_id(&self, chain_id: u64) -> String {
        format!("eip155:{}:{}", chain_id, self.as_ref())
    }

    pub fn null() -> Address {
        Address::from(NULL_ADDRESS)
    }

    pub fn is_null(&self) -> bool {
        self.as_ref() == NULL_ADDRESS
    }
}

impl From<&str> for Address {
    fn from(address: &str) -> Self {
        if address.starts_with(PREFIX) {
            Address(address.to_string())
        } else {
            Address(format!("{}{}", PREFIX, address))
        }
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl TryInto<ContractParam> for &Address {
    type Error = VdrError;

    fn try_into(self) -> Result<ContractParam, Self::Error> {
        let acc_address = Address_::from_str(self.as_ref()).map_err(|err| {
            VdrError::CommonInvalidData(format!(
                "Unable to parse account address. Err: {:?}",
                err.to_string()
            ))
        })?;
        Ok(ContractParam::Address(acc_address))
    }
}

impl TryFrom<ContractOutput> for Address {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        value.get_address(0)
    }
}

impl TryFrom<&DID> for Address {
    type Error = VdrError;

    /// Extract the on-chain account address from a DID identifier.
    ///
    /// Mirrors `DidUtils.convertEthereumIdentifierToAddress` from `indy-besu-contracts`:
    /// - 20-byte identifier: used as the address directly
    /// - 64-byte uncompressed public key: `keccak256(key)[12..]`
    /// - 65-byte uncompressed public key (`0x04` prefix): `keccak256(key[1..])[12..]`
    fn try_from(did: &DID) -> Result<Self, Self::Error> {
        use sha3::Digest;

        let parsed_did = ParsedDid::try_from(did)?;
        let identifier = parsed_did.identifier.as_str();

        let hex_part = identifier.strip_prefix(PREFIX).ok_or_else(|| {
            VdrError::CommonInvalidData(format!("Invalid DID identifier: {}", identifier))
        })?;
        let bytes = hex::decode(hex_part).map_err(|_| {
            VdrError::CommonInvalidData(format!("Invalid DID identifier: {}", identifier))
        })?;

        match bytes.len() {
            20 => Ok(Address::from(identifier)),
            64 => {
                let hash = sha3::Keccak256::digest(&bytes);
                Ok(Address::from(hex::encode(&hash[12..]).as_str()))
            }
            65 if bytes[0] == 0x04 => {
                let hash = sha3::Keccak256::digest(&bytes[1..]);
                Ok(Address::from(hex::encode(&hash[12..]).as_str()))
            }
            _ => Err(VdrError::CommonInvalidData(format!(
                "Invalid DID identifier: {}",
                identifier
            ))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const IDENTITY: &str = "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5";
    // Well-known secp256k1 test vector (private key = 1):
    // uncompressed public key and its derived Ethereum address
    const PUBLIC_KEY_NO_PREFIX: &str = "0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8";
    const PUBLIC_KEY_WITH_PREFIX: &str = "0x0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8";
    const DERIVED_ADDRESS: &str = "0x7e5f4552091a69125d5dfcb7b8c2659029395bdf";

    #[test]
    fn address_from_did_with_address_identifier() {
        let did = DID::from(format!("did:indy:besu:{}", IDENTITY).as_str());
        assert_eq!(Address::try_from(&did).unwrap().as_ref(), IDENTITY);

        let did = DID::from(format!("did:ethr:testnet:{}", IDENTITY).as_str());
        assert_eq!(Address::try_from(&did).unwrap().as_ref(), IDENTITY);
    }

    // Mirrors on-chain `DidUtils.convertEthereumIdentifierToAddress`
    #[test]
    fn address_from_did_with_public_key_identifier() {
        let did = DID::from(format!("did:indy:besu:{}", PUBLIC_KEY_NO_PREFIX).as_str());
        assert_eq!(Address::try_from(&did).unwrap().as_ref(), DERIVED_ADDRESS);

        let did = DID::from(format!("did:indy:besu:{}", PUBLIC_KEY_WITH_PREFIX).as_str());
        assert_eq!(Address::try_from(&did).unwrap().as_ref(), DERIVED_ADDRESS);
    }

    #[test]
    fn address_from_did_with_invalid_identifier_fails() {
        // 33-byte compressed public key is not supported by the contracts
        let did = DID::from(
            "did:indy:besu:0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        );
        Address::try_from(&did).unwrap_err();
    }
}
