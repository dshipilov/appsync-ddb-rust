## Overview
This solution demonstrates the development of a backend functionality using AWS services, focusing on an AppSync API that allows querying customer orders from DynamoDB. The primary goal is to assess the ability to implement backend infrastructure, adhere to best practices, and document the code effectively.

## Key Objectives
1. **Create an AWS CDK App**: Utilize AWS Cloud Development Kit (CDK) to define the cloud infrastructure using code.
2. **Provision DynamoDB**: Set up a DynamoDB table to store hierarchical seed data for customers, orders, and products.
3. **Seed Data Structure**:
    - **Customer**: email, full name
    - **Order**: id, date, total amount, products quantity
    - **Product**: name, price
4. **Implement AppSync with Rust Resolver**: Develop an AppSync GraphQL API with a Rust-based resolver to handle read-only queries for retrieving orders.

## Implementation Details
- **AWS CDK Constructs**: 
  - Define and provision the DynamoDB table.
  - Set up AppSync API with a schema supporting the required query structure.
  - Deploy a Rust Lambda function as the resolver for the AppSync API.
- **Query Structure**:
  ```graphql
  query {
    orders {
      id
      date
      totalAmount
      customer {
        email
        fullName
      }
      products {
        price
        quantity
      }
    }
  }
  ```

## Useful commands

* `npm run build`   compile typescript to js
* `npm run watch`   watch for changes and compile
* `npm run test`    perform the jest unit tests
* `npx cdk deploy`  deploy this stack to your default AWS account/region
* `npx cdk diff`    compare deployed stack with current state
* `npx cdk synth`   emits the synthesized CloudFormation template

## Rust lambda development

Use cargo lambda for build and debug Rust lambda code.
Use the following instruction to install: https://www.cargo-lambda.info/guide/installation.html

* Build with `cargo lambda build`
* Run locally with `RUST_BACKTRACE=1 cargo lambda watch --env-var ORDERS_TABLE=CustomerOrders`
* Invoke with `cargo lambda invoke --data-ascii '{"field": "orders"}'`
* Test deploy with `cargo lambda deploy --env-var ORDERS_TABLE=CustomerOrders`
