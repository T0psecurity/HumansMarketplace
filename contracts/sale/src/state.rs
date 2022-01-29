use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, Coin};
use cw_storage_plus::{Item, Map};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    pub admin: Addr,
    pub num_tokens: u64,
    pub sg721_code_id: u64,
    pub unit_price: Coin,
}

pub const CONFIG: Item<Config> = Item::new("config");
pub const SG721_ADDRESS: Item<Addr> = Item::new("sg721_address");
pub const TOKEN_ID_INDEX: Item<u64> = Item::new("token_id");
pub const TOKEN_URIS: Map<u64, String> = Map::new("token_uris");
