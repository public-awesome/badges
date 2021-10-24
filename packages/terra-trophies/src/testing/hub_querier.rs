use crate::hub::{QueryMsg, TrophyInfo};
use cosmwasm_std::{to_binary, Addr, QuerierResult};
use cw721_metadata_onchain::Metadata;
use std::collections::HashMap;

pub struct HubQuerier {
    pub hub_addr: Addr,
    pub trophy_infos: HashMap<u64, TrophyInfo<String>>,
}

impl Default for HubQuerier {
    fn default() -> Self {
        let info_1 = TrophyInfo {
            creator: "creator_1".to_string(),
            metadata: Metadata {
                image: Some("ipfs://hash-to-image-1".to_string()),
                image_data: None,
                external_url: None,
                description: Some("I am trophy number one".to_string()),
                name: Some("Trophy Number One".to_string()),
                attributes: None,
                background_color: None,
                animation_url: Some("ipfs://hash-of-video-1".to_string()),
                youtube_url: None,
            },
        };
        let info_2 = TrophyInfo {
            creator: "creator_1".to_string(),
            metadata: Metadata {
                image: Some("ipfs://hash-to-image-2".to_string()),
                image_data: None,
                external_url: None,
                description: Some("I am trophy number two".to_string()),
                name: Some("Trophy Number Two".to_string()),
                attributes: None,
                background_color: None,
                animation_url: Some("ipfs://hash-of-video-2".to_string()),
                youtube_url: None,
            },
        };

        let mut trophy_infos: HashMap<u64, TrophyInfo<String>> = HashMap::new();
        trophy_infos.insert(1, info_1);
        trophy_infos.insert(2, info_2);

        Self {
            hub_addr: Addr::unchecked("hub"),
            trophy_infos,
        }
    }
}

impl HubQuerier {
    pub fn handle_query(&self, contract_addr: &Addr, msg: QueryMsg) -> QuerierResult {
        if contract_addr != &self.hub_addr {
            panic!(
                "[mock]: made an hub query but address is correct, was: {}, should be: {}",
                contract_addr, self.hub_addr
            );
        }

        match msg {
            QueryMsg::TrophyInfo {
                trophy_id,
            } => Ok(to_binary(&self.query_trophy_info(trophy_id)).into()).into(),
        }
    }

    fn query_trophy_info(&self, trophy_id: u64) -> TrophyInfo<String> {
        if let Some(info) = self.trophy_infos.get(&trophy_id) {
            info.clone()
        } else {
            panic!("[mock]: cannot find trophy info for id {}", trophy_id)
        }
    }
}
