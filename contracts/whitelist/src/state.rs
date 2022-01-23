use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CanonicalAddr;
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // Owner of this whitelist
    pub owner: CanonicalAddr,
    // Whitelisted user address list
    pub userlist: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AlloInfo {
    // User wallet address
    pub wallet: String,
    // Max allocation for this user in public presale
    pub public_allocation: u64,
    // Max allocation for this user in private presale
    pub private_allocation: u64,
}

pub const STATE: Item<State> = Item::new("state");

pub const USERS: Map<String, AlloInfo> = Map::new("users");

pub const INDEXOF: Map<String, usize> = Map::new("indexOf");
