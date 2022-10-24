use crate::error::ContractError;
use crate::helpers::{map_validate, ExpiryRange};
use crate::msg::{
    AskHookMsg, BidHookMsg, ExecuteMsg, HookAction, InstantiateMsg,
    SaleHookMsg, NftInfoResponse
};
use crate::state::{
    ask_key, asks, bid_key, bids, Ask, Bid, Order, SaleType, SudoParams, TokenId, ASK_HOOKS, BID_HOOKS, SALE_HOOKS,
    SUDO_PARAMS
};
use cw721_base::Metadata;

#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    coin, to_binary, Addr, BankMsg, Coin, Decimal, Deps, DepsMut, Empty, Env, Event, MessageInfo,
    Reply, StdError, StdResult, Storage, Timestamp, Uint128, WasmMsg, Response, SubMsg, from_binary
};
use cw2::set_contract_version;
use cw721_base::ExecuteMsg as Cw721ExecuteMsg;
use cw721_base::QueryMsg as Cw721QueryMsg;
use cw721_base::CollectionInfoResponse;
use cw721::{Cw721ReceiveMsg};
use cw_storage_plus::Item;
use cw_utils::{may_pay, must_pay, nonpayable, Duration};
use schemars::JsonSchema;
use semver::Version;
use serde::{Deserialize, Serialize};
// use sg1::fair_burn;

pub const NATIVE_DENOM: &str = "uheart";

