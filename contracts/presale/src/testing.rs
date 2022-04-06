use crate::contract::{execute, instantiate};
use crate::error::ContractError;
use crate::msg::{InstantiateMsg, ExecuteMsg};
use crate::state::AlloInfo;
use cosmwasm_std::testing::{mock_env, mock_info, mock_dependencies};
use cosmwasm_std::{
    Uint128, Coin, Timestamp,
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

#[test]
fn test_deposit() {
    let mut deps = mock_dependencies(&[]);
    let init_msg = InstantiateMsg {
        fund_denom: "uusd".to_string(),
        reward_token: "reward_token".to_string(),
        vesting: "vesting".to_string(),
        whitelist_merkle_root: "2c0540dec9298f8a56e5a017a1a2613b06f6f99fb89f2957430dfcbf8bf8ed9e".to_string(),

        exchange_rate: Uint128::from(1u128),
        private_start_time: 0,
        public_start_time: 0,
        presale_period: 1000,
        distribution_amount: Uint128::from(1000u128),
    };
    let info = mock_info(&"owner".to_string(), &[]);
    let _ = instantiate(deps.as_mut(), mock_env(), info.clone(), init_msg).unwrap();

    let allo_info = AlloInfo { public_allocation: Uint128::from(1000u128), private_allocation: Uint128::from(100u128) };
    let proof = vec![
        "752a380bf2251efcea11d8e71fa03418ba31ba34d72854139f45d56c7602ebc3".to_string(),
        "f3b499bf4aa1f3832d7661832ba8be436b6e5e7f9feb3c4b3a5d87bb8612e2cf".to_string()
    ];

    let info = mock_info(&"user".to_string(), &[
        Coin {
            denom: "uusd".to_string(),
            amount: Uint128::from(100u128)
        }
    ]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(1);
    let msg = ExecuteMsg::Deposit { allo_info, proof };
    execute(deps.as_mut(), env.clone(), info.clone(), msg).unwrap();
}
