use std::convert::TryInto;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, CosmosMsg, WasmMsg, Uint128, WasmQuery, QueryRequest, attr, BankMsg, Coin};
use cw20::{Cw20ExecuteMsg, Cw20QueryMsg, BalanceResponse };
use sha2::Digest;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, ParticipantsCountResponse, GetParticipantResponse, GetParticipantsResponse, GetSaleStatusResponse, MigrateMsg};
use crate::querier::{query_decimals, query_balance};
use crate::state::{PARTICIPANTS, PRIVATE_SOLD_FUNDS, ACCURACY, State, Participant, AlloInfo, store_state, read_state};

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        owner: deps.api.addr_canonicalize(info.sender.as_str())?,
        fund_denom: msg.fund_denom,
        reward_token: deps.api.addr_canonicalize(msg.reward_token.as_str())?,
        vesting: deps.api.addr_canonicalize(msg.vesting.as_str())?,
        whitelist_merkle_root: msg.whitelist_merkle_root,

        exchange_rate: msg.exchange_rate,
        presale_period: msg.presale_period,
        public_start_time: msg.public_start_time,
        private_start_time: msg.private_start_time,

        private_sold_amount: Uint128::zero(),
        public_sold_amount: Uint128::zero(),
        userlist: vec![],
    };

    store_state(deps.storage, &state)?;

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
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::TransferOwnerShip {
            new_owner
        } => execute_transfer_ownership(deps, info, new_owner),

        ExecuteMsg::SetMerkleRoot { merkle_root } => execute_set_whitelist_merkle_root(deps, info, merkle_root),

        ExecuteMsg::UpdatePresaleInfo {
            new_private_start_time,
            new_public_start_time,
            new_presale_period
        } => execute_update_info(deps, info, new_private_start_time, new_public_start_time, new_presale_period),

        ExecuteMsg::Deposit { allo_info, proof } => execute_deposit(deps, _env, info, allo_info, proof),

        ExecuteMsg::DepositPrivateSale { allo_info, proof } => execute_deposit_private_sale(deps, _env, info, allo_info, proof),

        ExecuteMsg::WithdrawFunds { receiver } => execute_withdraw_funds(deps, _env, info, receiver),

        ExecuteMsg::WithdrawUnsoldToken { receiver } => execute_withdraw_unsold_token(deps, _env, info, receiver),
    }
}

pub fn execute_transfer_ownership(deps: DepsMut, info: MessageInfo, new_owner: String) -> Result<Response, ContractError> {
    let new_owner_canoncial = deps.api.addr_canonicalize(new_owner.as_str())?;
    let mut state: State = read_state(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    state.owner = new_owner_canoncial;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "transfer_ownership"),
        attr("owner", new_owner),
    ]))
}

pub fn execute_set_whitelist_merkle_root(deps: DepsMut, info: MessageInfo, merkle_root: String) -> Result<Response, ContractError> {
    let mut state: State = read_state(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    state.whitelist_merkle_root = merkle_root.clone();
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "register_merkle_root"),
        attr("merkle_root", merkle_root),
    ]))
}