// Version info for migration info
const CONTRACT_NAME: &str = "crates.io:human-marketplace";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    msg.ask_expiry.validate()?;
    msg.bid_expiry.validate()?;

    let params = SudoParams {
        // trading_fee_percent: Decimal::percent(msg.trading_fee_bps),
        ask_expiry: msg.ask_expiry,
        bid_expiry: msg.bid_expiry,
        operators: map_validate(deps.api, &msg.operators)?,
        // max_finders_fee_percent: Decimal::percent(msg.max_finders_fee_bps),
        min_price: msg.min_price,
        listing_fee: msg.listing_fee,
    };
    SUDO_PARAMS.save(deps.storage, &params)?;

    if let Some(hook) = msg.sale_hook {
        SALE_HOOKS.add_hook(deps.storage, deps.api.addr_validate(&hook)?)?;
    }

    Ok(Response::new())
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct AskInfo {
    pub sale_type: SaleType,
    pub collection: Addr,
    pub token_id: TokenId,
    pub price: Coin,
    pub funds_recipient: Option<Addr>,
    pub expires: u64,
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

pub struct BidInfo {
    collection: Addr,
    token_id: TokenId,
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    let api = deps.api;

    match msg {
        ExecuteMsg::ReceiveNft(rcv_msg) => execute_set_ask(deps, env, info, rcv_msg),
        
        ExecuteMsg::RemoveAsk {
            collection,
            token_id,
        } => execute_remove_ask(deps, info, api.addr_validate(&collection)?, token_id),
        ExecuteMsg::SetBid {
            collection,
            token_id,
        } => execute_set_bid(
            deps,
            env,
            info,
            BidInfo {
                collection: api.addr_validate(&collection)?,
                token_id,
            },
        ),
        // ExecuteMsg::RemoveBid {
        //     collection,
        //     token_id,
        // } => execute_remove_bid(deps, env, info, api.addr_validate(&collection)?, token_id),
        ExecuteMsg::AcceptBid {
            collection,
            token_id,
        } => execute_accept_bid(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            token_id,
        ),
        ExecuteMsg::UpdateAskPrice {
            collection,
            token_id,
            price,
        } => execute_update_ask_price(
            deps,
            env,
            info,
            api.addr_validate(&collection)?,
            token_id,
            price,
        ),
    }
}

/// A seller may set an Ask on their NFT to list it on Marketplace
pub fn execute_set_ask(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    rcv_msg: Cw721ReceiveMsg,
) -> Result<Response, ContractError> {
    let ask_info: AskInfo = from_binary(&rcv_msg.msg)?;
    let collection_address = info.clone().sender;
    
    let AskInfo {
        sale_type,
        collection,
        token_id,
        price,
        funds_recipient,
        expires,
    } = ask_info.clone();

    if rcv_msg.token_id != token_id {
        return Err(ContractError::IdMismatch{});
    }

    if ask_info.collection != collection_address {
        return Err(ContractError::CollectionMismatch{} );
    }

    price_validate(deps.storage, &price)?;

    let params = SUDO_PARAMS.load(deps.storage)?;
    params.ask_expiry.is_valid(expires)?;

    // Check if msg has correct listing fee
    let listing_fee = may_pay(&info, NATIVE_DENOM)?;
    if listing_fee != params.listing_fee {
        return Err(ContractError::InvalidListingFee(listing_fee));
    }

    let now = env.block.time;

    let nft_info: NftInfoResponse<Metadata> = deps
        .querier
        .query_wasm_smart(collection.clone(), &Cw721QueryMsg::NftInfo { token_id: token_id.clone() })?;

    let seller = deps.api.addr_validate(rcv_msg.sender.as_str())?;
    
    let ask = Ask {
        sale_type,
        collection: collection.clone(),
        token_id: token_id.clone(),
        img_url: nft_info.extension.image_url,
        seller: deps.api.addr_validate(rcv_msg.sender.as_str())?,
        price: price.amount,
        funds_recipient,
        expires_at: now.plus_seconds(expires),
        max_bidder: Some(env.contract.address.clone()),
        max_bid: Some(params.min_price),
        bid_count: Uint128::zero(),
        content_type: nft_info.content_type
    };
    store_ask(deps.storage, &ask)?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Create)?;

    let event = Event::new("set-ask")
        .add_attribute("action", "human_marketplace_set_ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("seller", seller)
        .add_attribute("price", price.to_string())
        .add_attribute("expires", expires.to_string());

    let res = Response::new();


    Ok(res
        .add_submessages(hook).add_event(event))
}

/// Removes the ask on a particular NFT
pub fn execute_remove_ask(
    deps: DepsMut,
    info: MessageInfo,
    collection: Addr,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let key = ask_key(&collection, &token_id);
    let ask = asks().load(deps.storage, key.clone())?;

    let owner = ask.clone().seller;
    only_owner_nft(&info, owner)?;

    if ask.sale_type == SaleType::Auction {
        return Err(ContractError::AuctionNotRemove {});
    }

    asks().remove(deps.storage, key)?;

    let cw721_transfer_msg = Cw721ExecuteMsg::<Metadata>::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: ask.seller.to_string(),
    };

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: ask.collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Delete)?;

    let event = Event::new("remove-ask")
        .add_attribute("action", "human_marketplace_remove_ask")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string());

    Ok(Response::new().add_event(event)
        .add_message(exec_cw721_transfer)
        .add_submessages(hook))
}

/// Updates the ask price on a particular NFT
pub fn execute_update_ask_price(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: TokenId,
    price: Coin,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;
    price_validate(deps.storage, &price)?;

    let key = ask_key(&collection, &token_id);

    let mut ask = asks().load(deps.storage, key.clone())?;

    only_owner_nft(&info, ask.clone().seller)?;

    if ask.is_expired(&env.block) {
        return Err(ContractError::AskExpired {});
    }

    ask.price = price.amount;
    asks().save(deps.storage, key, &ask)?;

    let hook = prepare_ask_hook(deps.as_ref(), &ask, HookAction::Update)?;

    let event = Event::new("update-ask")
        .add_attribute("action", "human_marketplace_update_ask_price")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("price", price.to_string());

    Ok(Response::new().add_event(event).add_submessages(hook))
}

