mod config;
mod http;

use config::Config;
use deadpool_lapin::{Manager, Pool, PoolError};
use std::result::Result;
use std::time::Duration;

use futures_lite::StreamExt;
use http::RequestBuilder;
use lapin::{options::*, types::FieldTable, ConnectionProperties};

type Connection = deadpool::managed::Object<deadpool_lapin::Manager>;

#[tokio::main]
async fn main() {
    let cfg = match Config::parse_from_file("./indexer.yml") {
        Ok(cfg) => cfg,
        Err(message) => panic!("{:?}", message),
    };

    let addr = cfg.queue_host;
    let manager = Manager::new(
        addr,
        ConnectionProperties::default()
            .with_executor(tokio_executor_trait::Tokio::current())
            .with_reactor(tokio_reactor_trait::Tokio),
    );

    let pool: Pool = deadpool::managed::Pool::builder(manager)
        .max_size(10)
        .build()
        .expect("can't create pool");

    // let msg = "message";

    let client = reqwest::Client::new();
    let rb = RequestBuilder::new(client, cfg.method, cfg.pattern);

    let mut retry_interval = tokio::time::interval(Duration::from_secs(5));
    loop {
        retry_interval.tick().await;
        println!("connecting rmq consumer...");
        match init_rmq_listen(pool.clone(), &rb).await {
            Ok(_) => println!("rmq listen returned"),
            Err(e) => eprintln!("rmq listen had an error: {}", e),
        };
    }
}

async fn init_rmq_listen(pool: Pool, requester: &RequestBuilder) -> Result<(), PoolError> {
    let rmq_con = get_rmq_con(pool).await.map_err(|e| {
        eprintln!("could not get rmq con: {}", e);
        e
    })?;
    let channel = rmq_con.create_channel().await?;

    let queue = channel
        .queue_declare(
            "hello",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await?;
    println!("Declared queue {:?}", queue);

    let mut consumer = channel
        .basic_consume(
            "hello",
            "index_consumer",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await?;

    println!("rmq consumer connected, waiting for messages");

    while let Some(delivery) = consumer.next().await {
        if let Ok(delivery) = delivery {
            println!("got a message: {:?}", delivery);

            let s = match std::str::from_utf8(&delivery.data) {
                Ok(s) => s,
                Err(e) => {
                    println!("could not convert message to string: {}", e);
                    continue;
                }
            };

            let req = match requester.clone().build(s) {
                Some(r) => r,
                None => {
                    println!("could not build request");
                    continue;
                }
            };

            let resp = req.send().await;
            println!("got response: {:?}", resp);

            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("could not ack");
        }
    }

    Ok(())
}

async fn get_rmq_con(pool: Pool) -> Result<Connection, PoolError> {
    let connection = pool.get().await?;
    Ok(connection)
}
