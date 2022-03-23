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
        fund_token: "fund_token".to_string(),
        reward_token: "reward_token".to_string(),
        vesting: "vesting".to_string(),
        whitelist: "whitelist".to_string(),

        exchange_rate: 1,
        private_start_time: 0,
        public_start_time: 0,
        presale_period: 100,
        distribution_amount: 1000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    println!("{:?}", "Initializing contract ok")
}

#[test]
fn test_security() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        fund_token: "fund_token".to_string(),
        reward_token: "reward_token".to_string(),
        vesting: "vesting".to_string(),
        whitelist: "whitelist".to_string(),

        exchange_rate: 1,
        private_start_time: 0,
        public_start_time: 0,
        presale_period: 100,
        distribution_amount: 1000,
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let update_msg = ExecuteMsg::UpdatePresaleInfo {
        new_private_start_time: 1,
        new_public_start_time: 10,
        new_presale_period: 100,
    };
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
        transfer_ownership_msg.clone(),
    );
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("Invalid error"),
    }

    execute(deps.as_mut(), mock_env(), info.clone(), transfer_ownership_msg).unwrap();
    execute(deps.as_mut(), mock_env(), mock_info(&"user".to_string(), &[]), update_msg).unwrap();
}