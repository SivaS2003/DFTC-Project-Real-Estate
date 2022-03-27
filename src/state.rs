use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use cosmwasm_std::Coin;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]

// the state of the smart contract, aka where the info for the instance of the smart contract is stored
pub struct State {
    pub address : String,
    pub rentcost : Vec<Coin>,
    pub renters : Vec<Addr>,
    pub ownername : String,
    pub owner: Addr,

}

pub const STATE: Item<State> = Item::new("state");
