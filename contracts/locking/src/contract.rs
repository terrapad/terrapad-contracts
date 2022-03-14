#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, Uint128, WasmMsg,
};

use crate::state::{
    read_config, read_lock_info, read_lock_infos, store_config, store_lock_info, Config, LockInfo
};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LockInfoResponse, LockedAccountsResponse,
};
use crate::types::OrderBy;
use cw20::Cw20ExecuteMsg;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    store_config(
        deps.storage,
        &Config {
            owner: deps.api.addr_canonicalize(&msg.owner)?,
            token: deps.api.addr_canonicalize(&msg.token)?,
            penalty_period: msg.penalty_period,
            dead: deps.api.addr_canonicalize(&msg.dead)?
        },
    )?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Deposit { amount } => deposit(deps, env, info, amount),
        ExecuteMsg::Withdraw { amount } => withdraw(deps, env, info, amount),
        _ => {
            assert_owner_privilege(deps.storage, deps.api, info.sender)?;
            match msg {
                ExecuteMsg::UpdateConfig {
                    owner,
                    token,
                    penalty_period,
                    dead,
                } => update_config(deps, owner, token, penalty_period, dead),
                _ => panic!("DO NOT ENTER HERE"),
            }
        }
    }
}

fn assert_owner_privilege(storage: &dyn Storage, api: &dyn Api, sender: Addr) -> StdResult<()> {
    if read_config(storage)?.owner != api.addr_canonicalize(sender.as_str())? {
        return Err(StdError::generic_err("unauthorized"));
    }

    Ok(())
}

pub fn update_config(
    deps: DepsMut,
    owner: Option<String>,
    token: Option<String>,
    penalty_period: Option<u64>,
    dead: Option<String>,
) -> StdResult<Response> {
    let mut config = read_config(deps.storage)?;
    if let Some(owner) = owner {
        config.owner = deps.api.addr_canonicalize(&owner)?;
    }

    if let Some(token) = token {
        config.token = deps.api.addr_canonicalize(&token)?;
    }

    if let Some(penalty_period) = penalty_period {
        config.penalty_period = penalty_period;
    }

    if let Some(dead) = dead {
        config.dead = deps.api.addr_canonicalize(&dead)?;
    }

    store_config(deps.storage, &config)?;

    Ok(Response::new().add_attributes(vec![("action", "update_config")]))
}

pub fn deposit(deps: DepsMut, env: Env, info: MessageInfo, amount: u64) -> StdResult<Response> {
    let current_time = env.block.time.seconds();
    let address = info.sender;
    let address_raw = deps.api.addr_canonicalize(&address.to_string())?;

    let config: Config = read_config(deps.storage)?;
    let mut lock_info: LockInfo = read_lock_info(deps.storage, &address_raw).unwrap_or(LockInfo { amount: 0, last_locked_time: current_time });

    let messages: Vec<CosmosMsg> = if amount == 0 {
        vec![]
    } else {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
                owner: address.to_string(),
                recipient: env.contract.address.into_string(),
                amount: Uint128::from(amount),
            })?,
        })]
    };

    lock_info.last_locked_time = current_time;
    lock_info.amount = amount;
    store_lock_info(deps.storage, &address_raw, &lock_info)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "deposit"),
        ("address", address.as_str()),
        ("amount", amount.to_string().as_str()),
        ("last_locked_time", current_time.to_string().as_str())
    ]))
}

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo, amount: u64) -> StdResult<Response> {
    let current_time = env.block.time.seconds();
    let address = info.sender;
    let address_raw = deps.api.addr_canonicalize(&address.to_string())?;

    let config: Config = read_config(deps.storage)?;
    let mut lock_info: LockInfo = read_lock_info(deps.storage, &address_raw)?;

    let penalty_amount = compute_penalty_amount(amount, current_time, &lock_info);
    let mut messages: Vec<CosmosMsg> = if penalty_amount == 0 {
        vec![]
    } else {
        vec![CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: deps.api.addr_humanize(&config.dead)?.to_string(),
                amount: Uint128::from(penalty_amount),
            })?,
        })]
    };
    messages.push(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: deps.api.addr_humanize(&config.token)?.to_string(),
            funds: vec![],
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: address.to_string(),
                amount: Uint128::from(amount - penalty_amount),
            })?,
        })
    );

    lock_info.amount = lock_info.amount - amount;
    store_lock_info(deps.storage, &address_raw, &lock_info)?;

    Ok(Response::new().add_messages(messages).add_attributes(vec![
        ("action", "withdraw"),
        ("address", address.as_str()),
        ("amount", amount.to_string().as_str()),
        ("penalty_amount", penalty_amount.to_string().as_str()),
    ]))
}


fn compute_penalty_amount(amount: u64, current_time: u64, lock_info: &LockInfo) -> u64 {
    let passed_time = current_time - lock_info.last_locked_time;
    return if passed_time < 10 * 86400 {
        amount / 10
    } else if passed_time < 20 * 86400 {
        amount / 20
    } else if passed_time < 30 * 86400 {
        amount / 30
    } else {
        0
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::LockInfo { address } => {
            Ok(to_binary(&query_lock_account(deps, address)?)?)
        }
        QueryMsg::LockedAccounts {
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_lock_accounts(
            deps,
            start_after,
            limit,
            order_by,
        )?)?),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state = read_config(deps.storage)?;
    let resp = ConfigResponse {
        owner: deps.api.addr_humanize(&state.owner)?.to_string(),
        token: deps.api.addr_humanize(&state.token)?.to_string(),
        penalty_period: state.penalty_period,
        dead: deps.api.addr_humanize(&state.dead)?.to_string(),
    };

    Ok(resp)
}

pub fn query_lock_account(deps: Deps, address: String) -> StdResult<LockInfoResponse> {
    let info = read_lock_info(deps.storage, &deps.api.addr_canonicalize(&address)?)?;
    let resp = LockInfoResponse { address, info };

    Ok(resp)
}

pub fn query_lock_accounts(
    deps: Deps,
    start_after: Option<String>,
    limit: Option<u32>,
    order_by: Option<OrderBy>,
) -> StdResult<LockedAccountsResponse> {
    let lock_infos = if let Some(start_after) = start_after {
        read_lock_infos(
            deps.storage,
            Some(deps.api.addr_canonicalize(&start_after)?),
            limit,
            order_by,
        )?
    } else {
        read_lock_infos(deps.storage, None, limit, order_by)?
    };

    let lock_account_responses: StdResult<Vec<LockInfoResponse>> = lock_infos
        .iter()
        .map(|lock_account| {
            Ok(LockInfoResponse {
                address: deps.api.addr_humanize(&lock_account.0)?.to_string(),
                info: lock_account.1.clone(),
            })
        })
        .collect();

    Ok(LockedAccountsResponse {
        lock_accounts: lock_account_responses?,
    })
}
