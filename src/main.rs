use axum::{
    extract::State,
    routing::get,
    Router
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Delivery {
    name: String,
    phone: String,
    zip: String,
    city: String,
    address: String,
    region: String,
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Payment {
    transaction: String,
    request_id: String,
    currency: String,
    provider: String,
    amount: u64,
    payment_dt: u64,
    bank: String,
    delivery_cost: u64,
    goods_total: u64,
    custom_fee: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Item {
    chrt_id: u64,
    track_number: String,
    price: u64,
    rid: String,
    name: String,
    sale: u64,
    size: String,
    total_price: u64,
    nm_id: u64,
    brand: String,
    status: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Order {
    order_uid: String,
    track_number: String,
    entry: String,
    delivery: Delivery,
    payment: Payment,
    items: Vec<Item>,
    locale: String,
    internal_signature: String,
    customer_id: String,
    delivery_service: String,
    shardkey: String,
    sm_id: u32,
    date_created: String,
    oof_shard: String,
}

struct AppState {
    orders: Vec<Order>,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    // Mock data for testing - the same JSON as provided earlier
    let orders = vec![
        Order {
            order_uid: "b563feb7b2b84b6test".to_string(),
            track_number: "WBILMTESTTRACK".to_string(),
            entry: "WBIL".to_string(),
            delivery: Delivery {
                name: "Test Testov".to_string(),
                phone: "+9720000000".to_string(),
                zip: "2639809".to_string(),
                city: "Kiryat Mozkin".to_string(),
                address: "Ploshad Mira 15".to_string(),
                region: "Kraiot".to_string(),
                email: "test@gmail.com".to_string(),
            },
            payment: Payment {
                transaction: "b563feb7b2b84b6test".to_string(),
                request_id: "".to_string(),
                currency: "USD".to_string(),
                provider: "wbpay".to_string(),
                amount: 1817,
                payment_dt: 1637907727,
                bank: "alpha".to_string(),
                delivery_cost: 1500,
                goods_total: 317,
                custom_fee: 0,
            },
            items: vec![
                Item {
                    chrt_id: 9934930,
                    track_number: "WBILMTESTTRACK".to_string(),
                    price: 453,
                    rid: "ab4219087a764ae0btest".to_string(),
                    name: "Mascaras".to_string(),
                    sale: 30,
                    size: "0".to_string(),
                    total_price: 317,
                    nm_id: 2389212,
                    brand: "Vivienne Sabo".to_string(),
                    status: 202,
                }
            ],
            locale: "en".to_string(),
            internal_signature: "".to_string(),
            customer_id: "test".to_string(),
            delivery_service: "meest".to_string(),
            shardkey: "9".to_string(),
            sm_id: 99,
            date_created: "2021-11-26T06:22:19Z".to_string(),
            oof_shard: "1".to_string(),
        },
    ];

    let app_state = Arc::new(AppState { orders });

    let app = Router::new()
        .route("/orders", get(get_orders))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn get_orders(State(state): State<Arc<AppState>>) -> axum::Json<Vec<Order>> {
    info!("Getting orders");
    axum::Json(state.orders.clone())
}
