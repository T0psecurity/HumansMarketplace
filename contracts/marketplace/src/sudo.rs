use crate::error::ContractError;
use crate::helpers::{map_validate, ExpiryRange};
use crate::msg::SudoMsg;
use crate::state::{ASK_CREATED_HOOKS, SALE_HOOKS, SUDO_PARAMS};
use cosmwasm_std::{entry_point, Addr, Decimal, DepsMut, Env};
use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        SudoMsg::UpdateParams {
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
            operators,
            max_finders_fee_bps,
        } => sudo_update_params(
            deps,
            env,
            trading_fee_bps,
            ask_expiry,
            bid_expiry,
            operators,
            max_finders_fee_bps,
        ),
        SudoMsg::AddSaleHook { hook } => sudo_add_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::AddAskCreatedHook { hook } => {
            sudo_add_ask_hook(deps, env, api.addr_validate(&hook)?)
        }
        SudoMsg::RemoveSaleHook { hook } => sudo_remove_sale_hook(deps, api.addr_validate(&hook)?),
        SudoMsg::RemoveAskCreatedHook { hook } => {
            sudo_remove_ask_hook(deps, api.addr_validate(&hook)?)
        }
    }
}

/// Only governance can update contract params
pub fn sudo_update_params(
    deps: DepsMut,
    _env: Env,
    trading_fee_bps: Option<u64>,
    ask_expiry: Option<ExpiryRange>,
    bid_expiry: Option<ExpiryRange>,
    operators: Option<Vec<String>>,
    max_finders_fee_bps: Option<u64>,
) -> Result<Response, ContractError> {
    let mut params = SUDO_PARAMS.load(deps.storage)?;

    ask_expiry.as_ref().map(|a| a.validate()).transpose()?;
    bid_expiry.as_ref().map(|b| b.validate()).transpose()?;

    params.trading_fee_percent = trading_fee_bps
        .map(Decimal::percent)
        .unwrap_or(params.trading_fee_percent);
    params.ask_expiry = ask_expiry.unwrap_or(params.ask_expiry);
    params.bid_expiry = bid_expiry.unwrap_or(params.bid_expiry);
    if let Some(operators) = operators {
        params.operators = map_validate(deps.api, &operators)?;
    }
    params.max_finders_fee_percent = max_finders_fee_bps
        .map(Decimal::percent)
        .unwrap_or(params.max_finders_fee_percent);
    SUDO_PARAMS.save(deps.storage, &params)?;

    Ok(Response::new().add_attribute("action", "update_params"))
}

pub fn sudo_add_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_sale_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_add_ask_hook(deps: DepsMut, _env: Env, hook: Addr) -> Result<Response, ContractError> {
    ASK_CREATED_HOOKS.add_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "add_ask_created_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_remove_sale_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    SALE_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_sale_hook")
        .add_attribute("hook", hook);
    Ok(res)
}

pub fn sudo_remove_ask_hook(deps: DepsMut, hook: Addr) -> Result<Response, ContractError> {
    ASK_CREATED_HOOKS.remove_hook(deps.storage, hook.clone())?;

    let res = Response::new()
        .add_attribute("action", "remove_ask_created_hook")
        .add_attribute("hook", hook);
    Ok(res)
}
