use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use cosmwasm_std::testing::{mock_env, mock_info, mock_dependencies};
use cosmwasm_std::{
    Uint128,
};


#[test]
fn test_initialize() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        fund_denom: "uusd".to_string(),
        reward_token: "reward_token".to_string(),
        vesting: "vesting".to_string(),
        whitelist_merkle_root: "root".to_string(),

        exchange_rate: Uint128::from(1u128),
        private_start_time: 0,
        public_start_time: 0,
        presale_period: 100,
        distribution_amount: Uint128::from(1000u128),
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info, init_msg).unwrap();

    println!("{:?}", "Initializing contract ok")
}

#[test]
fn test_security() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        fund_denom: "uusd".to_string(),
        reward_token: "reward_token".to_string(),
        vesting: "vesting".to_string(),
        whitelist_merkle_root: "root".to_string(),

        exchange_rate: Uint128::from(1u128),
        private_start_time: 0,
        public_start_time: 0,
        presale_period: 100,
        distribution_amount: Uint128::from(1000u128),
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
        Err(ContractError::Unauthorized { }) => {},
        _ => panic!("Invalid error"),
    }

    let res = execute(
        deps.as_mut(),
        mock_env(),
        mock_info(&"user".to_string(), &[]),
        transfer_ownership_msg.clone(),
    );
    match res {
        Err(ContractError::Unauthorized { }) => {},
        _ => panic!("Invalid error"),
    }

    execute(deps.as_mut(), mock_env(), info.clone(), transfer_ownership_msg).unwrap();
    execute(deps.as_mut(), mock_env(), mock_info(&"user".to_string(), &[]), update_msg).unwrap();
}