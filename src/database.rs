use tokio_postgres;
use serde_json::Value;
use crate::authentication::{Token, AppToken};
use std::time::{SystemTime, UNIX_EPOCH};
use chrono::{DateTime, Utc};

pub struct PublicPlaylist{
    pub id:  String,
    pub sha256:  Box<[u8]>,
	pub timestamp: DateTime<Utc>,
    pub data:  Value
}

pub type PublicPlaylists = Vec<PublicPlaylist>;

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
		client.execute("CREATE TABLE IF NOT EXISTS public_playlists (playlist_id TEXT, playlist_sha256 BYTEA, timestamp TIMESTAMP, playlist_data JSONB, PRIMARY KEY (playlist_id, ts)) ", &[]),
		client.execute("CREATE TABLE IF NOT EXISTS spotify_tokens (token_value TEXT, user_id TEXT, is_app BOOL, token_type TEXT, duration BIGINT, received_at BIGINT, PRIMARY KEY(user_id))", &[])
	);

}

pub async fn initiliaze_db() -> tokio_postgres::Client{
	let mut client = connect_db().await;

	create_tables(&mut client).await;

	client

}

pub async fn get_token(client: &mut tokio_postgres::Client, user_id: &String) -> Option<Token>{
	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&user_id.as_str()];
	let r = client.query_opt("SELECT token_value, duration, token_type, received_at, is_app FROM spotify_tokens WHERE user_id = $1::TEXT",&params);

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
					received_at: received_at,
					is_app: row.get("is_app"),
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
		&token.is_app,
		&token.token.token_type.as_str(),
		&i64::try_from(token.token.expires_in).unwrap(),
		&i64::try_from(token.received_at).unwrap()
	];

	let _r = client.execute("INSERT INTO spotify_tokens (user_id, token_value, is_app, token_type, duration, received_at) VALUES ($1::TEXT, $2::TEXT, $3::BOOL, $4::TEXT, $5::BIGINT, $6::BIGINT) ON CONFLICT (user_id) DO UPDATE SET token_value = $2::TEXT, is_app = $3::BOOL, token_type = $4::TEXT, duration = $5::BIGINT, received_at = $6::BIGINT", params)
		.await
		.unwrap();
}

pub async fn get_all_public_playlists(client: &mut tokio_postgres::Client) -> PublicPlaylists{
	let mut res = PublicPlaylists::new();
	let r = client.query("SELECT * FROM public_playlists", &[]).await.unwrap();

	for row in r{
		let t = PublicPlaylist{
			id: row.get("playlist_id"),
			sha256: Box::from(row.get::<&str, &[u8]>("playlist_sha256")),
			timestamp: row.get("timestamp"),
			data: row.get("playlist_data")
		};
		res.push(t);
	}

	res
}

pub async fn get_all_latest_public_playlists(client: &mut tokio_postgres::Client) -> PublicPlaylists{
	let mut res = PublicPlaylists::new();
	let r = client.query("SELECT DISTINCT ON (playlist_id) * FROM public_playlists ORDER BY playlist_id, timestamp DESC", &[]).await.unwrap();

	for row in r{
		let t = PublicPlaylist{
			id: row.get("playlist_id"),
			sha256: Box::from(row.get::<&str, &[u8]>("playlist_sha256")),
			timestamp: row.get("timestamp"),
			data: row.get("playlist_data")
		};
		res.push(t);
	}

	res
}

pub async fn set_public_playlist(client: &mut tokio_postgres::Client, playlist: &PublicPlaylist){
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		&playlist.id,
		&&(*playlist.sha256),
		&playlist.timestamp,
		&playlist.data
	];
	let _r = client.execute("INSERT INTO public_playlists VALUES ($1::TEXT, $2::BYTEA, $3::TIMESTAMP, $4::JSONB) ON CONFLICT (playlist_id, timestamp) DO UPDATE SET playlist_sha256 = $2::BYTEA, playlist_data = $4::JSONB", params)
		.await
		.unwrap();
}

pub async fn claim_new_client_id_unicity(client: &mut tokio_postgres::Client, client_id: &String) -> bool{
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		client_id
	];

	let r = client.execute("INSERT INTO spotify_tokens (user_id) VALUES ($1::TEXT) ON CONFLICT (user_id) DO NOTHING", params).await.unwrap();

	match r {
		0 => false,
		1 => true,
		_ => panic!("[DATABASE] More than one row is affected!"),
	}
	
}
