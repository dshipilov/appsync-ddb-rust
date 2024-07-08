mod model;
mod store;

use lambda_runtime::{run, service_fn, tracing, Error, LambdaEvent};
use serde_json::Value;

use model::Order;
use store::get_store;

async fn handler(_event: LambdaEvent<Value>) -> Result<Vec<Order>, Error> {
    let store = get_store().await;

    Ok(store.all_orders().await)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();

    run(service_fn(handler)).await
}
