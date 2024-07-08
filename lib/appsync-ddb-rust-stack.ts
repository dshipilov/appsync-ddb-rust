import * as cdk from 'aws-cdk-lib';

import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import * as appsync from "aws-cdk-lib/aws-appsync";
import * as lambda from "aws-cdk-lib/aws-lambda";
import { RustFunction } from 'cargo-lambda-cdk';

import { Construct } from 'constructs';
import * as path from 'path';
import { SampleDataTable } from '../constructs/sample-data-table';


export class AppsyncDdbRustStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Stage 1: deploy a DynamoDB table with sample data
    const table = new SampleDataTable(this, 'CustomerOrders', {
      tableName: 'CustomerOrders',
      dataFolderPath: path.join(__dirname, '../sample-data/'), // the path to the json files
      removalPolicy: cdk.RemovalPolicy.DESTROY,
      partitionKey: {
        name: 'PK',
        type: dynamodb.AttributeType.STRING,
      },
      sortKey: {
        name: 'SK',
        type: dynamodb.AttributeType.STRING,
      },
    }).table;

    // Stage 2: deploy an AppSync API with a Lambda resolvers
    const api = new appsync.GraphqlApi(this, "OrdersAPI", {
      name: "OrdersAPI",
      definition: appsync.Definition.fromFile(path.join(__dirname, '../schema/orders.graphql')),
      authorizationConfig: {
        defaultAuthorization: {
          authorizationType: appsync.AuthorizationType.API_KEY,
          apiKeyConfig: {
            description: "OrdersAPI Access Key",
            name: "OrdersAPI_Key",
            expires: cdk.Expiration.after(cdk.Duration.days(30)),
          },
        },
      },
    });

    // Define the Lambda function for the orders resolver in Rust
    const ordersLambda = new RustFunction(this, "ordersRustHandler", {
      manifestPath: path.join(__dirname, "../functions/orders-lambda/Cargo.toml"),
      environment: {
        ORDERS_TABLE: table.tableName,
      }
    });

    table.grantReadData(ordersLambda);

    const ordersDataSource = api.addLambdaDataSource(
      "OrdersDataSource",
      ordersLambda,
    );

    ordersDataSource.createResolver("orders", {
      typeName: "Query",
      fieldName: "orders",
    });

    // Output some useful deployment parameters
    new cdk.CfnOutput(this, "API_URL", {
      value: api.graphqlUrl,
    });
    new cdk.CfnOutput(this, "API_Key", {
      value: api.apiKey || "",
    });
    new cdk.CfnOutput(this, "TableName", {
      value: table.tableName || "",
    });
    new cdk.CfnOutput(this, "TableARN", {
      value: table.tableArn || "",
    });
  }
}
