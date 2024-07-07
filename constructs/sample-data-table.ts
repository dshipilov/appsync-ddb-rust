import * as cdk from 'aws-cdk-lib';
import * as dynamodb from 'aws-cdk-lib/aws-dynamodb';
import * as s3 from 'aws-cdk-lib/aws-s3';
import * as s3Deploy from 'aws-cdk-lib/aws-s3-deployment';

import { Construct } from 'constructs';

interface SampleDataTableProps
  extends Pick<
    dynamodb.TableProps,
    'removalPolicy' | 'partitionKey' | 'tableName' | 'sortKey'
  > {
  partitionKey: dynamodb.Attribute;
  sortKey: dynamodb.Attribute;
  removalPolicy: cdk.RemovalPolicy;

  // JSON data path to pre-populate the table
  dataFolderPath: string;
}

type FixedSampleDataTableProps = Omit<
  dynamodb.TableProps,
  'removalPolicy' | 'partitionKey' | 'tableName' | 'sortKey'
>;

export class SampleDataTable extends Construct {
  public readonly table: dynamodb.Table;

  constructor(scope: Construct, id: string, props: SampleDataTableProps) {
    super(scope, id);

    const fixedProps: FixedSampleDataTableProps = {
      billingMode: dynamodb.BillingMode.PAY_PER_REQUEST,
      encryption: dynamodb.TableEncryption.AWS_MANAGED,
      pointInTimeRecovery: true,
      contributorInsightsEnabled: true,
    };

    // Create S3 bucket to store the sample data for DDB table
    const bucket = new s3.Bucket(this, 'Bucket', {
      autoDeleteObjects: true,
      removalPolicy: cdk.RemovalPolicy.DESTROY,
    });

    // Create the deployment of the local JSON data into s3 for the table import
    let deployment = new s3Deploy.BucketDeployment(this, 'BucketDeployment', {
      sources: [s3Deploy.Source.asset(props.dataFolderPath)],
      destinationBucket: bucket,
    });

    let importSource = {
      bucket,
      inputFormat: dynamodb.InputFormat.dynamoDBJson(),
      compressionType: dynamodb.InputCompressionType.NONE,
    };

    deployment.node.addDependency(bucket); // Ensure the bucket is created first

    // Now create the table
    this.table = new dynamodb.Table(this, id, {
      ...fixedProps,
      importSource: importSource,
      ...props,
    });

    this.table.node.addDependency(deployment as s3Deploy.BucketDeployment); // Ensure the JSON data is copied first
  }
}
