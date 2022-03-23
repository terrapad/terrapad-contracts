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
    let init_msg = InstantiateMsg {
        reward_token: "reward_token".to_string(),
        release_interval: 60,
        release_rate: 10,
        initial_unlock: 10,
        lock_period: 600,
        vesting_period: 6000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    println!("{:?}", "Initializing contract ok")
}

#[test]
fn test_security() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        reward_token: "reward_token".to_string(),
        release_interval: 60,
        release_rate: 10,
        initial_unlock: 10,
        lock_period: 600,
        vesting_period: 6000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let update_msg = ExecuteMsg::UpdateRecipient { recp: "user".to_string(), amount: 1000 };
    let set_start_time_msg = ExecuteMsg::SetStartTime { new_start_time: 1000 };
    let transfer_ownership_msg = ExecuteMsg::TransferOwnerShip { new_owner: "user".to_string() };

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(&"user".to_string(), &[]),
        update_msg.clone(),
    );
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Invalid error"),
    }

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(&"user".to_string(), &[]),
        set_start_time_msg.clone(),
    );
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Invalid error"),
    }

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(&"user".to_string(), &[]),
        transfer_ownership_msg.clone(),
    );
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Invalid error"),
    }

    execute(deps.as_mut(), mock_env(), info.clone(), transfer_ownership_msg).unwrap();
    execute(deps.as_mut(), mock_env(), mock_info(&"user".to_string(), &[]), update_msg).unwrap();
    execute(deps.as_mut(), mock_env(), mock_info(&"user".to_string(), &[]), set_start_time_msg).unwrap();
}

#[test]
fn test_vesting_amount() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        reward_token: "reward_token".to_string(),
        release_interval: 60,
        release_rate: 100,
        initial_unlock: 100,
        lock_period: 600,
        vesting_period: 6000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg.clone()).unwrap();

    // update vesting info of `user`
    let user_vesting_amount: u64 = 1000;
    let msg = ExecuteMsg::UpdateRecipient { recp: "user".to_string(), amount: user_vesting_amount };
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let start_time: u64 = 1;

    // set start time of vesting
    let msg = ExecuteMsg::SetStartTime { new_start_time: start_time };
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // in lock period
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(start_time);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    assert_eq!(vested.amount, 0);
    assert_eq!(withdrawable.amount, 0);

    // initial unlock
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    let initial_unlock_amount = user_vesting_amount * init_msg.initial_unlock / 1000;
    assert_eq!(vested.amount, initial_unlock_amount);
    assert_eq!(withdrawable.amount, initial_unlock_amount);

    // first release tick
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period + init_msg.release_interval);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    let amount_per_interval = user_vesting_amount * init_msg.release_rate / 1000;
    assert_eq!(vested.amount, amount_per_interval + initial_unlock_amount);
    assert_eq!(withdrawable.amount, amount_per_interval + initial_unlock_amount);

    // before 5th release tick
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period + init_msg.release_interval * 5 - 1);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    assert_eq!(vested.amount, amount_per_interval * 4 + initial_unlock_amount);
    assert_eq!(withdrawable.amount, amount_per_interval * 4 + initial_unlock_amount);

    // after vesting period
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period + init_msg.vesting_period);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    assert_eq!(vested.amount, user_vesting_amount);
    assert_eq!(withdrawable.amount, user_vesting_amount);
}


#[test]
fn test_withdraw() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        reward_token: "reward_token".to_string(),
        release_interval: 60,
        release_rate: 100,
        initial_unlock: 100,
        lock_period: 600,
        vesting_period: 6000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg.clone()).unwrap();

    // update vesting info of `user`
    let user_vesting_amount: u64 = 1000;
    let msg = ExecuteMsg::UpdateRecipient { recp: "user".to_string(), amount: user_vesting_amount };
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    let start_time: u64 = 1;

    // set start time of vesting
    let msg = ExecuteMsg::SetStartTime { new_start_time: start_time };
    execute(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

    // 5th release tick
    let initial_unlock_amount = user_vesting_amount * init_msg.initial_unlock / 1000;
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period + init_msg.release_interval * 5);

    // withdraw
    let withdraw_msg = ExecuteMsg::Withdraw{};
    let res = execute(deps.as_mut(), env.clone(), mock_info(&"user".to_string(), &[]), withdraw_msg).unwrap();
    assert_eq!(res.messages.len(), 1);

    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    let amount_per_interval = user_vesting_amount * init_msg.release_rate / 1000;
    assert_eq!(vested.amount, amount_per_interval * 5 + initial_unlock_amount);
    assert_eq!(withdrawable.amount, 0);

    // after vesting period
    env.block.time = Timestamp::from_seconds(start_time + init_msg.lock_period + init_msg.vesting_period);
    let vested: AmountResponse = from_binary(
        &query(deps.as_ref(), env.clone(), QueryMsg::Vested { user: "user".to_string() }).unwrap(),
    ).unwrap();
    let withdrawable: AmountResponse = from_binary(
        &query(deps.as_ref(),env.clone(), QueryMsg::Withdrawable { user: "user".to_string() }).unwrap(),
    ).unwrap();

    assert_eq!(vested.amount, user_vesting_amount);
    assert_eq!(withdrawable.amount, user_vesting_amount - (amount_per_interval * 5 + initial_unlock_amount));
}
