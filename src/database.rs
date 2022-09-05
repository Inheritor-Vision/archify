use tokio_postgres;
use crate::authentication::{Token, AppToken};
use std::time::{SystemTime, UNIX_EPOCH};

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

async fn create_tables(client: &mut tokio_postgres::Client){

	let (_r1, _r2) = futures::join!(
		client.execute("CREATE TABLE IF NOT EXISTS public_playlists (playlist_id TEXT, playlist_sha256 VARCHAR(64), playlist_data JSONB, PRIMARY KEY (playlist_id)) ", &[]),
		client.execute("CREATE TABLE IF NOT EXISTS spotify_tokens (token_value TEXT, user_id TEXT, token_type TEXT, duration BIGINT, received_at BIGINT, PRIMARY KEY(user_id))", &[])
	);

}

pub async fn initiliaze_db() -> tokio_postgres::Client{
	let mut client = connect_db().await;

	create_tables(&mut client).await;

	client

}

pub async fn get_token(client: &mut tokio_postgres::Client, user_id: &String) -> Option<Token>{
	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&user_id.as_str()];
	let r = client.query_opt("SELECT token_value, duration, token_type, received_at FROM spotify_tokens WHERE user_id = $1::TEXT",&params);

	let time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let r = r.await.unwrap();

	let token = match r {
		None => None,
		Some(row) => {
			let received_at = u64::try_from(row.get::<&str,i64>("received_at")).unwrap();
			let duration = u64::try_from(row.get::<&str,i64>("duration")).unwrap();
			if !row.is_empty() && (duration + received_at > time){
				let t = Token {
					token: AppToken{
						access_token: row.get("token_value"),
						expires_in: duration,
						token_type: row.get("token_type")
					},
					received_at: received_at
				};
				Some(t)	
			}else{
				None
			}
		}
	};

	token
}

pub async fn set_token(client: &mut tokio_postgres::Client, user_id: &String, token: &Token){
	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		&user_id.as_str(), 
		&token.token.access_token.as_str(),
		&token.token.token_type.as_str(),
		&i64::try_from(token.token.expires_in).unwrap(),
		&i64::try_from(token.received_at).unwrap()
	];

	let _r = client.execute("INSERT INTO spotify_tokens (user_id, token_value, token_type, duration, received_at) VALUES ($1::TEXT, $2::TEXT, $3::TEXT, $4::BIGINT, $5::BIGINT) ON CONFLICT (user_id) DO UPDATE SET token_value = $2::TEXT, token_type = $3::TEXT, duration = $4::BIGINT, received_at = $5::BIGINT", params).await.unwrap();
}

pub async fn get_all_playlists(client: &mut tokio_postgres::Client){
	let r = client.execute("", &[]);
}
