use std::convert::TryInto;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, StdError, CosmosMsg, WasmMsg, Uint128, WasmQuery, QueryRequest, from_binary, Addr};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, TokenInfoResponse, BalanceResponse, Cw20ReceiveMsg};
use whitelist::msg::GetUserResponse;

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ParticipantsCountResponse, GetParticipantResponse, GetParticipantsResponse, GetSaleStatusResponse, Cw20HookMsg};
use crate::state::{PARTICIPANTS, STATE, PRIVATE_SOLD_FUNDS, ACCURACY, State, Participant};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        fund_token: deps.api.addr_canonicalize(msg.fund_token.as_str())?,
        reward_token: deps.api.addr_canonicalize(msg.reward_token.as_str())?,
        vesting: deps.api.addr_canonicalize(msg.vesting.as_str())?,
        whitelist: deps.api.addr_canonicalize(msg.whitelist.as_str())?,

        exchange_rate: msg.exchange_rate,
        presale_period: msg.presale_period,
        public_start_time: msg.public_start_time,
        private_start_time: msg.private_start_time,
        distribution_amount: msg.distribution_amount,

        private_sold_amount: 0,
        public_sold_amount: 0,
        userlist: vec![],
    };

    STATE.save(deps.storage, &state)?;

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
        ExecuteMsg::TransferOwnerShip {
            new_owner
        } => execute_transfer_ownership(deps, info, new_owner),
        ExecuteMsg::UpdatePresaleInfo {
            new_private_start_time,
            new_public_start_time,
            new_presale_period
        } => execute_update_info(deps, info, new_private_start_time, new_public_start_time, new_presale_period),
        ExecuteMsg::Receive(msg) => receive_cw20(deps, _env, info, msg),
        ExecuteMsg::WithdrawFunds { receiver } => execute_withdraw_funds(deps, _env, info, receiver),
        ExecuteMsg::WithdrawUnsoldToken { receiver } => execute_withdraw_unsold_token(deps, _env, info, receiver),
        ExecuteMsg::StartVesting {} => execute_start_vesting(deps, _env, info)
    }
}

pub fn receive_cw20(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    cw20_msg: Cw20ReceiveMsg,
) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    match from_binary(&cw20_msg.msg) {
        Ok(Cw20HookMsg::Deposit {}) => {
            // only staking token contract can execute this message
            if state.fund_token != deps.api.addr_canonicalize(info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            execute_deposit(deps, env, cw20_sender, u128::from(cw20_msg.amount).try_into().unwrap())
        }
        Ok(Cw20HookMsg::DepositPrivateSale {}) => {
            // only staking token contract can execute this message
            if state.fund_token != deps.api.addr_canonicalize(info.sender.as_str())? {
                return Err(StdError::generic_err("unauthorized"));
            }

            let cw20_sender = deps.api.addr_validate(&cw20_msg.sender)?;
            execute_deposit_private_sale(deps, env, cw20_sender, u128::from(cw20_msg.amount).try_into().unwrap())
        }
        Err(_) => Err(StdError::generic_err("data should be given")),
    }
}

pub fn execute_transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> StdResult<Response> {
    let new_owner = deps.api.addr_canonicalize(new_owner.as_str())?;
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.owner = new_owner;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "transfer_ownership"))
}

pub fn execute_update_info(deps: DepsMut, info: MessageInfo, new_private_start_time: u64, new_public_start_time: u64, new_presale_period: u64) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    state.private_start_time = new_private_start_time;
    state.public_start_time = new_public_start_time;
    state.presale_period = new_presale_period;
    STATE.save(deps.storage, &state)?;

    Ok(Response::new().add_attribute("method", "set_start_time"))
}

