use cosmwasm_std::{Addr, BlockInfo, Timestamp, Uint128};
use cw_storage_plus::{Index, IndexList, IndexedMap, Item, MultiIndex};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cw_controllers::Hooks;

use crate::helpers::ExpiryRange;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SudoParams {
    /// Fair Burn fee for winning bids
    // pub trading_fee_percent: Decimal,
    /// Valid time range for Asks
    /// (min, max) in seconds
    pub ask_expiry: ExpiryRange,
    /// Valid time range for Bids
    /// (min, max) in seconds
    pub bid_expiry: ExpiryRange,
    /// Operators are entites that are responsible for maintaining the active state of Asks
    /// They listen to NFT transfer events, and update the active state of Asks
    pub operators: Vec<Addr>,
    /// Max value for the finders fee
    // pub max_finders_fee_percent: Decimal,
    /// Min value for a bid
    pub min_price: Uint128,
    /// Listing fee to reduce spam
    pub listing_fee: Uint128,
}

pub const SUDO_PARAMS: Item<SudoParams> = Item::new("sudo-params");

pub const ASK_HOOKS: Hooks = Hooks::new("ask-hooks");
pub const BID_HOOKS: Hooks = Hooks::new("bid-hooks");
pub const SALE_HOOKS: Hooks = Hooks::new("sale-hooks");
pub const COLLECTION_BID_HOOKS: Hooks = Hooks::new("collection-bid-hooks");

pub type TokenId = String;

pub trait Order {
    fn expires_at(&self) -> Timestamp;

    fn is_expired(&self, block: &BlockInfo) -> bool {
        self.expires_at() <= block.time
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SaleType {
    FixedPrice,
    Auction,
}

/// Represents an ask on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Ask {
    pub sale_type: SaleType,
    pub collection: Addr,
    pub token_id: TokenId,
    pub img_url: String,
    pub seller: Addr,
    pub price: Uint128,
    pub funds_recipient: Option<Addr>,
    pub expires_at: Timestamp,
    pub max_bid: Option<Uint128>,
    pub max_bidder: Option<Addr>,
    pub bid_count: Uint128,
    pub content_type: String
}

impl Order for Ask {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for asks: (collection, token_id)
pub type AskKey = (Addr, TokenId);
/// Convenience ask key constructor
pub fn ask_key(collection: &Addr, token_id: &TokenId) -> AskKey {
    (collection.clone(), token_id.clone())
}

/// Defines indices for accessing Asks
pub struct AskIndicies<'a> {
    pub collection: MultiIndex<'a, Addr, Ask, AskKey>,
    pub collection_price: MultiIndex<'a, (Addr, u128), Ask, AskKey>,
    pub seller: MultiIndex<'a, Addr, Ask, AskKey>,
    pub bid_count: MultiIndex<'a, u128, Ask, AskKey>,
    pub content_type: MultiIndex<'a, String, Ask, AskKey>,
    pub price: MultiIndex<'a, u128, Ask, AskKey>
}

impl<'a> IndexList<Ask> for AskIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Ask>> + '_> {
        let v: Vec<&dyn Index<Ask>> = vec![&self.collection, &self.collection_price, &self.seller, &self.bid_count, &self.content_type, &self.price];
        Box::new(v.into_iter())
    }
}

pub fn asks<'a>() -> IndexedMap<'a, AskKey, Ask, AskIndicies<'a>> {
    let indexes = AskIndicies {
        collection: MultiIndex::new(|d: &Ask| d.collection.clone(), "asks", "asks__collection"),
        collection_price: MultiIndex::new(
            |d: &Ask| (d.collection.clone(), d.price.u128()),
            "asks",
            "asks__collection_price",
        ),
        seller: MultiIndex::new(|d: &Ask| d.seller.clone(), "asks", "asks__seller"),
        bid_count: MultiIndex::new(|d: &Ask| d.bid_count.u128() , "asks", "bids_count"),
        content_type: MultiIndex::new(|d: &Ask| d.content_type.clone() , "asks", "content_type"),
        price: MultiIndex::new(|d: &Ask| d.price.u128() , "asks", "asks__price")
    };
    IndexedMap::new("asks", indexes)
}

/// Represents a bid (offer) on the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Bid {
    pub collection: Addr,
    pub token_id: TokenId,
    pub bidder: Addr,
    pub price: Uint128,
    pub active: bool,
    pub time: Timestamp
}

impl Bid {
    pub fn new(
        collection: Addr,
        token_id: TokenId,
        bidder: Addr,
        price: Uint128,
        active: bool,
        time: Timestamp
    ) -> Self {
        Bid {
            collection,
            token_id,
            bidder,
            price,
            active,
            time
        }
    }
}