pub fn execute_update_info(deps: DepsMut, info: MessageInfo, new_private_start_time: u64, new_public_start_time: u64, new_presale_period: u64) -> Result<Response, ContractError> {
    let mut state: State = read_state(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    state.private_start_time = new_private_start_time;
    state.public_start_time = new_public_start_time;
    state.presale_period = new_presale_period;
    store_state(deps.storage, &state)?;

    Ok(Response::new().add_attributes(vec![
        attr("action", "set_start_time"),
        attr("new_private_start_time", new_private_start_time.to_string()),
        attr("new_public_start_time", new_public_start_time.to_string()),
        attr("new_presale_period", new_presale_period.to_string()),
    ]))
}

pub fn calc_reward_amount(deps: Deps, state: State, fund_amount: Uint128) -> StdResult<Uint128> {
    let fund_decimals: u32 = 6;
    let reward_decimals = query_decimals(deps, deps.api.addr_humanize(&state.reward_token)?.to_string())?;

    Ok(fund_amount
        .checked_mul(Uint128::from(ACCURACY)).unwrap()
        .checked_div(state.exchange_rate).unwrap()
        .checked_mul(Uint128::from(10u128).wrapping_pow(reward_decimals)).unwrap()
        .checked_div(Uint128::from(10u128).wrapping_pow(fund_decimals)).unwrap()
    )
}

pub fn verify_whitelist(state: State, sender: &String, allo_info: &AlloInfo, proof: &Vec<String>) -> Result<bool, ContractError> {
    let user_input = format!("{}{}{}", sender, allo_info.private_allocation, allo_info.public_allocation);
    let hash = sha2::Sha256::digest(user_input.as_bytes())
        .as_slice()
        .try_into()
        .map_err(|_| ContractError::WrongLength {})?;

    let hash = proof.into_iter().try_fold(hash, |hash, p| {
        let mut proof_buf = [0; 32];
        hex::decode_to_slice(p, &mut proof_buf)?;
        let mut hashes = [hash, proof_buf];
        hashes.sort_unstable();
        sha2::Sha256::digest(&hashes.concat())
            .as_slice()
            .try_into()
            .map_err(|_| ContractError::WrongLength {})
    })?;

    let mut root_buf: [u8; 32] = [0; 32];
    hex::decode_to_slice(state.whitelist_merkle_root.clone(), &mut root_buf)?;
    Ok(root_buf == hash)
}

pub fn execute_deposit(deps: DepsMut, env: Env, info: MessageInfo, allo_info: AlloInfo, proof: Vec<String>) -> Result<Response, ContractError> {
    let mut state: State = read_state(deps.storage)?;
    let sender = info.sender.to_string();

    /* Check if Presale in progress */
    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() > end_time || env.block.time.seconds() < state.public_start_time {
        return Err(ContractError::PublicNotInProgress {});
    }

    /* Check fund tokens */
    let amount;
    if let Some(coins) = info.funds.first() {
        if coins.denom != state.fund_denom || coins.amount.is_zero() {
            return Err(ContractError::Funds {  });
        }
        amount = coins.amount;
    } else {
        return Err(ContractError::Funds {  });
    }

    /* Verify if whitelisted */
    if state.whitelist_merkle_root.len() > 0 {
        if verify_whitelist(state.clone(), &sender, &allo_info, &proof)? == false {
            return Err(ContractError::NotWhitelisted {});
        }
    }



    let mut recp_info = Participant {
        fund_balance: Uint128::zero(),
        reward_balance: Uint128::zero()
    };
    let mut private_sold_fund = Uint128::zero();

    /* Add to participants list */
    if PARTICIPANTS.has(deps.storage, sender.clone()) {
        recp_info = PARTICIPANTS.load(deps.storage, sender.clone())?;
        private_sold_fund = PRIVATE_SOLD_FUNDS.load(deps.storage, sender.clone())?;
    } else {
        state.userlist.push(sender.clone());
    }

    /* Check allocation */
    let new_fund_balance = recp_info.fund_balance + amount;
    if allo_info.public_allocation + private_sold_fund < new_fund_balance {
        return Err(ContractError::ExceedAllocation {  });
    }

    /* Update rewards amount */
    let reward_amount = calc_reward_amount(deps.as_ref(), state.clone(), amount)?;
    recp_info.fund_balance = new_fund_balance;
    recp_info.reward_balance = recp_info.reward_balance + reward_amount;
    state.public_sold_amount = state.public_sold_amount + reward_amount;

    store_state(deps.storage, &state)?;
    PARTICIPANTS.save(deps.storage, sender.clone(), &recp_info)?;

    /* Update vesting */
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.vesting)?.to_string(),
        msg: to_binary(&vesting::msg::ExecuteMsg::UpdateRecipient {
            recp: sender.clone(),
            amount: recp_info.reward_balance.u128().try_into().unwrap(),
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "deposit"))
}

