use crate::state::{Ask, Bid};
use cosmwasm_std::{Addr, Coin};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SetBid {
        collection: String,
        token_id: u32,
    },
    RemoveBid {
        collection: String,
        token_id: u32,
    },
    SetAsk {
        collection: String,
        token_id: u32,
        price: Coin,
        funds_recipient: Option<String>,
    },
    RemoveAsk {
        collection: String,
        token_id: u32,
    },
    AcceptBid {
        collection: String,
        token_id: u32,
        bidder: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    CurrentAsk {
        collection: String,
        token_id: u32,
    },
    Asks {
        collection: String,
        start_after: Option<u32>,
        limit: Option<u32>,
    },
    AllAsks {},
    Bid {
        collection: String,
        token_id: u32,
        bidder: String,
    },
    Bids {
        collection: String,
        token_id: u32,
        start_after: Option<String>,
        limit: Option<u32>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidInfo {
    pub token_id: u32,
    pub price: Coin,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskInfo {
    pub token_id: u32,
    pub price: Coin,
    pub funds_recipient: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllAskInfo {
    pub collection: String,
    pub token_id: u32,
    pub price: Coin,
    pub funds_recipient: Option<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CurrentAskResponse {
    pub ask: Option<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AsksResponse {
    pub asks: Vec<AskInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AllAsksResponse {
    pub asks: Vec<AllAskInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse {
    pub bid: Option<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidsResponse {
    pub bids: Vec<BidInfo>,
}
