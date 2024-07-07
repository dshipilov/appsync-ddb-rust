import { AppSyncResolverHandler } from 'aws-lambda';
import { DynamoDB } from "aws-sdk";

type Customer = {
    fullName: string;
    email: string;
};

type Product = {
    name: string;
    price: number;
}

type ProductQuantity = {
    product: Product;
    quantity: number;
}

type Order = {
    id: string;
    date: string;
    totalAmount: number
    customer?: Customer
    products?: [ProductQuantity]
};

const docClient = new DynamoDB.DocumentClient();

export const handler: AppSyncResolverHandler<null, Order[] | null> =
    async () => {
        try {

            if (!process.env.ORDERS_TABLE) {
                console.log("ORDERS_TABLE was not specified");
                return null;
            }

            console.log("allOrders lambda has been called on: ", process.env.ORDERS_TABLE);

            //const data = await docClient
            //  .scan({ TableName: process.env.ORDERS_TABLE })
            //  .promise();

            //return data.Items as Order[];
            let orders: Order[] = [{ id: "1", date: "2024-01-01 12:00", totalAmount: 0.5, customer: { fullName: "John", email: "john@company.com" } }];
            return orders;
        } catch (err) {
            console.error("[Error] DynamoDB error: ", err);
            return null;
        }
    };
