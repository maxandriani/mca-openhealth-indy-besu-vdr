// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

//! TypeScript type definitions for the values crossing the JS <-> WASM boundary.
//!
//! The interfaces below mirror the serde (de)serialization of the corresponding
//! `indy-besu-vdr` Rust types, so the generated `.d.ts` package exposes fully
//! typed method inputs and outputs instead of `any`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPES: &'static str = r#"
/**
 * A DID handled by this VDR.
 *
 * Supported methods (per https://hyperledger-indy.github.io/indy-did-method/):
 * - `did:indy:besu(:network)?:<identifier>`
 * - `did:ethr(:network)?:<identifier>`
 *
 * At most ONE network segment is allowed, and the identifier must be either
 * a 20-byte Ethereum address or a 64/65-byte uncompressed public key
 * (hex-encoded, `0x` prefixed).
 */
export type DidString =
  | `did:indy:besu:${string}`
  | `did:ethr:${string}`;

/** Ethereum account address (`0x` + 40 hex chars) */
export type AddressString = string;

export interface ContractSpec {
  contractName: string;
  abi: Array<Record<string, unknown>>;
}

export interface ContractConfig {
  address: AddressString;
  specPath?: string | null;
  spec?: ContractSpec | null;
}

export interface QuorumConfig {
  nodes: string[];
  request_retries?: number | null;
  request_timeout?: number | null;
  retry_interval?: number | null;
}

export type Status =
  | { Ok: { block_number: number; block_timestamp: number } }
  | { Err: { msg: string } };

export interface PingStatus {
  status: Status;
}

export interface SignatureData {
  /** recovery ID used for public key recovery */
  recovery_id: number;
  /** ECDSA signature bytes */
  signature: number[];
}

export interface EventLog {
  /** 32-byte topic hashes (`0x` prefixed hex) */
  topics: string[];
  data: number[];
  block: number;
}

export type VerificationKeyType =
  | "Ed25519VerificationKey2018"
  | "X25519KeyAgreementKey2019"
  | "Ed25519VerificationKey2020"
  | "X25519KeyAgreementKey2020"
  | "JsonWebKey2020"
  | "EcdsaSecp256k1VerificationKey2019"
  | "EcdsaSecp256k1VerificationKey2020"
  | "EcdsaSecp256k1RecoveryMethod2020";

export interface VerificationMethod {
  id: string;
  type: VerificationKeyType;
  controller: string;
  blockchainAccountId?: string;
  publicKeyMultibase?: string;
  publicKeyHex?: string;
  publicKeyBase58?: string;
  publicKeyBase64?: string;
  publicKeyJwk?: string;
}

export type VerificationMethodOrReference = string | VerificationMethod;

export type ServiceType =
  | "LinkedDomains"
  | "DIDCommMessaging"
  | "CredentialRegistry"
  | "OID4VCI"
  | "OID4VP";

export interface ServiceEndpointObject {
  uri: string;
  accept: string[];
  routingKeys: string[];
}

export type ServiceEndpoint =
  | string
  | ServiceEndpointObject
  | Array<string | ServiceEndpointObject>;

export interface DidService {
  id: string;
  type: ServiceType;
  serviceEndpoint: ServiceEndpoint;
}

/** DID Document matching https://www.w3.org/TR/did-core/ */
export interface DidDocument {
  "@context": string | string[];
  id: DidString;
  controller?: string | string[];
  verificationMethod: VerificationMethod[];
  authentication?: VerificationMethodOrReference[];
  assertionMethod?: VerificationMethodOrReference[];
  capabilityInvocation?: VerificationMethodOrReference[];
  capabilityDelegation?: VerificationMethodOrReference[];
  keyAgreement?: VerificationMethodOrReference[];
  service?: DidService[];
  alsoKnownAs?: string[];
}

export interface DidMetadata {
  owner?: AddressString;
  created?: number;
  deactivated?: boolean;
  versionId?: number;
  updated?: number;
  nextVersionId?: number;
  nextUpdate?: number;
}

