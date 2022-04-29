use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{CanonicalAddr, StdResult, Storage, Uint128};
use cosmwasm_storage::{bucket, bucket_read, singleton, singleton_read, ReadonlyBucket};

use crate::types::OrderBy;

const KEY_CONFIG: &[u8] = b"config";
const PREFIX_KEY_LOCKING_INFO: &[u8] = b"locking_info";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub owner: CanonicalAddr,
    pub token: CanonicalAddr,
    pub penalty_period: u64,
    pub dead: CanonicalAddr
}

pub fn store_config(storage: &mut dyn Storage, config: &Config) -> StdResult<()> {
    singleton::<Config>(storage, KEY_CONFIG).save(config)
}

pub fn read_config(storage: &dyn Storage) -> StdResult<Config> {
    singleton_read::<Config>(storage, KEY_CONFIG).load()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct LockInfo {
    // Total amount of tokens to locked.
    pub amount: Uint128,
    // Last locked time.
    pub last_locked_time: u64,
    // Last unlocked time.
    pub last_unlocked_time: u64,
}

pub fn read_lock_info(storage: &dyn Storage, address: &CanonicalAddr) -> StdResult<LockInfo> {
    bucket_read::<LockInfo>(storage, PREFIX_KEY_LOCKING_INFO).load(address.as_slice())
}

pub fn store_lock_info(
    storage: &mut dyn Storage,
    address: &CanonicalAddr,
    lock_info: &LockInfo,
) -> StdResult<()> {
    bucket::<LockInfo>(storage, PREFIX_KEY_LOCKING_INFO).save(address.as_slice(), lock_info)
}

const MAX_LIMIT: u32 = 30;
const DEFAULT_LIMIT: u32 = 10;
pub fn read_lock_infos<'a>(
    storage: &'a dyn Storage,
    start_after: Option<CanonicalAddr>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<Vec<(CanonicalAddr, LockInfo)>> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;
    let (start, end, order_by) = match order_by {
        Some(OrderBy::Asc) => (calc_range_start_addr(start_after), None, OrderBy::Asc),
        _ => (None, calc_range_end_addr(start_after), OrderBy::Desc),
    };

    let lock_accounts: ReadonlyBucket<'a, LockInfo> =
        ReadonlyBucket::new(storage, PREFIX_KEY_LOCKING_INFO);

    lock_accounts
        .range(start.as_deref(), end.as_deref(), order_by.into())
        .take(limit)
        .map(|item| {
            let (k, v) = item?;
            Ok((CanonicalAddr::from(k), v))
        })
        .collect()
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_start_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| {
        let mut v = addr.as_slice().to_vec();
        v.push(1);
        v
    })
}

// this will set the first key after the provided key, by appending a 1 byte
fn calc_range_end_addr(start_after: Option<CanonicalAddr>) -> Option<Vec<u8>> {
    start_after.map(|addr| addr.as_slice().to_vec())
}
