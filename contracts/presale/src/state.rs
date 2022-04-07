use cosmwasm_storage::{singleton, singleton_read};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, Uint128, Storage, StdResult};
use cw_storage_plus::{Map};

const KEY_STATE: &[u8] = b"state";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    // Owner address
    pub owner: CanonicalAddr,

    /************** Address Infos *************/
    // Token for fundraise.
    pub fund_denom: String,
    // Token for distribution.
    pub reward_token: CanonicalAddr,
    // Vesting Contract.
    pub vesting: CanonicalAddr,
    // Whitelist Merkle Root.
    pub whitelist_merkle_root: String,

    /************** Presale Params *************/
    // Fixed rate between fundToken vs rewardToken = reward / fund * ACCURACY.
    pub exchange_rate: Uint128,
    // Presale Period.
    pub presale_period: u64,
    // Public Presale Start Time.
    pub public_start_time: u64,
    // Private Presale Start Time.
    pub private_start_time: u64,

    /************** Status Info *************/
    // Reward token amount sold by private sale
    pub private_sold_amount: Uint128,
    // Reward token amount sold by public sale
    pub public_sold_amount: Uint128,
    // Participants address list
    pub userlist: Vec<String>,
}

pub fn store_state(storage: &mut dyn Storage, state: &State) -> StdResult<()> {
    singleton::<State>(storage, KEY_STATE).save(state)
}

pub fn read_state(storage: &dyn Storage) -> StdResult<State> {
    singleton_read::<State>(storage, KEY_STATE).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AlloInfo {
    // Max allocation for this user in public presale
    pub public_allocation: Uint128,
    // Max allocation for this user in private presale
    pub private_allocation: Uint128,
}

pub const PRIVATE_SOLD_FUNDS: Map<String, Uint128> = Map::new("private_sold_funds");

pub const ACCURACY: u128 = 100000000u128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Participant {
    // Fund token amount by participant.
    pub fund_balance: Uint128,
    // Reward token amount need to be vested.
    pub reward_balance: Uint128,
}

pub const PARTICIPANTS: Map<String, Participant> = Map::new("participants");
