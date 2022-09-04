use futures::executor::block_on;
use tokio_postgres;

async fn connect_db() -> tokio_postgres::Client{
	let (client, connection) = tokio_postgres::connect("host=localhost user=archify-user dbname=archify-db", tokio_postgres::NoTls)
	.await
	.unwrap();

	tokio::spawn(async move {
		if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
	});

	client

}

async fn create_tables(){

}

pub async fn initiliaze_db() -> tokio_postgres::Client{
	let client = connect_db().await;

	create_tables().await;

	client

}