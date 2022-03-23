use crate::contract::{execute, instantiate, query};
use crate::mock_querier::mock_dependencies;
use crate::msg::ExecuteMsg::UpdateConfig;
use crate::msg::{
    ConfigResponse, Cw20HookMsg, ExecuteMsg, InstantiateMsg, QueryMsg, StakerInfoResponse,
    StateResponse,
};
use cosmwasm_std::testing::{mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, CosmosMsg, Decimal, StdError, SubMsg, Uint128, WasmMsg,
};
use cw20::{Cw20ExecuteMsg, Cw20ReceiveMsg};

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