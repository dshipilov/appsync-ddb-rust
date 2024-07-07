import * as cdk from 'aws-cdk-lib';
import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import { Construct } from 'constructs';
import * as path from 'path';
import { SampleDataTable } from '../constructs/sample-data-table';


export class AppsyncDdbRustStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

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

    new cdk.CfnOutput(this, "TableName", {
      value: table.tableName || "",
    });

    new cdk.CfnOutput(this, "TableARN", {
      value: table.tableArn || "",
    });
  }
}
