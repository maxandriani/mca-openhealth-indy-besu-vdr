// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use crate::{types::ContractOutput, ContractParam, VdrError, VdrResult};
use once_cell::sync::Lazy;
use regex_lite::Regex;
use serde_derive::{Deserialize, Serialize};

pub const DID_PREFIX: &str = "did";

// Indy DID method (incl. Besu-based Indy2) and ethr DID method.
//
// Supported identifiers:
// - Ethereum address: 20 bytes => 40 hex chars
// - Ethereum public key (common encodings):
//   - compressed: 33 bytes => 66 hex chars
//   - uncompressed (no 0x04 prefix): 64 bytes => 128 hex chars
//   - uncompressed (with 0x04 prefix): 65 bytes => 130 hex chars
//
// Note: this regex validates syntax only; the library may still require an address
// identifier in contexts where an on-chain account address is expected.
const DID_SYNTAX: &str =
    r"did:(?:indy:besu|indybesu|ethr):(?:[a-zA-Z0-9]+:)*0x(?:[a-fA-F0-9]{40}|[a-fA-F0-9]{66}|[a-fA-F0-9]{128}|[a-fA-F0-9]{130})";
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

        // Legacy: did:indybesu(:network)?:<id>
        if method == "indybesu" {
            let network = if method_specific.is_empty() {
                None
            } else {
                Some(method_specific.join(":"))
            };
            return Ok(ParsedDid {
                method: method.to_string(),
                network,
                identifier: identifier.to_string(),
            });
        }

        // New: did:indy:besu(:network)?:<id>
        if method == "indy" {
            if method_specific.first().copied() != Some("besu") {
                return Err(VdrError::ContractInvalidInputData);
            }

            let network_parts = &method_specific[1..];
            let network = if network_parts.is_empty() {
                None
            } else {
                Some(network_parts.join(":"))
            };

            return Ok(ParsedDid {
                method: "indy:besu".to_string(),
                network,
                identifier: identifier.to_string(),
            });
        }

        // Generic DID shape: did:<method>(:<network...>)?:<id>
        let network = if method_specific.is_empty() {
            None
        } else {
            Some(method_specific.join(":"))
        };

        Ok(ParsedDid {
            method: method.to_string(),
            network,
            identifier: identifier.to_string(),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_legacy_indybesu_did() {
        let did = DID::from("did:indybesu:testnet:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5");
        let parsed = ParsedDid::try_from(&did).unwrap();
        assert_eq!(parsed.method, "indybesu");
        assert_eq!(parsed.network.as_deref(), Some("testnet"));
        assert_eq!(
            parsed.identifier,
            "0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
        );
        assert_eq!(
            parsed.as_short_did().as_ref(),
            "did:indybesu:0xf0e2db6c8dc6c681bb5d6ad121a107f300e9b2b5"
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
