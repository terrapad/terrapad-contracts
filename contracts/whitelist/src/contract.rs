#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, UsersCountResponse, GetUserResponse, GetUsersResponse};
use crate::state::{USERS, AlloInfo, State, STATE, INDEXOF};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        userlist: vec![],
    };

    STATE.save(deps.storage, &state)?;

    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::AddToWhiteList { users } => try_add(deps, info, users),
        ExecuteMsg::RemoveFromWhiteList { addrs } => try_remove(deps, info, addrs),
    }
}

pub fn try_add(deps: DepsMut, info: MessageInfo, users: Vec<AlloInfo>) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    // add to whitelist
    for i in 0..users.len() {
        if !USERS.has(deps.storage, users[i].wallet.clone()) {
            INDEXOF.save(deps.storage, users[i].wallet.clone(), &state.userlist.len())?;
            state.userlist.push(users[i].wallet.clone());
        }
        USERS.save(deps.storage, users[i].wallet.clone(), &users[i])?;
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_add"))
}

pub fn try_remove(deps: DepsMut, info: MessageInfo, addrs: Vec<String>) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    // validate address format
    for i in 0..addrs.len() {
        if USERS.has(deps.storage, addrs[i].clone()) {
            USERS.remove(deps.storage, addrs[i].clone());

            let index = INDEXOF.load(deps.storage, addrs[i].clone())?;
            let last_index = state.userlist.len() - 1;
            let last_user = state.userlist[last_index].clone();

            INDEXOF.save(deps.storage, last_user, &index)?;
            INDEXOF.remove(deps.storage, addrs[i].clone());

            state.userlist[index] = state.userlist[last_index].clone();
            state.userlist.pop();
        }
    }

    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "try_remove"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::UsersCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetUsers { page, limit } => to_binary(&query_users(deps, page, limit)?),
        QueryMsg::GetUser { user } => to_binary(&query_user(deps, user)?),
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
        end = state.userlist.len();
    }

    Ok(GetUsersResponse { users: state.userlist[start..end].to_vec() })
}

fn query_user(deps: Deps, user: String) -> StdResult<GetUserResponse> {
    if USERS.has(deps.storage, user.clone()) {
        return Ok(GetUserResponse { data: USERS.load(deps.storage, user)? });
    }
    Ok(GetUserResponse {
        data: AlloInfo {
            wallet: "".to_string(),
            public_allocation: 0,
            private_allocation: 0,
        }
    })
}