/** DID Record stored in the IndyBesu DID Registry */
export interface DidRecord {
  document: DidDocument;
  metadata: DidMetadata;
}

export type DidResolutionError =
  | "notFound"
  | "invalidDid"
  | "representationNotSupported"
  | "invalidDidUrl"
  | "methodNotSupported";

export interface DidResolutionMetadata {
  contentType: string | null;
  error: DidResolutionError | null;
  message: string | null;
}

export interface DidDocumentWithMeta {
  didDocument?: DidDocument;
  didDocumentMetadata: DidMetadata;
  didResolutionMetadata: DidResolutionMetadata;
}

export interface DidResolutionOptions {
  accept?: string | null;
  blockTag?: number | null;
}

export type PublicKeyPurpose = "veriKey" | "sigAuth" | "enc";

export type PublicKeyType =
  | "Ed25519VerificationKey2020"
  | "X25519KeyAgreementKey2020"
  | "EcdsaSecp256k1VerificationKey2020";

export interface PublicKeyAttribute {
  purpose: PublicKeyPurpose;
  type: PublicKeyType;
  publicKeyHex?: string;
  publicKeyBase64?: string;
  publicKeyBase58?: string;
  publicKeyPem?: string;
}

export interface ServiceAttribute {
  type: ServiceType;
  serviceEndpoint: ServiceEndpoint;
}

/** Attribute which can be associated with a `did:ethr` DID */
export type DidDocAttribute = PublicKeyAttribute | ServiceAttribute;

export interface DidAttributeChanged {
  identity: AddressString;
  name: string;
  value: number[];
  validTo: number;
  previousChange: number;
}

export interface DidDelegateChanged {
  identity: AddressString;
  delegate: AddressString;
  delegateType: number[];
  validTo: number;
  previousChange: number;
}

export interface DidOwnerChanged {
  identity: AddressString;
  owner: AddressString;
  previousChange: number;
}

export type DidEvent =
  | { AttributeChangedEvent: DidAttributeChanged }
  | { DelegateChanged: DidDelegateChanged }
  | { OwnerChanged: DidOwnerChanged };

/** AnonCreds Schema matching https://hyperledger.github.io/anoncreds-spec/#term:schema */
export interface SchemaData {
  issuerId: DidString;
  name: string;
  version: string;
  attrNames: string[];
}

export interface SchemaMetadata {
  created: number;
}

export interface SchemaRecord {
  schema: SchemaData;
  metadata: SchemaMetadata;
}

/** AnonCreds Credential Definition matching https://hyperledger.github.io/anoncreds-spec/#term:credential-definition */
export interface CredentialDefinitionData {
  issuerId: DidString;
  schemaId: string;
  credDefType: "CL";
  tag: string;
  value: Record<string, unknown>;
}

export interface CredentialDefinitionMetadata {
  created: number;
}

export interface CredentialDefinitionRecord {
  credentialDefinition: CredentialDefinitionData;
  metadata: CredentialDefinitionMetadata;
}

export interface AccumKey {
  z: string;
}

export interface RevocationRegistryPublicKeys {
  accumKey: AccumKey;
}

export interface RevocationRegistryDefinitionValue {
  maxCredNum: number;
  publicKeys: RevocationRegistryPublicKeys;
  tailsHash: string;
  tailsLocation: string;
}

/** AnonCreds Revocation Registry Definition matching https://hyperledger.github.io/anoncreds-spec/#term:revocation-registry-definition */
export interface RevocationRegistryDefinitionData {
  issuerId: DidString;
  revocDefType: "CL_ACCUM";
  credDefId: string;
  tag: string;
  value: RevocationRegistryDefinitionValue;
}

export interface RevocationRegistryDefinitionMetadata {
  created: number;
}

export interface RevocationRegistryDefinitionRecord {
  revocationRegistryDefinition: RevocationRegistryDefinitionData;
  metadata: RevocationRegistryDefinitionMetadata;
}

