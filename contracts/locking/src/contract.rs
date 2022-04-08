#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;

use cosmwasm_std::{
    to_binary, Addr, Api, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response,
    StdError, StdResult, Storage, Uint128, WasmMsg, from_binary,
};

use crate::state::{
    read_config, read_lock_info, read_lock_infos, store_config, store_lock_info, Config, LockInfo
};
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LockInfoResponse, LockedAccountsResponse, Cw20HookMsg, MigrateMsg,
};
use crate::types::OrderBy;
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

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
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}


#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Receive(msg) => receive_cw20(deps, env, info, msg),
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

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let config: Config = read_config(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            // only staking token contract can execute this message
            if config.token != deps.api.addr_canonicalize(info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            deposit(deps, env, cw20_sender, cw20_msg.amount)
        }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

pub fn deposit(deps: DepsMut, env: Env, sender: Addr, amount: Uint128) -> StdResult<Response> {
    let current_time = env.block.time.seconds();
    let address = sender;
    let address_raw = deps.api.addr_canonicalize(&address.to_string())?;

    let mut lock_info: LockInfo = read_lock_info(deps.storage, &address_raw).unwrap_or(LockInfo { amount: Uint128::zero(), last_locked_time: current_time });

    lock_info.last_locked_time = current_time;
    lock_info.amount = amount;
    store_lock_info(deps.storage, &address_raw, &lock_info)?;

    Ok(Response::new().add_attributes(vec![
        ("action", "deposit"),
        ("address", address.as_str()),
        ("amount", amount.to_string().as_str()),
        ("last_locked_time", current_time.to_string().as_str())
    ]))
}

pub fn withdraw(deps: DepsMut, env: Env, info: MessageInfo, amount: Uint128) -> StdResult<Response> {
    let current_time = env.block.time.seconds();
    let address = info.sender;
    let address_raw = deps.api.addr_canonicalize(&address.to_string())?;

    let config: Config = read_config(deps.storage)?;
    let mut lock_info: LockInfo = read_lock_info(deps.storage, &address_raw)?;

    let penalty_amount = compute_penalty_amount(amount, current_time, &lock_info);
    let mut messages: Vec<CosmosMsg> = if penalty_amount.is_zero() {
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


fn compute_penalty_amount(amount: Uint128, current_time: u64, lock_info: &LockInfo) -> Uint128 {
    let passed_time = current_time - lock_info.last_locked_time;
    return if passed_time < 10 * 86400 {
        amount.checked_div(Uint128::from(10u128)).unwrap()
    } else if passed_time < 20 * 86400 {
        amount.checked_div(Uint128::from(20u128)).unwrap()
    } else if passed_time < 30 * 86400 {
        amount.checked_div(Uint128::from(30u128)).unwrap()
    } else {
        Uint128::zero()
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => Ok(to_binary(&query_config(deps)?)?),
        QueryMsg::LockInfo { address } => {
            Ok(to_binary(&query_lock_account(deps, env, address)?)?)
        }
        QueryMsg::LockedAccounts {
            start_after,
            limit,
            order_by,
        } => Ok(to_binary(&query_lock_accounts(
            deps,
            env,
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

pub fn query_lock_account(deps: Deps, env: Env, address: String) -> StdResult<LockInfoResponse> {
    let info = read_lock_info(deps.storage, &deps.api.addr_canonicalize(&address)?).unwrap_or(LockInfo { amount: Uint128::zero(), last_locked_time: 0  });
    let penalty_amount = compute_penalty_amount(info.amount, env.block.time.seconds(), &info);
    let resp = LockInfoResponse { address, info, penalty: penalty_amount };

    Ok(resp)
}

pub fn query_lock_accounts(
    deps: Deps,
    env: Env,
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
                penalty: compute_penalty_amount(lock_account.1.amount, env.block.time.seconds(), &lock_account.1)
            })
        })
        .collect();

    Ok(LockedAccountsResponse {
        lock_accounts: lock_account_responses?,
    })
}
