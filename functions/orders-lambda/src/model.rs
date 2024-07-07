//! Data models for the orders resolver
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Customer {
    #[serde(rename = "fullName")]
    pub full_name: String,
    pub email: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Product {
    pub name: String,
    pub price: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct ProductQuantity {
    pub product: Product,
    pub quantity: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct Order {
    pub id: String,
    pub date: String,
    #[serde(rename = "totalAmount")]
    pub total_amount: f64,
    pub customer: Customer,
    pub products: Vec<ProductQuantity>,
}
