// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{types::ContractOutput, ContractParam, VdrError, VdrResult};
use once_cell::sync::Lazy;
use regex_lite::Regex;
use serde_derive::{Deserialize, Serialize};

pub const DID_PREFIX: &str = "did";

// Indy-on-Besu DID method (`did:indy:besu`) and ethr DID method.
//
// Mirrors the on-chain `DidUtils.parseDid` implementation from `indy-besu-contracts`:
// - methods: `indy:besu` and `ethr` only
// - at most ONE optional network segment
// - supported identifiers (`DidUtils.convertEthereumIdentifierToAddress`):
//   - Ethereum address: 20 bytes => 40 hex chars
//   - uncompressed public key (no 0x04 prefix): 64 bytes => 128 hex chars
//   - uncompressed public key (with 0x04 prefix): 65 bytes => 130 hex chars
//
// Note: this regex validates syntax only; the library may still require an address
// identifier in contexts where an on-chain account address is expected.
const DID_SYNTAX: &str = r"did:(?:indy:besu|ethr):(?:[a-zA-Z0-9]+:)?0x(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{128}|[a-fA-F0-9]{130})";
const PATH: &str = r"\/[^#?]*";
const QUERY: &str = r"[?][^#]*";
const FRAGMENT: &str = r"[#].*";

static DID_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(&format!("^{DID_SYNTAX}$")).unwrap());

pub static DID_URL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(&format!(
        "^{DID_SYNTAX}(?:{PATH})?(?:{QUERY})?(?:{FRAGMENT})?$"
    ))
    .unwrap()
});

pub static RELATIVE_DID_URL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(&format!("^(?:{PATH})?(?:{QUERY})?(?:{FRAGMENT})?$")).unwrap());

/// Wrapper structure for DID
#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct DID(String);

impl DID {
    pub fn build(method: &str, network: Option<&str>, id: &str) -> DID {
        if let Some(network) = network {
            DID(format!("{}:{}:{}:{}", DID_PREFIX, method, network, id))
        } else {
            DID(format!("{}:{}:{}", DID_PREFIX, method, id))
        }
    }

    pub fn without_network(&self) -> VdrResult<DID> {
        Ok(ParsedDid::try_from(self)?.as_short_did())
    }

    pub(crate) fn validate(&self) -> VdrResult<()> {
        if !DID_REGEX.is_match(&self.0) {
            return Err(VdrError::InvalidDidDocument(format!(
                "Incorrect DID: {}",
                &self.0
            )));
        };

        Ok(())
    }
}

impl From<&str> for DID {
    fn from(did: &str) -> Self {
        DID(did.to_string())
    }
}

impl AsRef<str> for DID {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl ToString for DID {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl TryFrom<&DID> for ContractParam {
    type Error = VdrError;

    fn try_from(value: &DID) -> Result<Self, Self::Error> {
        Ok(ContractParam::String(value.to_string()))
    }
}

impl TryFrom<ContractOutput> for DID {
    type Error = VdrError;

