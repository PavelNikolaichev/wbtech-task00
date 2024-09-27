use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
    response::IntoResponse, http::StatusCode
};
use serde::{Deserialize, Serialize};
use std::{sync::Arc};
use tokio_postgres::{NoTls, Client};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use postgres_types::FromSql;
use validator::Validate;
use tokio::sync::RwLock;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromSql)]
struct Delivery {
    #[validate(length(min = 1))]
    name: String,
    // #[validate(phone)]
    phone: String,
    zip: String,
    city: String,
    address: String,
    region: String,
    #[validate(email)]
    email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromSql)]
struct Payment {
    transaction: String,
    request_id: String,
    currency: String,
    provider: String,
    amount: u32,
    payment_dt: u32,
    bank: String,
    delivery_cost: u32,
    goods_total: u32,
    custom_fee: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromSql)]
struct Item {
    chrt_id: u32,
    track_number: String,
    price: u32,
    rid: String,
    name: String,
    sale: u32,
    size: String,
    total_price: u32,
    nm_id: u32,
    brand: String,
    status: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, FromSql)]
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
    shared_key: String,
    sm_id: u32,
    date_created: String,
    oof_shard: String,
}

struct AppState {
    db_client: Client,
    cache: RwLock<HashMap<String, Order>>,
}

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let (client, connection) = tokio_postgres::connect("host=localhost user=postgres password=postgres dbname=postgres \
    ", NoTls)
        .await
        .expect("Failed to connect to Postgres");
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            error!("Postgres connection error: {}", e);
        }
    });

    // create table if not exists
    client
        .execute(
            "CREATE TABLE IF NOT EXISTS orders (
                order_uid VARCHAR PRIMARY KEY,
                data JSONB NOT NULL
            )",
            &[],
        )
        .await
        .expect("Failed to create orders table");

    let app_state = Arc::new(AppState {
        db_client: client,
        cache: RwLock::new(HashMap::new()),
    });

    let app = Router::new()
        .route("/orders", get(get_orders).post(create_order))
        .route("/orders/:id", get(get_order_by_id).put(update_order))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service()).await.unwrap();
}

async fn get_orders(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting all orders");

    let rows = match state.db_client.query("SELECT data FROM orders", &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Error querying orders: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to retrieve orders").into_response();
        }
    };

    let orders: Vec<Order> = rows
        .iter()
        .filter_map(|row| {
            let json_value: serde_json::Value = row.get("data");
            serde_json::from_value(json_value).ok()
        })
        .collect();
    let mut cache = state.cache.write().await;
    for order in orders.iter() {
        cache.insert(order.order_uid.clone(), order.clone());
    }

    (StatusCode::OK, Json(orders)).into_response()
}

async fn get_order_by_id(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>
) -> impl IntoResponse {
    info!("Getting order by ID: {}", id);

    let cache = state.cache.read().await;
    if let Some(order) = cache.get(&id) {
        return (StatusCode::OK, Json(order.clone())).into_response();
    }

    match state.db_client.query_one("SELECT data FROM orders WHERE order_uid = $1", &[&id]).await {
        Ok(row) => {
            let order: Order = row.get("data");
            let mut cache = state.cache.write().await;
            cache.insert(id.clone(), order.clone());
            (StatusCode::OK, Json(order)).into_response()
        }
        Err(e) => {
            error!("Error fetching order by ID: {}", e);
            (StatusCode::NOT_FOUND, "Order not found").into_response()
        }
    }
}



async fn create_order(State(state): State<Arc<AppState>>, Json(order): Json<Order>) -> Result<Json<Order>, String> {
    info!("Creating new order with UID: {}", order.order_uid);

    if let Err(validation_errors) = order.validate() {
        error!("Validation error: {:?}", validation_errors);
        return Err("Invalid data".to_string());
    }

    let query = "INSERT INTO orders (order_uid, data) VALUES ($1, $2::jsonb)";
    let order_json: serde_json::Value = serde_json::to_value(&order).unwrap();
    if let Err(e) = state.db_client.execute(query, &[&order.order_uid, &order_json]).await {
        error!("Error inserting order: {}", e);
        return Err("Failed to create order".to_string());
    }

    let mut cache = state.cache.write().await;
    cache.insert(order.order_uid.clone(), order.clone());

    Ok(Json(order))
}

async fn update_order(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(order): Json<Order>
) -> Result<Json<Order>, String> {
    info!("Updating order with UID: {}", id);

    if let Err(validation_errors) = order.validate() {
        error!("Validation error: {:?}", validation_errors);
        return Err("Invalid data".to_string());
    }

    let query = "UPDATE orders SET data = $1::jsonb WHERE order_uid = $2";
    let order_json = serde_json::to_string(&order).unwrap();
    if let Err(e) = state.db_client.execute(query, &[&order_json, &id]).await {
        error!("Error updating order: {}", e);
        return Err("Failed to update order".to_string());
    }

    let mut cache = state.cache.write().await;
    cache.insert(id.clone(), order.clone());

    Ok(Json(order))
}