export interface RevocationRegistryEntryData {
  currentAccumulator: string;
  prevAccumulator?: string | null;
  issued?: number[] | null;
  revoked?: number[] | null;
}

export interface RevocationRegistryEntryValue {
  revRegDefId: string;
  issuerId: DidString;
  revRegEntryData: RevocationRegistryEntryData;
}

export interface RevocationStatusListData {
  issuerId: DidString;
  revRegDefId: string;
  revocationList: number[];
  currentAccumulator: string;
  timestamp: number;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ContractConfig[]")]
    pub type ContractConfigArray;

    #[wasm_bindgen(typescript_type = "QuorumConfig | undefined | null")]
    pub type OptionalQuorumConfig;

    #[wasm_bindgen(typescript_type = "Promise<PingStatus>")]
    pub type PromisePingStatus;

    #[wasm_bindgen(typescript_type = "Promise<number[]>")]
    pub type PromiseBytes;

    #[wasm_bindgen(typescript_type = "Promise<EventLog[]>")]
    pub type PromiseEventLogArray;

    #[wasm_bindgen(typescript_type = "Promise<string>")]
    pub type PromiseString;

    #[wasm_bindgen(typescript_type = "SignatureData")]
    pub type SignatureDataJS;

    #[wasm_bindgen(typescript_type = "DidDocument")]
    pub type DidDocumentJS;

    #[wasm_bindgen(typescript_type = "DidRecord")]
    pub type DidRecordJS;

    #[wasm_bindgen(typescript_type = "DidDocumentWithMeta")]
    pub type DidDocumentWithMetaJS;

    #[wasm_bindgen(typescript_type = "DidResolutionOptions | undefined | null")]
    pub type OptionalDidResolutionOptions;

    #[wasm_bindgen(typescript_type = "DidDocAttribute")]
    pub type DidDocAttributeJS;

    #[wasm_bindgen(typescript_type = "EventLog")]
    pub type EventLogJS;

    #[wasm_bindgen(typescript_type = "DidAttributeChanged")]
    pub type DidAttributeChangedJS;

    #[wasm_bindgen(typescript_type = "DidDelegateChanged")]
    pub type DidDelegateChangedJS;

    #[wasm_bindgen(typescript_type = "DidOwnerChanged")]
    pub type DidOwnerChangedJS;

    #[wasm_bindgen(typescript_type = "DidEvent")]
    pub type DidEventJS;

    #[wasm_bindgen(typescript_type = "SchemaData")]
    pub type SchemaDataJS;

    #[wasm_bindgen(typescript_type = "SchemaRecord")]
    pub type SchemaRecordJS;

    #[wasm_bindgen(typescript_type = "Record<string, unknown>")]
    pub type CredentialDefinitionValueJS;

    #[wasm_bindgen(typescript_type = "CredentialDefinitionData")]
    pub type CredentialDefinitionDataJS;

    #[wasm_bindgen(typescript_type = "CredentialDefinitionRecord")]
    pub type CredentialDefinitionRecordJS;

    #[wasm_bindgen(typescript_type = "RevocationRegistryDefinitionValue")]
    pub type RevocationRegistryDefinitionValueJS;

    #[wasm_bindgen(typescript_type = "RevocationRegistryDefinitionData")]
    pub type RevocationRegistryDefinitionDataJS;

    #[wasm_bindgen(typescript_type = "RevocationRegistryDefinitionRecord")]
    pub type RevocationRegistryDefinitionRecordJS;

    #[wasm_bindgen(typescript_type = "RevocationRegistryEntryValue")]
    pub type RevocationRegistryEntryValueJS;

    #[wasm_bindgen(typescript_type = "RevocationStatusListData")]
    pub type RevocationStatusListDataJS;

    #[wasm_bindgen(typescript_type = "AddressString[]")]
    pub type ValidatorAddressesJS;
}