pub fn execute_deposit(deps: DepsMut, env: Env, sender: Addr, amount: u64) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() > end_time || env.block.time.seconds() < state.public_start_time {
        return Err(StdError::generic_err("presale not in progress"));
    }

    let sender = sender.to_string();
    let allo_info: GetUserResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.whitelist.to_string(),
        msg: to_binary(&whitelist::msg::QueryMsg::GetUser {
            user: sender.clone(),
        })?,
    }))?;

    if allo_info.data.wallet != sender.clone() {
        return Err(StdError::generic_err("not whitelisted"));
    }

    let mut recp_info = Participant {
        fund_balance: 0,
        reward_balance: 0
    };
    let mut private_sold_fund = 0;
    if PARTICIPANTS.has(deps.storage, sender.clone()) {
        recp_info = PARTICIPANTS.load(deps.storage, sender.clone())?;
        private_sold_fund = PRIVATE_SOLD_FUNDS.load(deps.storage, sender.clone())?;
    } else {
        state.userlist.push(sender.clone());
    }

    let new_fund_balance = recp_info.fund_balance + amount;
    if allo_info.data.public_allocation + private_sold_fund < new_fund_balance {
        return Err(StdError::generic_err("exceed allocation"));
    }

    let fund_token_info: TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.fund_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;
    let reward_token_info: TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    let mut reward_amount = (amount as u128).checked_mul(ACCURACY as u128).unwrap().checked_div(state.exchange_rate as u128).unwrap();
    reward_amount = reward_amount.checked_mul(u128::pow(10, reward_token_info.decimals as u32)).unwrap().checked_div(u128::pow(10, fund_token_info.decimals as u32)).unwrap();
    let reward_amount: u64 = reward_amount.try_into().unwrap();

    recp_info.fund_balance = new_fund_balance;
    recp_info.reward_balance = recp_info.reward_balance + reward_amount;
    state.public_sold_amount = state.public_sold_amount + reward_amount;

    STATE.save(deps.storage, &state)?;
    PARTICIPANTS.save(deps.storage, sender.clone(), &recp_info)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.vesting.to_string(),
        msg: to_binary(&vesting::msg::ExecuteMsg::UpdateRecipient {
            recp: sender.clone(),
            amount: recp_info.reward_balance,
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "deposit"))
}

pub fn execute_deposit_private_sale(deps: DepsMut, env: Env, sender: Addr, amount: u64) -> StdResult<Response> {
    let mut state: State = STATE.load(deps.storage)?;

    if env.block.time.seconds() < state.private_start_time {
        return Err(StdError::generic_err("private not in progress"));
    }

    let sender = sender.to_string();
    let allo_info: GetUserResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.whitelist.to_string(),
        msg: to_binary(&whitelist::msg::QueryMsg::GetUser {
            user: sender.clone(),
        })?,
    }))?;

    if allo_info.data.wallet != sender.clone() {
        return Err(StdError::generic_err("not whitelisted"));
    }

    let mut recp_info = Participant {
        fund_balance: 0,
        reward_balance: 0
    };
    let mut private_sold_fund = 0;
    if PARTICIPANTS.has(deps.storage, sender.clone()) {
        recp_info = PARTICIPANTS.load(deps.storage, sender.clone())?;
        private_sold_fund = PRIVATE_SOLD_FUNDS.load(deps.storage, sender.clone())?;
    } else {
        state.userlist.push(sender.clone());
    }

    let new_fund_balance = recp_info.fund_balance + amount;
    if allo_info.data.private_allocation < new_fund_balance {
        return Err(StdError::generic_err("exceed allocation"));
    }

    let fund_token_info: TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.fund_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;
    let reward_token_info: TokenInfoResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::TokenInfo {})?,
    }))?;

    let mut reward_amount = (amount as u128).checked_mul(ACCURACY as u128).unwrap().checked_div(state.exchange_rate as u128).unwrap();
    reward_amount = reward_amount.checked_mul(u128::pow(10, reward_token_info.decimals as u32)).unwrap().checked_div(u128::pow(10, fund_token_info.decimals as u32)).unwrap();
    let reward_amount: u64 = reward_amount.try_into().unwrap();

    recp_info.fund_balance = new_fund_balance;
    recp_info.reward_balance = recp_info.reward_balance + reward_amount;
    state.private_sold_amount = state.private_sold_amount + reward_amount;
    private_sold_fund = private_sold_fund + amount;

    STATE.save(deps.storage, &state)?;
    PARTICIPANTS.save(deps.storage, sender.clone(), &recp_info)?;
    PRIVATE_SOLD_FUNDS.save(deps.storage, sender.clone(), &private_sold_fund)?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.fund_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: sender.clone(),
            recipient: env.contract.address.into_string(),
            amount: Uint128::from(amount),
        })?,
        funds: vec![],
    }));
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.vesting.to_string(),
        msg: to_binary(&vesting::msg::ExecuteMsg::UpdateRecipient {
            recp: sender.clone(),
            amount: recp_info.reward_balance,
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "deposit_private"))
}

