use cosmwasm_std::testing::MockQuerier;
use cosmwasm_std::{
    from_binary, from_slice, Addr, Empty, Querier, QuerierResult, QueryRequest, SystemError,
    WasmQuery,
};

use super::hub_querier::HubQuerier;
use crate::hub;

pub struct CustomQuerier {
    base: MockQuerier<Empty>,
    hub_querier: HubQuerier,
}

impl Querier for CustomQuerier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
                .into()
            }
        };
        self.handle_query(&request)
    }
}

impl CustomQuerier {
    pub fn new() -> Self {
        Self {
            base: MockQuerier::new(&[]),
            hub_querier: HubQuerier::default(),
        }
    }

    pub fn handle_query(&self, request: &QueryRequest<Empty>) -> QuerierResult {
        match request {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr,
                msg,
            }) => {
                let contract_addr = Addr::unchecked(contract_addr);

                if let Ok(hub_query_msg) = from_binary::<hub::QueryMsg>(msg) {
                    return self.hub_querier.handle_query(&contract_addr, hub_query_msg);
                }

                panic!("[mock]: unsupported wasm query: {:?}", msg)
            }

            _ => self.base.handle_query(request),
        }
    }
}
