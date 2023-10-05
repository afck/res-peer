use std::collections::HashMap;

use async_graphql::{Request, Response, SimpleObject};
use linera_sdk::base::{Amount, ApplicationId, ContractAbi, Owner, ServiceAbi, Timestamp};
use serde::{Deserialize, Serialize};

pub struct MarketAbi;

impl ContractAbi for MarketAbi {
    type Parameters = ApplicationId<credit::CreditAbi>;
    type InitializationArgument = InitialState;
    type Operation = Operation;
    type Message = ();
    type ApplicationCall = ();
    type SessionCall = ();
    type SessionState = ();
    type Response = ();
}

impl ServiceAbi for MarketAbi {
    type Parameters = ApplicationId<credit::CreditAbi>;
    type Query = Request;
    type QueryResponse = Response;
}

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject, Eq, PartialEq)]
pub struct NFT {
    /// Sequence ID of NFT in collections
    pub token_id: u16,
    /// Storage location of http or ipfs
    pub uri_index: u16,
    /// Price in Linera Token
    pub price: Option<Amount>,
    pub on_sale: bool,
    pub minted_at: Timestamp,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, SimpleObject, Eq, PartialEq)]
pub struct Collection {
    pub collection_id: u64,
    pub base_uri: String,
    pub uris: Vec<String>,
    pub nfts: HashMap<u16, NFT>,
    pub price: Option<Amount>,
    pub name: String,
    pub created_at: Timestamp,
    pub publisher: Owner,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct InitialState {
    pub credits_per_linera: Amount,
    pub max_credits_percent: u8,
    pub trade_fee_percent: u8,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Operation {
    CreateCollection {
        base_uri: String,
        price: Option<Amount>,
        name: String,
        uris: Vec<String>,
    },
    MintNFT {
        collection_id: u64,
        uri_index: u16,
        price: Option<Amount>,
        name: String,
    },
    BuyNFT {
        collection_id: u64,
        token_id: u16,
        credits: Amount,
    },
    UpdateCreditsPerLinera {
        credits_per_linera: Amount,
    },
    UpdateNFTPrice {
        collection_id: u64,
        token_id: Option<u16>,
        price: Amount,
    },
    OnSaleNFT {
        collection_id: u64,
        token_id: u16,
    },
    OffSaleNFT {
        collection_id: u64,
        token_id: u16,
    },
    Deposit {
        amount: Amount,
    },
    SetAvatar {
        collection_id: u64,
        token_id: u16,
    },
}
