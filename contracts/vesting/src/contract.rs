#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, CosmosMsg, WasmMsg, Uint128};
use cw20::Cw20ExecuteMsg;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UsersCountResponse, GetUserResponse, GetUsersResponse, AmountResponse, MigrateMsg};
use crate::state::{RECIPIENTS, UserInfo, State, STATE, ACCURACY};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        worker: deps.api.addr_canonicalize(info.sender.as_str())?,
        reward_token: deps.api.addr_canonicalize(msg.reward_token.as_str())?,
        release_interval: msg.release_interval,
        release_rate: msg.release_rate,
        initial_unlock: msg.initial_unlock,
        lock_period: msg.lock_period,
        vesting_period: msg.vesting_period,
        userlist: vec![],
        start_time: 0,
        total_vesting_amount: 0,
        distribution_amount: msg.distribution_amount
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

/************************************ Migration *************************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    Ok(Response::new())
}

/************************************ Execution *************************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::TransferOwnerShip { new_owner } => execute_transfer_ownership(deps, info, new_owner),
        ExecuteMsg::SetWorker { worker } => execute_set_worker(deps, info, worker),
        ExecuteMsg::SetStartTime { new_start_time } => execute_set_start_time(deps, info, new_start_time),
        ExecuteMsg::UpdateRecipient { recp, amount } => execute_update_recipient(deps, _env, info, recp, amount),
        ExecuteMsg::Withdraw {} => execute_withdraw(deps, _env, info)
    }
}

pub fn execute_transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> StdResult<Response> {
    let new_owner = deps.api.addr_canonicalize(new_owner.as_str())?;
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.owner = new_owner.clone();
    STATE.save(deps.storage, &state)?;

    let messages: Vec<CosmosMsg> = vec![];
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "transfer_ownership"))
}

pub fn execute_set_worker(deps: DepsMut, info: MessageInfo, worker: String) -> StdResult<Response> {
    let worker = deps.api.addr_canonicalize(worker.as_str())?;
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.worker = worker.clone();
    STATE.save(deps.storage, &state)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.reward_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::IncreaseAllowance {
            spender: deps.api.addr_humanize(&worker)?.to_string(),
            amount: Uint128::MAX,
            expires: None
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "set_worker"))
}

pub fn execute_set_start_time(deps: DepsMut, info: MessageInfo, new_start_time: u64) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.start_time = new_start_time;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "set_start_time"))
}

pub fn execute_update_recipient(deps: DepsMut, env: Env, info: MessageInfo, recp: String, amount: u64) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    let sender_canonical = deps.api.addr_canonicalize(info.sender.as_str())?;
    if sender_canonical != state.owner && sender_canonical != state.worker {
        return Err(StdError::generic_err("unauthorized"));
    }

    // timeline check
    if state.start_time != 0 && state.start_time < env.block.time.seconds()  {
        return Err(StdError::generic_err("already started"));
    }

    // update
    if RECIPIENTS.has(deps.storage, recp.clone()) {
        let recp_info = RECIPIENTS.load(deps.storage, recp.clone())?;
        state.total_vesting_amount = state.total_vesting_amount - recp_info.total_amount;
    } else {
        state.userlist.push(recp.clone());
    }
    RECIPIENTS.save(deps.storage, recp.clone(), &UserInfo { total_amount: amount, withrawn_amount: 0 })?;

    state.total_vesting_amount = state.total_vesting_amount + amount;
    if state.total_vesting_amount > state.distribution_amount {
        return Err(StdError::generic_err("exceed total distribution amount"));
    }
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "update_recipient"))
}

pub fn execute_withdraw(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let state: State = STATE.load(deps.storage)?;
    let sender = info.sender.into_string();
    let mut recpinfo = RECIPIENTS.load(deps.storage, sender.clone())?;
    if recpinfo.total_amount == 0 {
        return Ok(Response::new());
    }

    let vested = query_vested(deps.as_ref(), env.clone(), sender.clone())?;
    let withdrawable = query_withdrawable(deps.as_ref(), env.clone(), sender.clone())?;
    recpinfo.withrawn_amount = vested.amount;
    RECIPIENTS.save(deps.storage, sender.clone(), &recpinfo)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.reward_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: sender.clone(),
            amount: Uint128::from(withdrawable.amount),
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "withdraw"))
}

/************************************ Query *************************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UsersCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetUsers { page, limit } => to_binary(&query_users(deps, page, limit)?),
        QueryMsg::GetUser { user } => to_binary(&query_user(deps, user)?),
        QueryMsg::Vested { user } => to_binary( &query_vested(deps, _env, user)? ),
        QueryMsg::Locked { user } => to_binary( &query_locked(deps, _env, user)? ),
        QueryMsg::Withdrawable { user } => to_binary( &query_withdrawable(deps, _env, user)? ),
    }
}

fn query_count(deps: Deps) -> StdResult<UsersCountResponse> {
    let state: State = STATE.load(deps.storage)?;
    Ok(UsersCountResponse { count: state.userlist.len() as u64 })
}

fn query_users(deps: Deps, page: u64, limit: u64) -> StdResult<GetUsersResponse> {
    let state: State = STATE.load(deps.storage)?;

    let start = (page * limit) as usize;
    let mut end = (page * limit + limit) as usize;
    if end > state.userlist.len() {
        end = state.userlist.len()
    };

    Ok(GetUsersResponse { users: state.userlist[start..end].to_vec() })
}

fn query_user(deps: Deps, user: String) -> StdResult<GetUserResponse> {
    let recp_data = RECIPIENTS.load(deps.storage, user).unwrap_or(UserInfo { total_amount: 0, withrawn_amount: 0 });
    Ok(GetUserResponse { data: recp_data })
}

fn query_vested(deps: Deps, env: Env, user: String) -> StdResult<AmountResponse> {
    let state: State = STATE.load(deps.storage)?;

    let lock_end_time = state.start_time + state.lock_period;
    let vesting_end_time = lock_end_time + state.vesting_period;
    let recpinfo = RECIPIENTS.load(deps.storage, user).unwrap_or(UserInfo { total_amount: 0, withrawn_amount: 0 });

    let amount: u64;
    if state.start_time == 0 || recpinfo.total_amount == 0 || env.block.time.seconds() < lock_end_time {
        amount = 0;
    } else if env.block.time.seconds() >= vesting_end_time {
        amount = recpinfo.total_amount;
    } else {
        let initial_unlock_amount = recpinfo.total_amount * state.initial_unlock / ACCURACY;
        let unlock_amount_per_interval = recpinfo.total_amount * state.release_rate / ACCURACY;
        let mut vested_amount = (env.block.time.seconds() - lock_end_time) / state.release_interval * unlock_amount_per_interval + initial_unlock_amount;
        vested_amount = if recpinfo.withrawn_amount > vested_amount { recpinfo.withrawn_amount } else { vested_amount };
        amount = if vested_amount > recpinfo.total_amount { recpinfo.total_amount } else { vested_amount};
    }

    Ok(AmountResponse { amount })
}

fn query_locked(deps: Deps, env: Env, user: String) -> StdResult<AmountResponse> {
    let recpinfo = RECIPIENTS.load(deps.storage, user.clone()).unwrap_or(UserInfo { total_amount: 0, withrawn_amount: 0 });
    let vested = query_vested(deps, env, user.clone())?;

    Ok(AmountResponse { amount: recpinfo.total_amount - vested.amount })
}

fn query_withdrawable(deps: Deps, env: Env, user: String) -> StdResult<AmountResponse> {
    let recpinfo = RECIPIENTS.load(deps.storage, user.clone()).unwrap_or(UserInfo { total_amount: 0, withrawn_amount: 0 });
    let vested = query_vested(deps, env, user.clone())?;

    Ok(AmountResponse { amount: vested.amount - recpinfo.withrawn_amount })
}
