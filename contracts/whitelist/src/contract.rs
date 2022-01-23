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

mod tests {
    use cosmwasm_std::{Uint128, testing::{mock_dependencies, mock_info, mock_env}, from_binary};
    use super::*;

    #[test]
    fn test_initialize() {
        let mut deps = mock_dependencies(&[]);
        let init_msg = InstantiateMsg {};
        let info = mock_info(&"owner".to_string(), &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

        println!("{:?}", "Initializing contract ok")
    }

    #[test]
    fn test_add() {
        let mut deps = mock_dependencies(&[]);
        let init_msg = InstantiateMsg {};
        let info = mock_info(&"owner".to_string(), &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let mut list = Vec::new();
        for i in 1..10001 {
            list.push(AlloInfo { wallet: format!("user{}", i), public_allocation: 1, private_allocation: 2 });
        }
        let add_msg = ExecuteMsg::AddToWhiteList { users: list };

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&"user".to_string(), &[]),
            add_msg.clone(),
        );
        match res {
            Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
            _ => panic!("Invalid error"),
        }

        execute(deps.as_mut(), mock_env(), info.clone(), add_msg).unwrap();

        let user_count: UsersCountResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::UsersCount {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(user_count.count, 10000);

        let user1: GetUserResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetUser { user: "user1".to_string() },
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(user1.data.wallet, "user1".to_string());
        assert_eq!(user1.data.public_allocation, 1);
        assert_eq!(user1.data.private_allocation, 2);

        let mut list1 = Vec::new();
        list1.push(AlloInfo { wallet: "user2".to_string(), public_allocation: 2, private_allocation: 3 });
        list1.push(AlloInfo { wallet: "user3".to_string(), public_allocation: 1, private_allocation: 2 });
        let add_msg = ExecuteMsg::AddToWhiteList { users: list1 };
        execute(deps.as_mut(), mock_env(), info.clone(), add_msg).unwrap();

        let users: GetUsersResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::GetUsers { page: 0, limit: 4 },
            )
            .unwrap(),
        )
        .unwrap();

        let expected = ["user1".to_string(), "user2".to_string(), "user3".to_string()];
        for i in 0..3 {
            assert_eq!(users.users[i], expected[i]);
        }
    }

    #[test]
    fn test_remove() {
        let mut deps = mock_dependencies(&[]);
        let init_msg = InstantiateMsg {};
        let info = mock_info(&"owner".to_string(), &[]);
        let _ = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

        let mut list = Vec::new();
        list.push(AlloInfo { wallet: "user1".to_string(), public_allocation: 1, private_allocation: 2 });
        list.push(AlloInfo { wallet: "user2".to_string(), public_allocation: 2, private_allocation: 3 });
        list.push(AlloInfo { wallet: "user3".to_string(), public_allocation: 1, private_allocation: 2 });
        list.push(AlloInfo { wallet: "user4".to_string(), public_allocation: 1, private_allocation: 2 });
        execute(deps.as_mut(), mock_env(), info.clone(), ExecuteMsg::AddToWhiteList { users: list }).unwrap();

        let user_count: UsersCountResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::UsersCount {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(user_count.count, 4);

        let mut addrs = Vec::new();
        addrs.push("user2".to_string());
        addrs.push("user5".to_string());
        let remove_msg = ExecuteMsg::RemoveFromWhiteList { addrs };

        let res = execute(
            deps.as_mut(),
            mock_env(),
            mock_info(&"user".to_string(), &[]),
            remove_msg.clone(),
        );
        match res {
            Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
            _ => panic!("Invalid error"),
        }
        execute(deps.as_mut(), mock_env(), info, remove_msg).unwrap();

        let user_count: UsersCountResponse = from_binary(
            &query(
                deps.as_ref(),
                mock_env(),
                QueryMsg::UsersCount {},
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(user_count.count, 3);
    }
}