/// Places a bid on a listed or unlisted NFT. The bid is escrowed in the contract.
pub fn execute_set_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    bid_info: BidInfo,
) -> Result<Response, ContractError> {
    let BidInfo {
        collection,
        token_id,
    } = bid_info;
    let params = SUDO_PARAMS.load(deps.storage)?;

    let bid_price = must_pay(&info, NATIVE_DENOM)?;
    if bid_price < params.min_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }

    let bidder = info.sender.clone();
    let mut res = Response::new();
    let ask_key = ask_key(&collection, &token_id);
    let current_bid_key = bid_key(&collection, &token_id, &bidder);

    let existing_ask = asks().may_load(deps.storage, ask_key.clone())?;

    // if there is no ask
    // return an error
    if existing_ask.is_none() {
        return Err(ContractError::AskNotFound {});
    }

    let mut ask = existing_ask.unwrap();

    if ask.is_expired(&env.block) {
        return Err(ContractError::AskExpired {});
    }

    // If the bid price is lower than the required one, it fails
    if ask.sale_type == SaleType::Auction && ask.price > bid_price {
        return Err(ContractError::PriceTooSmall(bid_price));
    }
   

    let existing_bid = bids().may_load(deps.storage, current_bid_key.clone())?;
    if existing_bid.is_some() {
        bids().remove(deps.storage, current_bid_key)?;
    }
    else{
       if ask.sale_type == SaleType::Auction {
            ask.bid_count = ask.bid_count + Uint128::new(1);
            asks().save(deps.storage, ask_key.clone(), &ask)?;
       }
    }

    
    let save_bid = |store| -> StdResult<_> {
        let bid = Bid::new(
            collection.clone(),
            token_id.clone(),
            bidder.clone(),
            bid_price,
            true,
            env.block.time
        );
        store_bid(store, &bid)?;
        Ok(Some(bid))
    };

    let mut action = String::new();

    let bid = match ask.sale_type {
        SaleType::FixedPrice => {
            if ask.price != bid_price {
                return Err(ContractError::InvalidPrice {});
            }
            action = "human_marketplace_buy_fixed_price".to_string();
            asks().remove(deps.storage, ask_key)?;
            finalize_sale(
                deps.as_ref(),
                ask,
                bid_price,
                bidder.clone(),
                // finder,
                &mut res,
            )?;
            None
        },
        SaleType::Auction => {
            if ask.max_bid.is_none() || ask.max_bidder.is_none() {
                return Err(ContractError::WrongAskInfo {});
            }

            if bid_price <= ask.max_bid.unwrap() {
                return Err(ContractError::InsufficientFundsSend {});
            }

            action = "human_marketplace_set_bid".to_string();

            let max_bidder = ask.max_bidder.unwrap();

            let refund_msg = BankMsg::Send {
                to_address: max_bidder.to_string(),
                amount: vec![coin(ask.max_bid.unwrap().u128(), NATIVE_DENOM)],
            };

            if max_bidder != env.contract.address {
                res = res.add_message(refund_msg);
                
                let prev_bid_key = bid_key(&collection, &token_id, &max_bidder);
                let prev_bid = bids().may_load(deps.storage, prev_bid_key.clone())?;

                if let Some(mut prev_bid) = prev_bid {
                    prev_bid.active = false;
                    bids().save(deps.storage, prev_bid_key, &prev_bid)?;
                }
            }
                
            ask.max_bid = Some(bid_price);
            ask.max_bidder = Some(info.sender);
            asks().save(deps.storage, ask_key, &ask)?;

            save_bid(deps.storage)?
        }
    };

    let hook = if let Some(bid) = bid {
        prepare_bid_hook(deps.as_ref(), &bid, HookAction::Create)?
    } else {
        vec![]
    };

    let event = Event::new("set-bid")
        .add_attribute("action", action)
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("bidder", bidder)
        .add_attribute("bid_price", bid_price.to_string());

    Ok(res.add_submessages(hook).add_event(event))
}

/// Removes a bid made by the bidder. Bidders can only remove their own bids
// pub fn execute_remove_bid(
//     deps: DepsMut,
//     _env: Env,
//     info: MessageInfo,
//     collection: Addr,
//     token_id: TokenId,
// ) -> Result<Response, ContractError> {
//     nonpayable(&info)?;
//     let bidder = info.sender;

