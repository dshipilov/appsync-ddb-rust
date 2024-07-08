use std::collections::HashMap;

use crate::tracing::info;
use aws_config::BehaviorVersion;
use aws_sdk_dynamodb::{types::AttributeValue, Client};

use crate::model::{Customer, Order, Product, ProductQuantity};

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

        // Initial scan through the data to build up the customers and products info
        // Orders are stored for later processing
        for item in data.iter() {
            let pk = item.get("PK").unwrap().as_s().unwrap().to_string();
            let sk = item.get("SK").unwrap().as_s().unwrap().to_string();

            if sk.starts_with("CUSTOMER#") {
                // Customer item - we can build the Customer struct from it righ away
                let customer = Customer {
                    full_name: item.get("fullName").unwrap().as_s().unwrap().to_string(),
                    email: item.get("email").unwrap().as_s().unwrap().to_string(),
                };
                customers.insert(pk.clone(), customer);
            } else if sk.starts_with("PRODUCT#") {
                // Product item - build Product and ProductQuantity structs and accumulate them for the orde PK
                let product = Product {
                    name: item.get("name").unwrap().as_s().unwrap().to_string(),
                    price: item.get("price").unwrap().as_n().unwrap().parse().unwrap(),
                };
                let quantity = item
                    .get("quantity")
                    .unwrap()
                    .as_n()
                    .unwrap()
                    .parse()
                    .unwrap();
                let product_quantity = ProductQuantity {
                    product: product,
                    quantity: quantity,
                };
                let product_quantities = products.entry(pk).or_insert(vec![]);
                product_quantities.push(product_quantity);
            } else if sk == "META" {
                // Order item - store it in a raw form for later processing
                orders.insert(pk.clone(), item);
            }
        }

        // Go through the orders collection and build the final vector of Order structs
        orders
            .into_iter()
            .map(|(order_id, order_item)| {
                // We already have the customers and products info collected, so we are ready to build the Order struct
                let customer = customers.get(&order_id).unwrap();
                let product_quantities = products.get(&order_id).unwrap();

                // Compute the total amount of the order from the products info
                let total_amount = compute_total_amount(product_quantities);

                Order {
                    id: order_item.get("id").unwrap().as_n().unwrap().to_string(),
                    date: order_item.get("date").unwrap().as_s().unwrap().to_string(),
                    total_amount: total_amount,
                    customer: customer.clone(),
                    products: product_quantities.clone(),
                }
            })
            .collect()
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
