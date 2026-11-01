// Copyright (c) 2024 DSR Corporation, Denver, Colorado.
// https://www.dsr-corporation.com
// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::future_to_promise;

use indy_besu_vdr::{ContractConfig, LedgerClient, QuorumConfig};

use crate::{
    error::{JsResult, Result},
    event_query::EventQueryWrapper,
    transaction::TransactionWrapper,
    types::{
        ContractConfigArray, OptionalQuorumConfig, PromiseBytes, PromiseEventLogArray,
        PromisePingStatus, PromiseString,
    },
};

#[wasm_bindgen(js_name = LedgerClient)]
pub struct LedgerClientWrapper(pub(crate) Rc<LedgerClient>);

#[wasm_bindgen(js_class = LedgerClient)]
impl LedgerClientWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        chain_id: u32,
        node_address: String,
        contract_configs: ContractConfigArray,
        network: Option<String>,
        quorum_config: OptionalQuorumConfig,
    ) -> Result<LedgerClientWrapper> {
        console_error_panic_hook::set_once();
        let contract_configs: Vec<ContractConfig> =
            serde_wasm_bindgen::from_value(contract_configs.into())?;
        let quorum_config: Option<QuorumConfig> =
            serde_wasm_bindgen::from_value(quorum_config.into()).ok();
        let client = LedgerClient::new(
            chain_id as u64,
            &node_address,
            &contract_configs,
            network.as_deref(),
            quorum_config.as_ref(),
        )
        .as_js()?;
        Ok(LedgerClientWrapper(Rc::new(client)))
    }

    pub fn ping(&self) -> PromisePingStatus {
        let client = self.0.clone();
        future_to_promise(async move {
            let status = client.ping().await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&status)?;
            Ok(result)
        })
        .unchecked_into()
    }

    #[wasm_bindgen(js_name = submitTransaction)]
    pub fn submit_transaction(&self, transaction: &TransactionWrapper) -> PromiseBytes {
        let client = self.0.clone();
        let transaction = transaction.0.clone();
        future_to_promise(async move {
            let transaction = transaction.borrow();
            let response = client.submit_transaction(&transaction).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&response)?;
            Ok(result)
        })
        .unchecked_into()
    }

    #[wasm_bindgen(js_name = queryEvents)]
    pub fn query_events(&self, query: &EventQueryWrapper) -> PromiseEventLogArray {
        let client = self.0.clone();
        let query = query.0.clone();
        future_to_promise(async move {
            let response = client.query_events(&query).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&response)?;
            Ok(result)
        })
        .unchecked_into()
    }

    #[wasm_bindgen(js_name = getReceipt)]
    pub fn get_receipt(&self, hash: Vec<u8>) -> PromiseString {
        let client = self.0.clone();
        future_to_promise(async move {
            let receipt = client.get_receipt(&hash).await.as_js()?;
            let result: JsValue = serde_wasm_bindgen::to_value(&receipt)?;
            Ok(result)
        })
        .unchecked_into()
    }
}