pub fn execute_withdraw_funds(deps: DepsMut, env: Env, info: MessageInfo, receiver: String) -> StdResult<Response> {
    let state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() <= end_time {
        return Err(StdError::generic_err("presale in progress"));
    }

    let fund_balance_info: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.fund_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: env.contract.address.to_string()
        })?,
    }))?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::Transfer {
            recipient: receiver.clone(),
            amount: fund_balance_info.balance,
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "withdraw_funds"))
}

pub fn execute_withdraw_unsold_token(deps: DepsMut, env: Env, info: MessageInfo, receiver: String) -> StdResult<Response> {
    let state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() <= end_time {
        return Err(StdError::generic_err("presale in progress"));
    }

    let reward_balance_info: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: state.vesting.to_string()
        })?,
    }))?;

    let sold_amount = state.private_sold_amount + state.public_sold_amount;
    let unsold_amount = reward_balance_info.balance - Uint128::from(sold_amount);

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: state.vesting.to_string(),
            recipient: receiver.clone(),
            amount: unsold_amount,
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "withdraw_unsold_token"))
}

pub fn execute_start_vesting(deps: DepsMut, env: Env, info: MessageInfo) -> StdResult<Response> {
    let state: State = STATE.load(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(StdError::generic_err("unauthorized"));
    }

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() <= end_time {
        return Err(StdError::generic_err("presale in progress"));
    }

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: state.reward_token.to_string(),
        msg: to_binary(&vesting::msg::ExecuteMsg::SetStartTime {
            new_start_time: env.block.time.seconds() + 1
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "start_vesting"))
}

/************************************ Query *************************************/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::ParticipantsCount {} => to_binary(&query_count(deps)?),
        QueryMsg::GetParticipants { page, limit } => to_binary(&query_participants(deps, page, limit)?),
        QueryMsg::GetParticipant { user } => to_binary(&query_participant(deps, user)?),
        QueryMsg::GetSaleStatus { } => to_binary( &query_sale_status(deps)? ),
    }
}

fn query_count(deps: Deps) -> StdResult<ParticipantsCountResponse> {
    let state: State = STATE.load(deps.storage)?;
    Ok(ParticipantsCountResponse { count: state.userlist.len() as u64 })
}

fn query_participants(deps: Deps, page: u64, limit: u64) -> StdResult<GetParticipantsResponse> {
    let state: State = STATE.load(deps.storage)?;

    let start = (page * limit) as usize;
    let end = (page * limit + limit) as usize;

    Ok(GetParticipantsResponse { participants: state.userlist[start..end].to_vec() })
}

fn query_participant(deps: Deps, user: String) -> StdResult<GetParticipantResponse> {
    Ok(GetParticipantResponse { data: PARTICIPANTS.load(deps.storage, user)? })
}

fn query_sale_status(deps: Deps) -> StdResult<GetSaleStatusResponse> {
    let state: State = STATE.load(deps.storage)?;
    Ok(GetSaleStatusResponse { private_sold_amount: state.private_sold_amount, public_sold_amount: state.public_sold_amount })
}


mod tests {
    use cosmwasm_std::{Uint128, testing::{mock_dependencies, mock_info, mock_env}, from_binary, Timestamp};
    use super::*;

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
}
