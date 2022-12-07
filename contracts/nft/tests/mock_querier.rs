#![allow(dead_code)]

use std::collections::HashMap;

use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_binary, from_slice, to_binary, Addr, ContractInfoResponse, Empty, Querier, QuerierResult,
    QueryRequest, SystemError, WasmQuery,
};

use badges::{hub, Badge};

pub struct CustomQuerier {
    pub base: MockQuerier<Empty>,
    pub hub: HubQuerier,
}

impl Default for CustomQuerier {
    fn default() -> Self {
        let mut querier = CustomQuerier {
            base: MockQuerier::new(&[]),
            hub: HubQuerier::default(),
        };
        querier.base.update_wasm(wasm_querier_handler);
        querier
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
    badges: HashMap<u64, Badge>,
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
    pub fn set_badge(&mut self, id: u64, badge: Badge) {
        self.badges.insert(id, badge);
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
                    .cloned()
                    .unwrap_or_else(|| panic!("[mock]: cannot find badge with id {}", id));
                let res = hub::BadgeResponse::from((id, badge));
                Ok(to_binary(&res).into()).into()
            },

            _ => panic!("[mock]: unsupported hub query: {:?}", msg),
        }
    }
}

/// sg721 requires that the deployer must be a contract:
/// https://github.com/public-awesome/launchpad/blob/v0.21.1/contracts/sg721-base/src/contract.rs#L39-L47
///
/// to pass the test, we use a custom wasm query handler that returns "badge_hub"
/// as a valid contract, and make sure to use "badge_hub" here as the sender.
fn wasm_querier_handler(query: &WasmQuery) -> QuerierResult {
    match query {
        WasmQuery::ContractInfo {
            contract_addr,
        } if contract_addr == "badge_hub" => {
            Ok(to_binary(&ContractInfoResponse::new(69420, "larry")).into()).into()
        },
        _ => panic!("[mock]: unimplemented wasm query: {query:?}"),
    }
}
