mod model;
mod utils;

use crate::tracing::info;
use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde_json::Value;

use model::{Customer, Order, Product, ProductQuantity};
use utils::get_dynamodb_store;

async fn handler(event: LambdaEvent<Value>) -> Result<Vec<Order>, Error> {
    let _store = get_dynamodb_store().await;

    info!("Got event: {:?}", event);
    let customer = Customer {
        full_name: "John Doe".to_string(),
        email: "john@company.com".to_string(),
    };

    let product = Product {
        name: "Milk".to_string(),
        price: 1.5,
    };

    let order = Order {
        id: "1".to_string(),
        date: "2021-01-01 12:24:00".to_string(),
        total_amount: 1.5,
        customer: customer,
        products: vec![ProductQuantity {
            product: product,
            quantity: 1.0,
        }],
    };

    Ok(vec![order])
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(handler)).await
}
