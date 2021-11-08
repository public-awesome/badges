use cosmwasm_std::Addr;
use cw_storage_plus::{Item, Map, U64Key};

use terra_trophies::hub::TrophyInfo;

pub struct State<'a> {
    /// Address of `trophy-nft` contract
    pub nft: Item<'a, Addr>,
    /// Total number of trophies
    pub trophy_count: Item<'a, u64>,
    /// Info of trophies
    pub trophies: Map<'a, U64Key, TrophyInfo<Addr>>,
    /// Whether a user has claimed a certain trophy
    pub claimed: Map<'a, (&'a Addr, U64Key), bool>,
}

impl<'a> Default for State<'a> {
    fn default() -> Self {
        Self {
            nft: Item::new("nft"),
            trophy_count: Item::new("trophy_count"),
            trophies: Map::new("trophies"),
            claimed: Map::new("claimed"),
        }
    }
}