//     let key = bid_key(&collection, &token_id, &bidder);
//     let bid = bids().load(deps.storage, key.clone())?;
//     bids().remove(deps.storage, key)?;

//     let refund_bidder_msg = BankMsg::Send {
//         to_address: bid.bidder.to_string(),
//         amount: vec![coin(bid.price.u128(), NATIVE_DENOM)],
//     };

//     let hook = prepare_bid_hook(deps.as_ref(), &bid, HookAction::Delete)?;

//     let event = Event::new("remove-bid")
//         .add_attribute("collection", collection)
//         .add_attribute("token_id", token_id.to_string())
//         .add_attribute("bidder", bidder);

//     let res = Response::new()
//         .add_message(refund_bidder_msg)
//         .add_event(event)
//         .add_submessages(hook);

//     Ok(res)
// }

// Seller can accept a bid which transfers funds as well as the token. The bid may or may not be associated with an ask.
pub fn execute_accept_bid(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    collection: Addr,
    token_id: TokenId,
) -> Result<Response, ContractError> {
    nonpayable(&info)?;

    let ask_key = ask_key(&collection, &token_id);
    let existing_ask = asks().may_load(deps.storage, ask_key.clone())?.unwrap();

    only_owner_nft(&info, existing_ask.clone().seller)?;

    if !existing_ask.is_expired(&env.block) {
        return Err(ContractError::AuctionNotEnded {});
    }

    asks().remove(deps.storage, ask_key)?;
 

    let mut res = Response::new();

    let max_bidder = existing_ask.clone().max_bidder.unwrap();
    let max_bid_price = existing_ask.clone().max_bid.unwrap();

    if max_bidder != env.contract.address {
        finalize_sale(
            deps.as_ref(),
            existing_ask.clone(),
            max_bid_price,
            max_bidder.clone(),
            // finder,
            &mut res,
        )?;
    } else {
        let cw721_transfer_msg = Cw721ExecuteMsg::<Metadata>::TransferNft {
            token_id: token_id.to_string(),
            recipient: existing_ask.seller.to_string(),
        };
    
        let exec_cw721_transfer = WasmMsg::Execute {
            contract_addr: collection.to_string(),
            msg: to_binary(&cw721_transfer_msg)?,
            funds: vec![],
        };

        res.clone().add_message(exec_cw721_transfer);
    }

    let event = Event::new("accept-bid")
        .add_attribute("action", "human_marketplace_accept_bid")
        .add_attribute("collection", collection.to_string())
        .add_attribute("token_id", token_id.to_string())
        .add_attribute("buyer", max_bidder);

    Ok(res.add_event(event))
}
/// Transfers funds and NFT, updates bid
fn finalize_sale(
    deps: Deps,
    ask: Ask,
    price: Uint128,
    buyer: Addr,
    res: &mut Response,
) -> StdResult<()> {
    payout(
        deps,
        ask.collection.clone(),
        price,
        ask.funds_recipient
            .clone()
            .unwrap_or_else(|| ask.seller.clone()),
        res,
    )?;

    let cw721_transfer_msg = Cw721ExecuteMsg::<Metadata>::TransferNft {
        token_id: ask.token_id.to_string(),
        recipient: buyer.to_string(),
    };

    let exec_cw721_transfer = WasmMsg::Execute {
        contract_addr: ask.collection.to_string(),
        msg: to_binary(&cw721_transfer_msg)?,
        funds: vec![],
    };
    res.messages.push(SubMsg::new(exec_cw721_transfer));

    res.messages
        .append(&mut prepare_sale_hook(deps, &ask, buyer.clone())?);

    let event = Event::new("finalize-sale")
        .add_attribute("collection", ask.collection.to_string())
        .add_attribute("token_id", ask.token_id.to_string())
        .add_attribute("seller", ask.seller.to_string())
        .add_attribute("buyer", buyer.to_string())
        .add_attribute("price", price.to_string());
    res.events.push(event);

    Ok(())
}

