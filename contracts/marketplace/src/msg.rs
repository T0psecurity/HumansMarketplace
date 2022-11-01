use crate::{
    helpers::ExpiryRange,
    state::{Ask, Bid, CollectionBid, SudoParams, TokenId},
};
use cosmwasm_std::{to_binary, Addr, Binary, Coin, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw721::Cw721ReceiveMsg;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// Fair Burn fee for winning bids
    /// 0.25% = 25, 0.5% = 50, 1% = 100, 2.5% = 250
    pub trading_fee_bps: u64,
    /// Valid time range for Asks
    /// (min, max) in seconds
    pub ask_expiry: ExpiryRange,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// Operators are entites that are responsible for maintaining the active state of Asks.
    /// They listen to NFT transfer events, and update the active state of Asks.
    pub operators: Vec<String>,
    /// The address of the airdrop claim contract to detect sales
    pub sale_hook: Option<String>,
    /// Max basis points for the finders fee
    // pub max_finders_fee_bps: u64,
    /// Min value for bids and asks
    pub min_price: Uint128,
    /// Listing fee to reduce spam
    pub listing_fee: Uint128,

    pub create_collection_address: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ReceiveNft(Cw721ReceiveMsg),
    /// List an NFT on the marketplace by creating a new ask
    
    /// Remove an existing ask from the marketplace
    RemoveAsk {
        collection: String,
        token_id: TokenId,
    },
    /// Update the price of an existing ask
    UpdateAskPrice {
        collection: String,
        token_id: TokenId,
        price: Coin,
    },
    /// Place a bid on an existing ask
    SetBid {
        collection: String,
        token_id: TokenId,
    },
    /// Remove an existing bid from an ask
    // RemoveBid {
    //     collection: String,
    //     token_id: TokenId,
    // },
    // /// Accept a bid on an existing ask
    AcceptBid {
        collection: String,
        token_id: TokenId,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SudoMsg {
    /// Update the contract parameters
    /// Can only be called by governance
    UpdateParams {
        ask_expiry: Option<ExpiryRange>,
        bid_expiry: Option<ExpiryRange>,
        operators: Option<Vec<String>>,
        min_price: Option<Uint128>,
        listing_fee: Option<Uint128>,
    },
    /// Add a new operator
    AddOperator { operator: String },
    /// Remove operator
    RemoveOperator { operator: String },
    /// Add a new hook to be informed of all asks
    AddAskHook { hook: String },
    /// Add a new hook to be informed of all bids
    AddBidHook { hook: String },
    /// Remove a ask hook
    RemoveAskHook { hook: String },
    /// Remove a bid hook
    RemoveBidHook { hook: String },
    /// Add a new hook to be informed of all trades
    AddSaleHook { hook: String },
    /// Remove a trade hook
    RemoveSaleHook { hook: String },
}

pub type Collection = String;
pub type Bidder = String;
pub type Seller = String;

/// Offset for ask pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOffset {
    pub price: Uint128,
    pub token_id: TokenId,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOffsetBidCount {
    pub bid_count: Uint128,
    pub token_id: TokenId,
    pub collection: String
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskOffsetSellPrice {
    pub price: Uint128,
    pub token_id: TokenId,
    pub collection: String
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AsksOffsetExpiration {
    pub token_id: TokenId,
    pub collection: String,
    pub time: u64
}


impl AskOffset {
    pub fn new(price: Uint128, token_id: TokenId) -> Self {
        AskOffset { price, token_id }
    }
}

/// Offset for bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidOffset {
    pub price: Uint128,
    pub token_id: TokenId,
    pub bidder: Addr,
}

impl BidOffset {
    pub fn new(price: Uint128, token_id: TokenId, bidder: Addr) -> Self {
        BidOffset {
            price,
            token_id,
            bidder,
        }
    }
}
/// Offset for collection pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionOffset {
    pub collection: String,
    pub token_id: TokenId,
}

impl CollectionOffset {
    pub fn new(collection: String, token_id: TokenId) -> Self {
        CollectionOffset {
            collection,
            token_id,
        }
    }
}

/// Offset for collection bid pagination
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidOffset {
    pub price: Uint128,
    pub collection: Collection,
    pub bidder: Bidder,
}

impl CollectionBidOffset {
    pub fn new(price: Uint128, collection: String, bidder: Bidder) -> Self {
        CollectionBidOffset {
            price,
            collection,
            bidder,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// List of collections that have asks on them
    /// Return type: `CollectionsResponse`
    Collections {
        start_after: Option<Collection>,
        limit: Option<u32>,
    },
    /// Get the current ask for specific NFT
    /// Return type: `CurrentAskResponse`
    Ask {
        collection: Collection,
        token_id: TokenId,
    },
    /// Get all asks for a collection
    /// Return type: `AsksResponse`
    Asks {
        collection: Collection,
        start_after: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all asks for a collection in reverse
    /// Return type: `AsksResponse`
    ReverseAsks {
        collection: Collection,
        start_before: Option<TokenId>,
        limit: Option<u32>,
    },
    /// Get all asks for a collection, sorted by price
    /// Return type: `AsksResponse`
    AsksSortedByPrice {
        collection: Collection,
        start_after: Option<AskOffset>,
        limit: Option<u32>,
    },
    /// Get all asks for a collection, sorted by price in reverse
    /// Return type: `AsksResponse`
    ReverseAsksSortedByPrice {
        collection: Collection,
        start_before: Option<AskOffset>,
        limit: Option<u32>,
    },
    /// Get top asks which are based on the bids_count 
    /// Return type: `AsksResponse`
    AsksSortedByBidCount{
        start_after: Option<AskOffsetBidCount>,
        limit: Option<u32>
    },
    //Get sorted asks by the price for all NFTs
    AsksSortedBySellPrice{
        start_after: Option<AskOffsetSellPrice>,
        limit: Option<u32>
    },
    //Asks sorted by the expiration
    AsksSortedByExpiration{
        limit: Option<u32>
    },
    //Get sorted asks by the price for all NFTs(reverse)
    ReverseSortedBySellPrice{
        start_after: Option<AskOffsetSellPrice>,
        limit: Option<u32>
    },
    //Get top asks which are base on the content-type
    AsksSortedByContentType{
      content_type: String,
      start_after: Option<CollectionOffset>,
      limit: Option<u32>  
    },
    /// Count of all asks
    /// Return type: `AskCountResponse`
    AskCount { collection: Collection },
    /// Get all asks by seller
    /// Return type: `AsksResponse`
    AsksBySeller {
        seller: Seller,
        start_after: Option<CollectionOffset>,
        limit: Option<u32>,
    },
    /// Get data for a specific bid
    /// Return type: `BidResponse`
    Bid {
        collection: Collection,
        token_id: TokenId,
        bidder: Bidder,
    },
    /// Get all bids by a bidder
    /// Return type: `BidsResponse`
    BidsByBidder {
        bidder: Bidder,
        start_after: Option<CollectionOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a specific NFT
    /// Return type: `BidsResponse`
    Bids {
        collection: Collection,
        token_id: TokenId,
        start_after: Option<Bidder>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price
    /// Return type: `BidsResponse`
    BidsSortedByPrice {
        collection: Collection,
        start_after: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Get all bids for a collection, sorted by price in reverse
    /// Return type: `BidsResponse`
    ReverseBidsSortedByPrice {
        collection: Collection,
        start_before: Option<BidOffset>,
        limit: Option<u32>,
    },
    /// Show all registered ask hooks
    /// Return type: `HooksResponse`
    AskHooks {},
    /// Show all registered bid hooks
    /// Return type: `HooksResponse`
    BidHooks {},
    /// Show all registered sale hooks
    /// Return type: `HooksResponse`
    SaleHooks {},
    /// Get the config for the contract
    /// Return type: `ParamsResponse`
    Params {},
}



#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CreateCollectionQueryMsg {
    /// List of collections that have asks on them
    /// Return type: `CollectionsResponse`
      CheckCollection{address: String}
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskResponse {
    pub ask: Option<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AsksResponse {
    pub asks: Vec<Ask>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AskCountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionsResponse {
    pub collections: Vec<Addr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidResponse {
    pub bid: Option<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BidsResponse {
    pub bids: Vec<Bid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ParamsResponse {
    pub params: SudoParams,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidResponse {
    pub bid: Option<CollectionBid>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBidsResponse {
    pub bids: Vec<CollectionBid>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct SaleHookMsg {
    pub collection: String,
    pub token_id: String,
    pub price: Coin,
    pub seller: String,
    pub buyer: String,
}

impl SaleHookMsg {
    pub fn new(
        collection: String,
        token_id: String,
        price: Coin,
        seller: String,
        buyer: String,
    ) -> Self {
        SaleHookMsg {
            collection,
            token_id,
            price,
            seller,
            buyer,
        }
    }

    /// serializes the message
    pub fn into_binary(self) -> StdResult<Binary> {
        let msg = SaleExecuteMsg::SaleHook(self);
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum SaleExecuteMsg {
    SaleHook(SaleHookMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HookAction {
    Create,
    Update,
    Delete,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AskHookMsg {
    pub ask: Ask,
}

impl AskHookMsg {
    pub fn new(ask: Ask) -> Self {
        AskHookMsg { ask } 
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => AskHookExecuteMsg::AskCreatedHook(self),
            HookAction::Update => AskHookExecuteMsg::AskUpdatedHook(self),
            HookAction::Delete => AskHookExecuteMsg::AskDeletedHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AskHookExecuteMsg {
    AskCreatedHook(AskHookMsg),
    AskUpdatedHook(AskHookMsg),
    AskDeletedHook(AskHookMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct BidHookMsg {
    pub bid: Bid,
}

impl BidHookMsg {
    pub fn new(bid: Bid) -> Self {
        BidHookMsg { bid }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => BidExecuteMsg::BidCreatedHook(self),
            HookAction::Update => BidExecuteMsg::BidUpdatedHook(self),
            HookAction::Delete => BidExecuteMsg::BidDeletedHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum BidExecuteMsg {
    BidCreatedHook(BidHookMsg),
    BidUpdatedHook(BidHookMsg),
    BidDeletedHook(BidHookMsg),
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CollectionBidHookMsg {
    pub collection_bid: CollectionBid,
}

impl CollectionBidHookMsg {
    pub fn new(collection_bid: CollectionBid) -> Self {
        CollectionBidHookMsg { collection_bid }
    }

    /// serializes the message
    pub fn into_binary(self, action: HookAction) -> StdResult<Binary> {
        let msg = match action {
            HookAction::Create => CollectionBidExecuteMsg::CollectionBidCreatedHook(self),
            HookAction::Update => CollectionBidExecuteMsg::CollectionBidUpdatedHook(self),
            HookAction::Delete => CollectionBidExecuteMsg::CollectionBidDeletedHook(self),
        };
        to_binary(&msg)
    }
}

// This is just a helper to properly serialize the above message
#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum CollectionBidExecuteMsg {
    CollectionBidCreatedHook(CollectionBidHookMsg),
    CollectionBidUpdatedHook(CollectionBidHookMsg),
    CollectionBidDeletedHook(CollectionBidHookMsg),
}




#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
pub struct NftInfoResponse<T> {
    /// Universal resource identifier for this NFT
    /// Should point to a JSON file that conforms to the ERC721
    /// Metadata JSON Schema
    pub token_uri: Option<String>,

    pub content_type: String,

    pub created_time: u64,
    /// You can add any custom metadata here when you extend cw721-base
    pub extension: T,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Metadata {
    pub image_url: String,
    pub minter: String
}