    fn try_from(value: ContractOutput) -> Result<Self, Self::Error> {
        Ok(DID::from(value.get_string(0)?.as_str()))
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct ParsedDid {
    pub(crate) method: String,
    pub(crate) network: Option<String>,
    pub(crate) identifier: String,
}

impl ParsedDid {
    pub(crate) fn as_short_did(&self) -> DID {
        DID::from(format!("{}:{}:{}", DID_PREFIX, self.method, self.identifier).as_str())
    }
}

impl TryFrom<&DID> for ParsedDid {
    type Error = VdrError;

    fn try_from(did: &DID) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = did.as_ref().split(':').collect();
        if parts.len() < 3 {
            return Err(VdrError::ContractInvalidInputData);
        }
        if parts[0] != DID_PREFIX {
            return Err(VdrError::ContractInvalidInputData);
        }

        let method = parts[1];
        let identifier = parts
            .last()
            .copied()
            .ok_or(VdrError::ContractInvalidInputData)?;
        let method_specific: &[&str] = &parts[2..parts.len() - 1];

        // Indy-on-Besu: did:indy:besu(:network)?:<id>
        if method == "indy" {
            if method_specific.first().copied() != Some("besu") {
                return Err(VdrError::ContractInvalidInputData);
            }

            let network_parts = &method_specific[1..];
            // On-chain `DidUtils.parseDid` allows at most one network segment
            let network = match network_parts {
                [] => None,
                [network] => Some(network.to_string()),
                _ => return Err(VdrError::ContractInvalidInputData),
            };

            return Ok(ParsedDid {
                method: "indy:besu".to_string(),
                network,
                identifier: identifier.to_string(),
            });
        }

        // Ethr: did:ethr(:network)?:<id>
        if method == "ethr" {
            // On-chain `DidUtils.parseDid` allows at most one network segment
            let network = match method_specific {
                [] => None,
                [network] => Some(network.to_string()),
                _ => return Err(VdrError::ContractInvalidInputData),
            };

            return Ok(ParsedDid {
                method: method.to_string(),
                network,
                identifier: identifier.to_string(),
            });
        }

        // This VDR only deals with `did:indy:besu` and `did:ethr` DIDs
        Err(VdrError::ContractInvalidInputData)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_ethr_did() {
        let did = DID::from("did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        let parsed = ParsedDid::try_from(&did).unwrap();
        assert_eq!(parsed.method, "ethr");
        assert_eq!(parsed.network.as_deref(), Some("testnet"));
        assert_eq!(
            parsed.identifier,
            "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
        assert_eq!(
            parsed.as_short_did().as_ref(),
            "did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
    }

    #[test]
    fn parse_unsupported_method_fails() {
        // Legacy `did:indybesu` is no longer supported
        ParsedDid::try_from(&DID::from(
            "did:indybesu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5",
        ))
        .unwrap_err();
        ParsedDid::try_from(&DID::from(
            "did:sov:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5",
        ))
        .unwrap_err();
    }

    #[test]
    fn build_new_indy_besu_did() {
        assert_eq!(
            DID::build(
                "indy:besu",
                None,
                "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
            )
            .as_ref(),
            "did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
        assert_eq!(
            DID::build(
                "indy:besu",
                Some("testnet"),
                "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
            )
            .as_ref(),
            "did:indy:besu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
    }

    #[test]
    fn validate_did_formats() {
        // New format, with and without network
        DID::from("did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap();
        DID::from("did:indy:besu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap();
        // Legacy format is no longer accepted
        DID::from("did:indybesu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
        // Ethr method
        DID::from("did:ethr:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap();
        // Unknown method or malformed identifier must be rejected
        DID::from("did:unknown:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
        DID::from("did:indy:besu:123").validate().unwrap_err();
        // `did:indy` without the `besu` sub-method is not served by this VDR
        DID::from("did:indy:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
    }

    // Mirrors on-chain `DidUtils.parseDid`: at most ONE network segment is allowed
    #[test]
    fn multiple_network_segments_are_rejected() {
        DID::from("did:indy:besu:one:two:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
        ParsedDid::try_from(&DID::from(
            "did:indy:besu:one:two:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5",
        ))
        .unwrap_err();
        DID::from("did:ethr:one:two:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
        ParsedDid::try_from(&DID::from(
            "did:ethr:one:two:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5",
        ))
        .unwrap_err();
        // A nested DID used as identifier is also invalid
        DID::from("did:indy:besu:did:ethr:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5")
            .validate()
            .unwrap_err();
    }

    // Mirrors on-chain `DidUtils.convertEthereumIdentifierToAddress`:
    // 20-byte addresses and 64/65-byte uncompressed public keys are supported
    #[test]
    fn public_key_identifiers() {
        // 65-byte uncompressed public key (0x04 prefix) => 130 hex chars
        DID::from("did:indy:besu:0x0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
            .validate()
            .unwrap();
        // 64-byte uncompressed public key (no prefix) => 128 hex chars
        DID::from("did:indy:besu:0x79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8")
            .validate()
            .unwrap();
        // 33-byte compressed public key is NOT supported by the contracts => rejected
        DID::from(
            "did:indy:besu:0x0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
        )
        .validate()
        .unwrap_err();
    }

    #[test]
    fn without_network_new_indy_besu_did() {
        let did = DID::from("did:indy:besu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        assert_eq!(
            did.without_network().unwrap().as_ref(),
            "did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
        // Already-short DID stays unchanged
        let did = DID::from("did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        assert_eq!(
            did.without_network().unwrap().as_ref(),
            "did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
    }

    #[test]
    fn parse_new_indy_besu_did_without_network() {
        let did = DID::from("did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        let parsed = ParsedDid::try_from(&did).unwrap();
        assert_eq!(parsed.method, "indy:besu");
        assert_eq!(parsed.network, None);
        assert_eq!(
            parsed.identifier,
            "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
    }

    #[test]
    fn parse_new_indy_besu_did() {
        let did = DID::from("did:indy:besu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        let parsed = ParsedDid::try_from(&did).unwrap();
        assert_eq!(parsed.method, "indy:besu");
        assert_eq!(parsed.network.as_deref(), Some("testnet"));
        assert_eq!(
            parsed.identifier,
            "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
        assert_eq!(
            parsed.as_short_did().as_ref(),
            "did:indy:besu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
    }
}