/// Payout a bid
fn payout(
    deps: Deps,
    collection: Addr,
    payment: Uint128,
    payment_recipient: Addr,
    // finder: Option<Addr>,
    // finders_fee_bps: Option<u64>,
    res: &mut Response,
) -> StdResult<()> {
    // let params = SUDO_PARAMS.load(deps.storage)?;

    // Append Fair Burn message
    // let network_fee = payment * params.trading_fee_percent / Uint128::from(100u128);
    // fair_burn(network_fee.u128(), None, res);

    let collection_info: CollectionInfoResponse = deps
        .querier
        .query_wasm_smart(collection.clone(), &Cw721QueryMsg::GetCollectionState  {})?;


    match collection_info.royalty_info {
        // If token supports royalities, payout shares to royalty recipient
        Some(royalty) => {
            let amount = coin((payment * royalty.royalty_rate).u128(), NATIVE_DENOM);
            if payment < amount.amount {
                return Err(StdError::generic_err("Fees exceed payment"));
            }
            res.messages.push(SubMsg::new(BankMsg::Send {
                to_address: royalty.address.to_string(),
                amount: vec![amount.clone()],
            }));

            let event = Event::new("royalty-payout")
                .add_attribute("collection", collection.to_string())
                .add_attribute("amount", amount.to_string())
                .add_attribute("recipient", royalty.address.to_string());
            res.events.push(event);

            let seller_share_msg = BankMsg::Send {
                to_address: payment_recipient.to_string(),
                amount: vec![coin(
                    (payment * (Decimal::one() - royalty.royalty_rate)).u128(),
                    NATIVE_DENOM.to_string(),
                )],
            };
            res.messages.push(SubMsg::new(seller_share_msg));
        }
        None => {
            // if payment < network_fee {
            //     return Err(StdError::generic_err("Fees exceed payment"));
            // }
            // If token doesn't support royalties, pay seller in full
            let seller_share_msg = BankMsg::Send {
                to_address: payment_recipient.to_string(),
                amount: vec![coin(
                    payment.u128(),
                    NATIVE_DENOM.to_string(),
                )],
            };
            res.messages.push(SubMsg::new(seller_share_msg));
        }
    }

    Ok(())
}

fn price_validate(store: &dyn Storage, price: &Coin) -> Result<(), ContractError> {
    if price.amount.is_zero() || price.denom != NATIVE_DENOM {
        return Err(ContractError::InvalidPrice {});
    }

    if price.amount < SUDO_PARAMS.load(store)?.min_price {
        return Err(ContractError::PriceTooSmall(price.amount));
    }

    Ok(())
}

fn store_bid(store: &mut dyn Storage, bid: &Bid) -> StdResult<()> {
    bids().save(
        store,
        bid_key(&bid.collection, &bid.token_id, &bid.bidder),
        bid,
    )
}

fn store_ask(store: &mut dyn Storage, ask: &Ask) -> StdResult<()> {
    asks().save(store, ask_key(&ask.collection, &ask.token_id), ask)
}

/// Checks to enfore only NFT owner can call
fn only_owner_nft(
    info: &MessageInfo,
    owner: Addr,
) -> Result<Response, ContractError> {
    if owner != info.sender {
        return Err(ContractError::UnauthorizedOwner {});
    }

    Ok(Response::default())
}

/// Checks to enforce only privileged operators
fn only_operator(store: &dyn Storage, info: &MessageInfo) -> Result<Addr, ContractError> {
    let params = SUDO_PARAMS.load(store)?;
    if !params
        .operators
        .iter()
        .any(|a| a.as_ref() == info.sender.as_ref())
    {
        return Err(ContractError::UnauthorizedOperator {});
    }

    Ok(info.sender.clone())
}

enum HookReply {
    Ask = 1,
    Sale,
    Bid,
    CollectionBid,
}

