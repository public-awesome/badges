#![allow(dead_code)]

use std::collections::HashMap;

use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, Empty, Querier, QuerierResult, QueryRequest,
    SystemError, WasmQuery,
};

use badges::{hub, Badge};

pub struct CustomQuerier {
    pub base: MockQuerier<Empty>,
    pub hub: HubQuerier,
}

impl Default for CustomQuerier {
    fn default() -> Self {
        CustomQuerier {
            base: MockQuerier::new(&[]),
            hub: HubQuerier::default(),
        }
    }
}

impl Querier for CustomQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = from_slice(bin_request)
            .map_err(|err| SystemError::InvalidRequest {
                error: format!("[mock]: parsing query request: {}", err),
                request: bin_request.into(),
            })
            .unwrap();

        match request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                let contract_addr = Addr::unchecked(contract_addr);

                if let Ok(hub_query_msg) = from_binary::<hub::QueryMsg>(&msg) {
                    return self.hub.handle_query(&contract_addr, hub_query_msg);
                }

                panic!("[mock]: unsupported wasm query: {:?}", msg);
            },

            _ => self.base.handle_query(&request),
        }
    }
}

pub struct HubQuerier {
    contract_addr: Addr,
    badges: HashMap<u64, Badge<String>>,
}

impl Default for HubQuerier {
    fn default() -> Self {
        HubQuerier {
            contract_addr: Addr::unchecked("hub"),
            badges: HashMap::default(),
        }
    }
}

impl HubQuerier {
    pub fn set_badge(&mut self, badge: Badge<String>) {
        self.badges.insert(badge.id, badge);
    }

    pub fn handle_query(&self, contract_addr: &Addr, msg: hub::QueryMsg) -> QuerierResult {
        if contract_addr != &self.contract_addr {
            panic!(
                "[mock]: made a badge hub query but addresses is incorrect: expected {}, found {}",
                self.contract_addr, contract_addr
            );
        }

        match msg {
            hub::QueryMsg::Badge {
                id,
            } => {
                let badge = self
                    .badges
                    .get(&id)
                    .unwrap_or_else(|| panic!("[mock]: cannot find badge with id {}", id));
                Ok(to_binary(badge).into()).into()
            },

            _ => panic!("[mock]: unsupported hub query: {:?}", msg),
        }
    }
}
