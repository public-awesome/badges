use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map, U64Key};

use terra_trophies::hub::TrophyInfo;

pub struct State<'a> {
    pub nft: Item<'a, Addr>,
    pub trophies: Map<'a, U64Key, TrophyInfo<Addr>>,
    pub trophy_count: Item<'a, u64>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            nft: Item::new("nft"),
            trophies: Map::new("trophies"),
            trophy_count: Item::new("trophy_count"),
        }
    }
}
