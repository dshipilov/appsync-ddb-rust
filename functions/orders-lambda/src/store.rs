use std::collections::HashMap;

use crate::tracing::{error, info};
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

use crate::model::{Customer, Order, Product, ProductQuantity};
use anyhow::{anyhow, Result};

pub struct OrdersStore {
    table_name: String,
    client: Client,
}

impl OrdersStore {
    pub async fn all_orders(&self) -> Vec<Order> {
        // `data` contains the list of items from the DynamoDB table, but it's not in the format we need.
        // We need to convert this data into a Vec<Order>
        let data = self
            .client
            .scan()
            .table_name(&self.table_name)
            .send()
            .await
            .map_or_else(|_| vec![], |v| v.items.unwrap_or_default());

        info!("Got DDB scan data: {:?}", data);

        // The approach here is to iterate over the data
        // and build up components of the Order struct into separate collections
        // And finally iterate other collection of order meta data to build the final Order structs

        type OrdersItem = HashMap<String, AttributeValue>;

        let mut customers: HashMap<String, Customer> = HashMap::new(); // Store customers related to ORDER/CUSTOMER items; use PK as key
        let mut orders: HashMap<String, &OrdersItem> = HashMap::new(); // Stores raw order data for later processing; use PK as key
        let mut products: HashMap<String, Vec<ProductQuantity>> = HashMap::new(); // Stores products related to ORDER/PRODUCT items; use PK as key

        // Helper function to compute the total amount of an order from its products
        // Folds over the products and sums the product price * quantity
        fn compute_total_amount(products: &Vec<ProductQuantity>) -> f64 {
            products
                .iter()
                .fold(0.0, |acc, pq| acc + pq.product.price * pq.quantity)
        }

        // Helper functions to extract values from the items
        // Demonstrates how to handle errors with anyhow::Result and Result type combinators
        fn get_attr_str(item: &OrdersItem, key: &str) -> Result<String> {
            item.get(key)
                .ok_or_else(|| anyhow!("Missing key: {}", key))?
                .as_s()
                .map_err(|_| anyhow!("Invalid value type for key: {}", key))
                .map(|v| v.to_string())
        }

        // This function additionally needs to parse string to f64 as a final step
        fn get_attr_num(item: &OrdersItem, key: &str) -> Result<f64> {
            item.get(key)
                .ok_or_else(|| anyhow!("Missing key: {}", key))?
                .as_n()
                .map_err(|_| anyhow!("Expected numeric value: {}", key))
                .and_then(|v| {
                    v.parse()
                        .map_err(|_| anyhow!("Invalid numeric value: {}", key))
                })
        }

        // Initial scan through the data to build up the customers and products info
        // Orders are stored for later processing
        // Actual code is wrapped into lambda which returns Result<> type to handle errors with ? operator
        let items_processing_res: Result<()> = (|| {
            for item in data.iter() {
                let pk = get_attr_str(item, "PK")?;
                let sk = get_attr_str(item, "SK")?;

                match sk.split("#").next().unwrap() {
                    // Get the first part of the SK
                    "CUSTOMER" => {
                        let customer = Customer {
                            full_name: get_attr_str(item, "fullName")?,
                            email: get_attr_str(item, "email")?,
                        };
                        customers.insert(pk.clone(), customer);
                    }
                    "PRODUCT" => {
                        let product = Product {
                            name: get_attr_str(item, "name")?,
                            price: get_attr_num(item, "price")?,
                        };
                        let quantity = get_attr_num(item, "quantity")?;
                        let product_quantity = ProductQuantity {
                            product: product,
                            quantity: quantity,
                        };
                        let product_quantities = products.entry(pk).or_insert(vec![]);
                        product_quantities.push(product_quantity);
                    }
                    "META" => {
                        orders.insert(pk.clone(), item);
                    }
                    _ => Err(anyhow!("Unknown item type: {}", sk))?,
                }
            }
            Ok(())
        })();

        // Break out early if there was an error processing the items
        if let Err(e) = items_processing_res {
            error!("Error processing items: {:?}", e);
            return vec![];
        }

        // Go through the orders collection and build the final vector of Order structs
        orders
            .into_iter()
            .filter_map(|(order_id, order_item)| {
                // Attempt to construct the Order struct
                // Using the same lambda pattern as before to handle errors
                let result: Result<Order> = (|| {
                    let customer = customers
                        .get(&order_id)
                        .ok_or_else(|| anyhow!("Customer not found"))?;

                    let product_quantities = products
                        .get(&order_id)
                        .ok_or_else(|| anyhow!("Product quantities not found"))?;

                    let total_amount = compute_total_amount(product_quantities);

                    Ok(Order {
                        id: get_attr_str(order_item, "id")?,
                        date: get_attr_str(order_item, "date")?,
                        total_amount: total_amount,
                        customer: customer.clone(),
                        products: product_quantities.clone(),
                    })
                })();

                match result {
                    Ok(order) => Some(order),
                    Err(e) => {
                        error!("Error processing order {}: {:?}", order_id, e);
                        None
                    }
                }
            })
            .collect::<Vec<Order>>()
    }
}

pub async fn get_store() -> OrdersStore {
    // Load AWS configuration
    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;

    // Initialize a DynamoDB client
    let table_name = std::env::var("ORDERS_TABLE").expect("ORDERS_TABLE env variable must be set");
    info!(
        "Initializing DynamoDB store with table name: {}",
        table_name
    );
    let client = aws_sdk_dynamodb::Client::new(&config);

    OrdersStore { table_name, client }
}
