use serde::{Deserialize, Serialize};
use postgres_types::FromSql;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromSql)]
pub(crate) struct Delivery {
    #[validate(length(min = 1))]
    pub(crate) name: String,
    pub(crate) phone: String,
    pub(crate) zip: String,
    pub(crate) city: String,
    pub(crate) address: String,
    pub(crate) region: String,
    #[validate(email)]
    pub(crate) email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromSql)]
pub(crate) struct Payment {
    pub(crate) transaction: String,
    pub(crate) request_id: String,
    pub(crate) currency: String,
    pub(crate) provider: String,
    pub(crate) amount: u32,
    pub(crate) payment_dt: u32,
    pub(crate) bank: String,
    pub(crate) delivery_cost: u32,
    pub(crate) goods_total: u32,
    pub(crate) custom_fee: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromSql)]
pub(crate) struct Item {
    pub(crate) chrt_id: u32,
    pub(crate) track_number: String,
    pub(crate) price: u32,
    pub(crate) rid: String,
    pub(crate) name: String,
    pub(crate) sale: u32,
    pub(crate) size: String,
    pub(crate) total_price: u32,
    pub(crate) nm_id: u32,
    pub(crate) brand: String,
    pub(crate) status: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromSql)]
pub(crate) struct Order {
    pub(crate) order_uid: String,
    pub(crate) track_number: String,
    pub(crate) entry: String,
    pub(crate) delivery: Delivery,
    pub(crate) payment: Payment,
    pub(crate) items: Vec<Item>,
    pub(crate) locale: String,
    pub(crate) internal_signature: String,
    pub(crate) customer_id: String,
    pub(crate) delivery_service: String,
    pub(crate) shared_key: String,
    pub(crate) sm_id: u32,
    pub(crate) date_created: String,
    pub(crate) oof_shard: String,
}

// PUT requests might be incomplete, so we need to use an optional struct
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub(crate) struct PutOrder {
    pub(crate) track_number: Option<String>,
    pub(crate) entry: Option<String>,
    pub(crate) delivery: Option<Delivery>,
    pub(crate) payment: Option<Payment>,
    pub(crate) items: Option<Vec<Item>>,
    pub(crate) locale: Option<String>,
    pub(crate) internal_signature: Option<String>,
    pub(crate) customer_id: Option<String>,
    pub(crate) delivery_service: Option<String>,
    pub(crate) shared_key: Option<String>,
    pub(crate) sm_id: Option<u32>,
    pub(crate) date_created: Option<String>,
    pub(crate) oof_shard: Option<String>,
}