impl From<u64> for HookReply {
    fn from(item: u64) -> Self {
        match item {
            1 => HookReply::Ask,
            2 => HookReply::Sale,
            3 => HookReply::Bid,
            4 => HookReply::CollectionBid,
            _ => panic!("invalid reply type"),
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> Result<Response, ContractError> {
    match HookReply::from(msg.id) {
        HookReply::Ask => {
            let res = Response::new()
                .add_attribute("action", "ask-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Sale => {
            let res = Response::new()
                .add_attribute("action", "sale-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::Bid => {
            let res = Response::new()
                .add_attribute("action", "bid-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
        HookReply::CollectionBid => {
            let res = Response::new()
                .add_attribute("action", "collection-bid-hook-failed")
                .add_attribute("error", msg.result.unwrap_err());
            Ok(res)
        }
    }
}

fn prepare_ask_hook(deps: Deps, ask: &Ask, action: HookAction) -> StdResult<Vec<SubMsg>> {
    let submsgs = ASK_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = AskHookMsg { ask: ask.clone() };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(action.clone())?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Ask as u64))
    })?;

    Ok(submsgs)
}

fn prepare_sale_hook(deps: Deps, ask: &Ask, buyer: Addr) -> StdResult<Vec<SubMsg>> {
    let submsgs = SALE_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = SaleHookMsg {
            collection: ask.collection.to_string(),
            token_id: ask.token_id.to_string(),
            price: coin(ask.price.clone().u128(), NATIVE_DENOM),
            seller: ask.seller.to_string(),
            buyer: buyer.to_string(),
        };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary()?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Sale as u64))
    })?;

    Ok(submsgs)
}

fn prepare_bid_hook(deps: Deps, bid: &Bid, action: HookAction) -> StdResult<Vec<SubMsg>> {
    let submsgs = BID_HOOKS.prepare_hooks(deps.storage, |h| {
        let msg = BidHookMsg { bid: bid.clone() };
        let execute = WasmMsg::Execute {
            contract_addr: h.to_string(),
            msg: msg.into_binary(action.clone())?,
            funds: vec![],
        };
        Ok(SubMsg::reply_on_error(execute, HookReply::Bid as u64))
    })?;

    Ok(submsgs)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: Empty) -> Result<Response, ContractError> {
    let current_version = cw2::get_contract_version(deps.storage)?;
    if current_version.contract != CONTRACT_NAME {
        return Err(StdError::generic_err("Cannot upgrade to a different contract").into());
    }
    let version: Version = current_version
        .version
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;
    let new_version: Version = CONTRACT_VERSION
        .parse()
        .map_err(|_| StdError::generic_err("Invalid contract version"))?;

    if version > new_version {
        return Err(StdError::generic_err("Cannot upgrade to a previous contract version").into());
    }
    // if same version return
    if version == new_version {
        return Ok(Response::new());
    }

    // SudoParamsV015 represents the previous state from v0.15.0 version
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct SudoParamsV015 {
        // pub trading_fee_percent: Decimal,
        pub ask_expiry: ExpiryRange,
        pub bid_expiry: ExpiryRange,
        pub operators: Vec<Addr>,
        // pub max_finders_fee_percent: Decimal,
        pub min_price: Uint128,
        pub stale_bid_duration: Duration,
        pub bid_removal_reward_percent: Decimal,
    }

    // load state that contains the old struct type
    let params_item: Item<SudoParamsV015> = Item::new("sudo-params");
    let current_params = params_item.load(deps.storage)?;

    // migrate to the new struct
    let new_sudo_params = SudoParams {
        // trading_fee_percent: current_params.trading_fee_percent,
        ask_expiry: current_params.ask_expiry,
        bid_expiry: current_params.bid_expiry,
        operators: current_params.operators,
        // max_finders_fee_percent: current_params.max_finders_fee_percent,
        min_price: current_params.min_price,
        listing_fee: Uint128::zero(),
    };
    // store migrated params
    SUDO_PARAMS.save(deps.storage, &new_sudo_params)?;

    // set new contract version
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    Ok(Response::new())
}
