#![cfg_attr(target_arch = "wasm32", no_main)]

mod state;

use self::state::Mall;
use async_graphql::{EmptySubscription, Object, Request, Response, Schema};
use async_trait::async_trait;
use linera_sdk::{
    base::{Amount, WithServiceAbi},
    QueryContext, Service, ViewStateStorage,
};
use mall::Operation;
use std::sync::Arc;
use thiserror::Error;

linera_sdk::service!(Mall);

impl WithServiceAbi for Mall {
    type Abi = mall::MallAbi;
}

#[async_trait]
impl Service for Mall {
    type Error = ServiceError;
    type Storage = ViewStateStorage<Self>;

    async fn query_application(
        self: Arc<Self>,
        _context: &QueryContext,
        request: Request,
    ) -> Result<Response, Self::Error> {
        let schema = Schema::build(self.clone(), MutationRoot {}, EmptySubscription).finish();
        let response = schema.execute(request).await;
        Ok(response)
    }
}

struct MutationRoot;

#[Object]
impl MutationRoot {
    async fn create_collection(
        &self,
        base_uri: String,
        price: Option<Amount>,
        name: String,
    ) -> Vec<u8> {
        bcs::to_bytes(&Operation::CreateCollection {
            base_uri,
            price,
            name,
        })
        .unwrap()
    }

    async fn mint_nft(
        &self,
        collection_id: u64,
        uri: Option<String>,
        price: Option<Amount>,
        name: String,
    ) -> Vec<u8> {
        bcs::to_bytes(&Operation::MintNFT {
            collection_id,
            uri,
            price,
            name,
        })
        .unwrap()
    }

    async fn buy_nft(&self, collection_id: u64, token_id: u16, credits: Amount) -> Vec<u8> {
        bcs::to_bytes(&Operation::BuyNFT {
            collection_id,
            token_id,
            credits,
        })
        .unwrap()
    }

    async fn update_credits_per_linera(&self, credits_per_linera: Amount) -> Vec<u8> {
        bcs::to_bytes(&Operation::UpdateCreditsPerLinera { credits_per_linera }).unwrap()
    }

    async fn update_nft_price(
        &self,
        collection_id: u64,
        token_id: Option<u16>,
        price: Amount,
    ) -> Vec<u8> {
        bcs::to_bytes(&Operation::UpdateNFTPrice {
            collection_id,
            token_id,
            price,
        })
        .unwrap()
    }

    async fn on_sale_nft(&self, collection_id: u64, token_id: u16) -> Vec<u8> {
        bcs::to_bytes(&Operation::OnSaleNFT {
            collection_id,
            token_id,
        })
        .unwrap()
    }

    async fn off_sale_nft(&self, collection_id: u64, token_id: u16) -> Vec<u8> {
        bcs::to_bytes(&Operation::OffSaleNFT {
            collection_id,
            token_id,
        })
        .unwrap()
    }
}

/// An error that can occur while querying the service.
#[derive(Debug, Error)]
pub enum ServiceError {
    /// Query not supported by the application.
    #[error("Queries not supported by application")]
    QueriesNotSupported,

    /// Invalid query argument; could not deserialize request.
    #[error("Invalid query argument; could not deserialize request")]
    InvalidQuery(#[from] serde_json::Error),
    // Add error variants here.
}
