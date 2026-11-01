# WASM FFI for Indy Besu VDR

### TypeScript definitions

The generated package (`pkg/indy_besu_vdr_wasm.d.ts`) exports full TypeScript
definitions for every method input and output (see `src/types.rs`). They mirror
the serde serialization of the corresponding `indy-besu-vdr` Rust types:

* `DidString` — template-literal type accepting only the DID methods supported
  by this VDR per the [Indy DID method](https://hyperledger-indy.github.io/indy-did-method/)
  specification: `did:indy:besu(:network)?:<identifier>` and
  `did:ethr(:network)?:<identifier>`.
* `DidDocument`, `DidRecord`, `DidDocumentWithMeta`, `DidDocAttribute`, DID event types.
* `SchemaData`/`SchemaRecord`, `CredentialDefinitionData`/`CredentialDefinitionRecord`,
  revocation registry definition/entry/status-list types.
* Client types: `ContractConfig`, `QuorumConfig`, `PingStatus`, `EventLog`, `SignatureData`.

### Requirements

* Rust of version 1.79.0 or higher.
* Wasm-pack: https://rustwasm.github.io/wasm-pack/installer/

Install rustup alongside Homebrew Rust—no need to uninstall anything:

```sh
brew install rustup
```

Then initialize a toolchain and install the WebAssembly target:

```sh
export PATH="/opt/homebrew/opt/rustup/bin:$PATH"
rustup toolchain install stable
rustup default stable
rustup target add wasm32-unknown-unknown
```

### Build

#### NodeJS bindings

```
wasm-pack build --target=nodejs
```

#### WASM (consumed in browser without bundler usage) bindings

```
wasm-pack build --target=web
```

#### Bindings to use with webpack

```
wasm-pack build
```

### Run demo

#### NodeJS demo

* Build NodeJS bindings from `wasm` directory
  ```
  wasm-pack build --target=nodejs
  ```
* Install dependencies
  ```
  cd demo/node
  yarn install
  ```
* Run demo
  ```
  yarn start
  ```