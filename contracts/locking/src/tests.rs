use crate::contract::{execute, instantiate, query};
use crate::types::OrderBy;
use crate::msg::{
    ConfigResponse, ExecuteMsg, InstantiateMsg, QueryMsg, LockedAccountsResponse,
    LockInfoResponse,
};
use crate::state::{ Config, LockInfo };

use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
use cosmwasm_std::{
    attr, from_binary, to_binary, Api, CanonicalAddr, CosmosMsg, StdError, SubMsg, Timestamp,
    Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;

#[test]
fn proper_initialization() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        token: "token".to_string(),
        penalty_period: 12345u64,
        dead: "dead".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner".to_string(),
            token: "token".to_string(),
            penalty_period: 12345u64,
            dead: "dead".to_string()
        }
    );
}

#[test]
fn update_config() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        token: "token".to_string(),
        penalty_period: 12345u64,
        dead: "dead".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner2".to_string()),
        token: None,
        penalty_period: None,
        dead: None,
    };
    let info = mock_info("owner", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner2".to_string(),
            token: "token".to_string(),
            penalty_period: 12345u64,
            dead: "dead".to_string(),
        }
    );

    let msg = ExecuteMsg::UpdateConfig {
        owner: Some("owner".to_string()),
        token: None,
        penalty_period: None,
        dead: None
    };
    let info = mock_info("owner", &[]);
    let res = execute(deps.as_mut(), mock_env(), info, msg);
    match res {
        Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, "unauthorized"),
        _ => panic!("DO NOT ENTER HERE"),
    }

    let msg = ExecuteMsg::UpdateConfig {
        owner: None,
        token: Some("token2".to_string()),
        penalty_period: Some(1u64),
        dead: Some("dead2".to_string()),
    };
    let info = mock_info("owner2", &[]);
    let _res = execute(deps.as_mut(), mock_env(), info, msg).unwrap();

    assert_eq!(
        from_binary::<ConfigResponse>(
            &query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap()
        )
        .unwrap(),
        ConfigResponse {
            owner: "owner2".to_string(),
            token: "token2".to_string(),
            penalty_period: 1u64,
            dead: "dead2".to_string(),
        }
    );
}

#[test]
fn deposit_and_withdraw() {
    let mut deps = mock_dependencies(&[]);

    let msg = InstantiateMsg {
        owner: "owner".to_string(),
        token: "token".to_string(),
        penalty_period: 100u64,
        dead: "dead".to_string(),
    };

    let info = mock_info("addr0000", &[]);
    let _res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

    let msg = ExecuteMsg::Deposit { amount: 100u64 };
    let info = mock_info("addr0000", &[]);
    let mut env = mock_env();
    env.block.time = Timestamp::from_seconds(0);
    let _res = execute(deps.as_mut(), env.clone(), info, msg).unwrap();
    assert_eq!(
        _res.attributes,
        vec![
            attr("action", "deposit"),
            attr("address", "addr0000"),
            attr("amount", "100"),
            attr("last_locked_time", "0"),
        ]
    );

    let info = mock_info("addr0000", &[]);
    env.block.time = Timestamp::from_seconds(100);

    let msg = ExecuteMsg::Withdraw { amount: 100u64 };
    let res = execute(deps.as_mut(), env.clone(), info.clone(), msg.clone()).unwrap();
    assert_eq!(
        res.attributes,
        vec![
            attr("action", "withdraw"),
            attr("address", "addr0000"),
            attr("amount", "100"),
            attr("penalty_amount", "10"),
        ]
    );
    assert_eq!(
        res.messages,
        vec![SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "dead".to_string(),
                amount: Uint128::from(10u128),
            })
            .unwrap(),
            funds: vec![],
        })),
        SubMsg::new(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: "token".to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: "addr0000".to_string(),
                amount: Uint128::from(90u128),
            })
            .unwrap(),
            funds: vec![],
        }))],
    );
}
