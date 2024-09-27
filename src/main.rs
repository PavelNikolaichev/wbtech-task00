use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_postgres::{Client, NoTls};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;
use validator::Validate;

#[path="models/models.rs"] mod models;
use crate::models::{Order, PutOrder};


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

    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=postgres password=postgres dbname=postgres \
    ",
        NoTls,
    )
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

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    info!("Listening on: {}", listener.local_addr().unwrap());

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn get_orders(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    info!("Getting all orders");

    let rows = match state.db_client.query("SELECT data FROM orders", &[]).await {
        Ok(rows) => rows,
        Err(e) => {
            error!("Error querying orders: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to retrieve orders",
            )
                .into_response();
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
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Getting order by ID: {}", id);

    let cache = state.cache.read().await;
    if let Some(order) = cache.get(&id) {
        return (StatusCode::OK, Json(order.clone())).into_response();
    }

    match state
        .db_client
        .query_one("SELECT data FROM orders WHERE order_uid = $1", &[&id])
        .await
    {
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

async fn create_order(
    State(state): State<Arc<AppState>>,
    Json(order): Json<Order>,
) -> Result<Json<Order>, String> {
    info!("Creating new order with UID: {}", order.order_uid);

    if let Err(validation_errors) = order.validate() {
        error!("Validation error: {:?}", validation_errors);
        return Err("Invalid data".to_string());
    }

    let query = "INSERT INTO orders (order_uid, data) VALUES ($1, $2::jsonb)";
    let order_json: serde_json::Value = serde_json::to_value(&order).unwrap();
    if let Err(e) = state
        .db_client
        .execute(query, &[&order.order_uid, &order_json])
        .await
    {
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
    Json(partial_order): Json<PutOrder>,
) -> Result<Json<Order>, String> {
    info!("Updating order with UID: {}", id);

    let mut cache = state.cache.write().await;

    let mut order = if let Some(cached_order) = cache.get(&id) {
        cached_order.clone()
    } else {
        match state
            .db_client
            .query_one("SELECT data FROM orders WHERE order_uid = $1", &[&id])
            .await
        {
            Ok(row) => {
                let order_json: serde_json::Value = row.get("data");
                serde_json::from_value::<Order>(order_json).unwrap()
            }
            Err(_) => return Err("Order not found".to_string()),
        }
    };

    // Well, sorry for this bunch of code, I know it's bad and I know that my DB schema is the worst - basically useless.
    if let Some(track_number) = partial_order.track_number {
        order.track_number = track_number;
    }
    if let Some(entry) = partial_order.entry {
        order.entry = entry;
    }
    if let Some(delivery) = partial_order.delivery {
        order.delivery = delivery;
    }
    if let Some(payment) = partial_order.payment {
        order.payment = payment;
    }
    if let Some(items) = partial_order.items {
        order.items = items;
    }
    if let Some(locale) = partial_order.locale {
        order.locale = locale;
    }
    if let Some(internal_signature) = partial_order.internal_signature {
        order.internal_signature = internal_signature;
    }
    if let Some(customer_id) = partial_order.customer_id {
        order.customer_id = customer_id;
    }
    if let Some(delivery_service) = partial_order.delivery_service {
        order.delivery_service = delivery_service;
    }
    if let Some(shared_key) = partial_order.shared_key {
        order.shared_key = shared_key;
    }
    if let Some(sm_id) = partial_order.sm_id {
        order.sm_id = sm_id;
    }
    if let Some(date_created) = partial_order.date_created {
        order.date_created = date_created;
    }
    if let Some(oof_shard) = partial_order.oof_shard {
        order.oof_shard = oof_shard;
    }

    let query = "UPDATE orders SET data = $1::jsonb WHERE order_uid = $2";
    let order_json = serde_json::to_value(&order).unwrap();
    if let Err(e) = state.db_client.execute(query, &[&order_json, &id]).await {
        error!("Error updating order: {}", e);
        return Err("Failed to update order".to_string());
    }

    cache.insert(id.clone(), order.clone());

    Ok(Json(order))
}
