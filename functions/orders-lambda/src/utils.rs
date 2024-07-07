use crate::tracing::info;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::Client;

pub struct DDBStore {
    table_name: String,
    client: Client,
}

pub async fn get_dynamodb_store() -> DDBStore {
    // Load AWS configuration
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // Initialize a DynamoDB client
    let table_name = std::env::var("ORDERS_TABLE").expect("ORDERS_TABLE env variable must be set");
    info!(
        "Initializing DynamoDB store with table name: {}",
        table_name
    );
    let client = aws_sdk_dynamodb::Client::new(&config);

    DDBStore { table_name, client }
}