/// Primary key for bids: (collection, token_id, bidder)
pub type BidKey = (Addr, TokenId, Addr);
/// Convenience bid key constructor
pub fn bid_key(collection: &Addr, token_id: &TokenId, bidder: &Addr) -> BidKey {
    (collection.clone(), token_id.clone(), bidder.clone())
}

/// Defines incides for accessing bids
pub struct BidIndicies<'a> {
    pub collection: MultiIndex<'a, Addr, Bid, BidKey>,
    pub collection_token_id: MultiIndex<'a, (Addr, TokenId), Bid, BidKey>,
    pub collection_price: MultiIndex<'a, (Addr, u128), Bid, BidKey>,
    pub bidder: MultiIndex<'a, Addr, Bid, BidKey>,
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
}

impl<'a> IndexList<Bid> for BidIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<Bid>> + '_> {
        let v: Vec<&dyn Index<Bid>> = vec![
            &self.collection,
            &self.collection_token_id,
            &self.collection_price,
            &self.bidder,
        ];
        Box::new(v.into_iter())
    }
}

pub fn bids<'a>() -> IndexedMap<'a, BidKey, Bid, BidIndicies<'a>> {
    let indexes = BidIndicies {
        collection: MultiIndex::new(|d: &Bid| d.collection.clone(), "bids", "bids__collection"),
        collection_token_id: MultiIndex::new(
            |d: &Bid| (d.collection.clone(), d.token_id.clone()),
            "bids",
            "bids__collection_token_id",
        ),
        collection_price: MultiIndex::new(
            |d: &Bid| (d.collection.clone(), d.price.u128()),
            "bids",
            "bids__collection_price",
        ),
        bidder: MultiIndex::new(|d: &Bid| d.bidder.clone(), "bids", "bids__bidder"),
    };
    IndexedMap::new("bids", indexes)
}

/// Represents a bid (offer) across an entire collection in the marketplace
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct CollectionBid {
    pub collection: Addr,
    pub bidder: Addr,
    pub price: Uint128,
    // pub finders_fee_bps: Option<u64>,
    pub expires_at: Timestamp,
}

impl Order for CollectionBid {
    fn expires_at(&self) -> Timestamp {
        self.expires_at
    }
}

/// Primary key for bids: (collection, token_id, bidder)
pub type CollectionBidKey = (Addr, Addr);
/// Convenience collection bid key constructor
pub fn collection_bid_key(collection: &Addr, bidder: &Addr) -> CollectionBidKey {
    (collection.clone(), bidder.clone())
}

/// Defines incides for accessing collection bids
pub struct CollectionBidIndicies<'a> {
    pub collection: MultiIndex<'a, Addr, CollectionBid, CollectionBidKey>,
    pub collection_price: MultiIndex<'a, (Addr, u128), CollectionBid, CollectionBidKey>,
    pub bidder: MultiIndex<'a, Addr, CollectionBid, CollectionBidKey>,
    // Cannot include `Timestamp` in index, converted `Timestamp` to `seconds` and stored as `u64`
    pub bidder_expires_at: MultiIndex<'a, (Addr, u64), CollectionBid, CollectionBidKey>,
}

impl<'a> IndexList<CollectionBid> for CollectionBidIndicies<'a> {
    fn get_indexes(&'_ self) -> Box<dyn Iterator<Item = &'_ dyn Index<CollectionBid>> + '_> {
        let v: Vec<&dyn Index<CollectionBid>> = vec![
            &self.collection,
            &self.collection_price,
            &self.bidder,
            &self.bidder_expires_at,
        ];
        Box::new(v.into_iter())
    }
}

pub fn collection_bids<'a>(
) -> IndexedMap<'a, CollectionBidKey, CollectionBid, CollectionBidIndicies<'a>> {
    let indexes = CollectionBidIndicies {
        collection: MultiIndex::new(
            |d: &CollectionBid| d.collection.clone(),
            "col_bids",
            "col_bids__collection",
        ),
        collection_price: MultiIndex::new(
            |d: &CollectionBid| (d.collection.clone(), d.price.u128()),
            "col_bids",
            "col_bids__collection_price",
        ),
        bidder: MultiIndex::new(
            |d: &CollectionBid| d.bidder.clone(),
            "col_bids",
            "col_bids__bidder",
        ),
        bidder_expires_at: MultiIndex::new(
            |d: &CollectionBid| (d.bidder.clone(), d.expires_at.seconds()),
            "col_bids",
            "col_bids__bidder_expires_at",
        ),
    };
    IndexedMap::new("col_bids", indexes)
}