pub fn execute_deposit_private_sale(deps: DepsMut, env: Env, info: MessageInfo, allo_info: AlloInfo, proof: Vec<String>) -> Result<Response, ContractError> {
    let mut state: State = read_state(deps.storage)?;
    let sender = info.sender.to_string();

    /* Check if Presale in progress */
    if env.block.time.seconds() < state.private_start_time {
        return Err(ContractError::PrivateNotInProgress {  });
    }

    /* Check fund tokens */
    let amount ;
    if let Some(coins) = info.funds.first() {
        if coins.denom != state.fund_denom || coins.amount.is_zero() {
            return Err(ContractError::Funds {  });
        }
        amount = coins.amount;
    } else {
        return Err(ContractError::Funds {  });
    }

    /* Verify if whitelisted */
    if state.whitelist_merkle_root.len() > 0 {
        if verify_whitelist(state.clone(), &sender, &allo_info, &proof)? == false {
            return Err(ContractError::NotWhitelisted {});
        }
    }


    let mut recp_info = Participant {
        fund_balance: Uint128::zero(),
        reward_balance: Uint128::zero()
    };
    let mut private_sold_fund = Uint128::zero();

    /* Add to participants list */
    if PARTICIPANTS.has(deps.storage, sender.clone()) {
        recp_info = PARTICIPANTS.load(deps.storage, sender.clone())?;
        private_sold_fund = PRIVATE_SOLD_FUNDS.load(deps.storage, sender.clone())?;
    } else {
        state.userlist.push(sender.clone());
    }

    /* Check allocation */
    let new_fund_balance = recp_info.fund_balance + amount;
    if allo_info.private_allocation < new_fund_balance {
        return Err(ContractError::ExceedAllocation {  });
    }

    /* Update rewards amount */
    let reward_amount = calc_reward_amount(deps.as_ref(), state.clone(), amount)?;
    recp_info.fund_balance = new_fund_balance;
    recp_info.reward_balance = recp_info.reward_balance + reward_amount;
    state.private_sold_amount = state.private_sold_amount + reward_amount;
    private_sold_fund = private_sold_fund + amount;

    store_state(deps.storage, &state)?;
    PARTICIPANTS.save(deps.storage, sender.clone(), &recp_info)?;
    PRIVATE_SOLD_FUNDS.save(deps.storage, sender.clone(), &private_sold_fund)?;

    /* Update vesting */
    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.vesting)?.to_string(),
        msg: to_binary(&vesting::msg::ExecuteMsg::UpdateRecipient {
            recp: sender.clone(),
            amount: recp_info.reward_balance.u128().try_into().unwrap(),
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "deposit_private"))
}

pub fn execute_withdraw_funds(deps: DepsMut, env: Env, info: MessageInfo, receiver: String) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() <= end_time {
        return Err(ContractError::StillInProgress {  });
    }

    let fund_balance = query_balance(deps.as_ref(), env.contract.address, state.fund_denom.clone())?;

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Bank(BankMsg::Send {
        to_address: receiver,
        amount: vec![Coin {
            denom: state.fund_denom.to_string(),
            amount: fund_balance.clone(),
        }]
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "withdraw_funds"))
}

pub fn execute_withdraw_unsold_token(deps: DepsMut, env: Env, info: MessageInfo, receiver: String) -> Result<Response, ContractError> {
    let state: State = read_state(deps.storage)?;

    // permission check
    if deps.api.addr_canonicalize(info.sender.as_str())? != state.owner {
        return Err(ContractError::Unauthorized {});
    }

    let end_time = state.public_start_time + state.presale_period;
    if env.block.time.seconds() <= end_time {
        return Err(ContractError::StillInProgress {  });
    }

    let reward_balance_info: BalanceResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: deps.api.addr_humanize(&state.reward_token)?.to_string(),
        msg: to_binary(&Cw20QueryMsg::Balance {
            address: deps.api.addr_humanize(&state.vesting)?.to_string()
        })?,
    }))?;

    let sold_amount = state.private_sold_amount + state.public_sold_amount;
    let unsold_amount = reward_balance_info.balance - Uint128::from(sold_amount);

    let mut messages: Vec<CosmosMsg> = vec![];
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: deps.api.addr_humanize(&state.reward_token)?.to_string(),
        msg: to_binary(&Cw20ExecuteMsg::TransferFrom {
            owner: deps.api.addr_humanize(&state.vesting)?.to_string(),
            recipient: receiver.clone(),
            amount: unsold_amount,
        })?,
        funds: vec![],
    }));
    Ok(Response::new()
        .add_messages(messages)
        .add_attribute("method", "withdraw_unsold_token"))
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
    let state: State = read_state(deps.storage)?;
    Ok(ParticipantsCountResponse { count: state.userlist.len() as u64 })
}

fn query_participants(deps: Deps, page: u64, limit: u64) -> StdResult<GetParticipantsResponse> {
    let state: State = read_state(deps.storage)?;

    let start = (page * limit) as usize;
    let end = (page * limit + limit) as usize;

    Ok(GetParticipantsResponse { participants: state.userlist[start..end].to_vec() })
}

fn query_participant(deps: Deps, user: String) -> StdResult<GetParticipantResponse> {
    let data = PARTICIPANTS.load(deps.storage, user).unwrap_or(Participant { fund_balance: Uint128::zero(), reward_balance: Uint128::zero() });
    Ok(GetParticipantResponse { data })
}

fn query_sale_status(deps: Deps) -> StdResult<GetSaleStatusResponse> {
    let state: State = read_state(deps.storage)?;
    Ok(GetSaleStatusResponse { private_sold_amount: state.private_sold_amount, public_sold_amount: state.public_sold_amount })
}

