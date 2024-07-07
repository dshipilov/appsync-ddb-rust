import * as cdk from 'aws-cdk-lib';

import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import * as appsync from "aws-cdk-lib/aws-appsync";
import * as lambda from "aws-cdk-lib/aws-lambda";

import { Construct } from 'constructs';
import * as path from 'path';
import { SampleDataTable } from '../constructs/sample-data-table';


export class AppsyncDdbRustStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    // Stage 1: deploy a DynamoDB table with sample data
    const table = new SampleDataTable(this, 'CustomerOrders', {
      tableName: 'ddb-test-customer-orders',
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
    const api = new appsync.GraphqlApi(this, "API", {
      name: "OrdersAPI",
      schema: appsync.SchemaFile.fromAsset("schema/orders.graphql"),
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

    const allOrdersLambda = new lambda.Function(this, "allOrdersHandler", {
      handler: "allOrders.handler",
      runtime: lambda.Runtime.NODEJS_16_X,
      code: lambda.Code.fromAsset("functions"),
      environment: {
        ORDERS_TABLE: table.tableName,
      },
    });

    table.grantReadData(allOrdersLambda);

    const allOrdersDataSource = api.addLambdaDataSource(
      "allOrdersDataSource",
      allOrdersLambda,
    );

    allOrdersDataSource.createResolver("allOrders", {
      typeName: "Query",
      fieldName: "allOrders",
    });

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
