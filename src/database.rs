use tokio_postgres;
use serde_json::Value;
use crate::{authentication::{Token, AppToken, FullToken}, get_app_data};
use chrono::{DateTime, Utc};

pub struct PublicPlaylist {
    pub id:  String,
    pub sha256:  Box<[u8]>,
	pub timestamp: DateTime<Utc>,
    pub data:  Value
}

pub type PublicPlaylists = Vec<PublicPlaylist>;

pub struct PrivatePlaylist {
    pub id:  String,
	pub user_id: String,
    pub sha256:  Box<[u8]>,
	pub timestamp: DateTime<Utc>,
    pub data:  Value
}

pub type PrivatePlaylists = Vec<PrivatePlaylist>;

pub struct User {
	pub user_id: String,
	pub spot_id: String,
	pub cookie: String,
}

async fn connect_db() -> tokio_postgres::Client{
	let app_data = get_app_data();
	let config = format!("host={0} user={1} dbname={2}", app_data["host"], app_data["user"], app_data["dbname"]);

	let (client, connection) = tokio_postgres::connect(config.as_str(), tokio_postgres::NoTls)
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

	let (_r1, _r2, _r3, _r4) = futures::join!(
		client.execute("CREATE TABLE IF NOT EXISTS public_playlists (playlist_id TEXT, playlist_sha256 BYTEA, timestamp TIMESTAMP, playlist_data JSONB, PRIMARY KEY (playlist_id, ts)) ", &[]),
		client.execute("CREATE TABLE IF NOT EXISTS private_playlists (playlist_id TEXT, user_id TEXT, playlist_sha256 BYTEA, timestamp TIMESTAMP, playlist_data JSONB, PRIMARY KEY (playlist_id, user_id, ts), CONSTRAINT fk_user_id FOREIGN KEY(user_id) REFERENCES users(user_id)) ", &[]),
		client.execute("CREATE TABLE IF NOT EXISTS spotify_tokens (refresh_token_value TEXT, user_id TEXT, access_token_value TEXT, token_type TEXT, duration BIGINT, received_at BIGINT, PRIMARY KEY(user_id), CONSTRAINT fk_user_id FOREIGN KEY(users_id) REFERENCES users(user_id))", &[]),
		client.execute("CREATE TABLE IF NOT EXISTS users (client_id TEXT, spot_id TEXT, cookie TEXT, PRIMARY KEY (user_id))", &[]),
	);

}

pub async fn initiliaze_db() -> tokio_postgres::Client{
	let mut client = connect_db().await;

	create_tables(&mut client).await;

	client

}

pub async fn get_access_token(client: &mut tokio_postgres::Client, user_id: &String) -> Option<Token>{
	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&user_id.as_str()];
	let r = client.query_opt("SELECT access_token_value, duration, token_type, received_at FROM spotify_tokens WHERE user_id = $1::TEXT",&params);

	let r = r.await.unwrap();

	let token = match r {
		None => None,
		Some(row) => {
			let received_at = u64::try_from(row.get::<&str,i64>("received_at")).unwrap();
			let duration = u64::try_from(row.get::<&str,i64>("duration")).unwrap();
			if !row.is_empty(){
				let t = Token {
					token: AppToken{
						access_token: row.get("access_token_value"),
						expires_in: duration,
						token_type: row.get("token_type")
					},
					received_at: received_at,
					client_id: user_id.clone(),
				};
				Some(t)	
			}else{
				None
			}
		}
	};

	token
}

pub async fn get_full_token(client: &mut tokio_postgres::Client, user_id: &String) -> Option<FullToken>{

	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[&user_id.as_str()];
	let r = client.query_opt("SELECT * FROM spotify_tokens WHERE user_id = $1::TEXT",&params)
		.await
		.unwrap();

	let token = match r {
		None => None,
		Some(row) => {
			let received_at = u64::try_from(row.get::<&str,i64>("received_at")).unwrap();
			let duration = u64::try_from(row.get::<&str,i64>("duration")).unwrap();
			if !row.is_empty() {
				let t = FullToken{
					access_token: Token {
						token: AppToken{
							access_token: row.get("access_token_value"),
							expires_in: duration,
							token_type: row.get("token_type")
						},
						received_at: received_at,
						client_id: user_id.clone(),
					},
					refresh_token: row.get("refresh_token"),
				};
				Some(t)	
			}else{
				None
			}
		}
	};

	token
}

pub async fn update_access_token(client: &mut tokio_postgres::Client, user_id: &String, token: &Token){
	let params:&[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		&user_id.as_str(), 
		&token.token.access_token.as_str(),
		&token.token.token_type.as_str(),
		&i64::try_from(token.token.expires_in).unwrap(),
		&i64::try_from(token.received_at).unwrap()
	];

	let _r = client.execute("UPDATE spotify_tokens SET (access_token_value, token_type, duration, received_at) = ($2::TEXT, $3::BOOL, $4::TEXT, $5::BIGINT, $6::BIGINT) WHERE user_id = $1::TEXT", params)
		.await
		.unwrap();
}

pub async fn set_full_token(client: &mut tokio_postgres::Client, user_id: &String, token: &FullToken){
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		user_id,
		&token.refresh_token,
		&token.access_token.token.access_token,
		&token.access_token.token.token_type,
		&i64::try_from(token.access_token.token.expires_in).unwrap(),
		&i64::try_from(token.access_token.received_at).unwrap()
	];
	let _r = client.execute("INSERT INTO spotify_tokens (user_id, refresh_token_value, access_token_value, token_type, duration, received_at) VALUES ($1::TEXT, $2::TEXT, $3::TEXT, $4::TEXT, $5::BIGINT, $6::BIGINT) ON CONFLICT (user_id) DO UPDATE SET refresh_token_value = $2::TEXT, access_token_value = $3::TEXT, token_type = $4::TEXT, duration = $5::BIGINT, received_at = $6::BIGINT", params)
		.await
		.unwrap();
}

// Outdated because there is multiple version of a playlist (That is the principle of this application)
// pub async fn get_all_public_playlists(client: &mut tokio_postgres::Client) -> PublicPlaylists{
// 	let mut res = PublicPlaylists::new();
// 	let r = client.query("SELECT * FROM public_playlists", &[]).await.unwrap();
// 
// 	for row in r{
// 		let t = PublicPlaylist{
// 			id: row.get("playlist_id"),
// 			sha256: Box::from(row.get::<&str, &[u8]>("playlist_sha256")),
// 			timestamp: row.get("timestamp"),
// 			data: row.get("playlist_data")
// 		};
// 		res.push(t);
// 	}
// 
// 	res
// }
// 
// pub async fn get_all_private_playlists(client: &mut tokio_postgres::Client) -> PrivatePlaylists{
// 	let mut res = PrivatePlaylists::new();
// 	let r = client.query("SELECT * FROM private_playlists", &[]).await.unwrap();
// 
// 	for row in r{
// 		let t = PrivatePlaylist{
// 			id: row.get("playlist_id"),
// 			user_id: row.get("user_id"),
// 			sha256: Box::from(row.get::<&str, &[u8]>("playlist_sha256")),
// 			timestamp: row.get("timestamp"),
// 			data: row.get("playlist_data")
// 		};
// 		res.push(t);
// 	}
// 
// 	res
// }

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

pub async fn get_all_latest_private_playlists(client: &mut tokio_postgres::Client) -> PrivatePlaylists{
	let mut res = PrivatePlaylists::new();
	let r = client.query("SELECT DISTINCT ON (playlist_id) * FROM private_playlists ORDER BY playlist_id, timestamp DESC", &[]).await.unwrap();

	for row in r{
		let t = PrivatePlaylist{
			id: row.get("playlist_id"),
			user_id: row.get("user_id"),
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
	let _r = client.execute("INSERT INTO public_playlists VALUES ($1::TEXT, $2::BYTEA, $3::TIMESTAMP, $4::JSONB)", params)
		.await
		.unwrap();
}

pub async fn set_private_playlist(client: &mut tokio_postgres::Client, playlist: &PrivatePlaylist, user_id: &String){
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		&playlist.id,
		&user_id,
		&&(*playlist.sha256),
		&playlist.timestamp,
		&playlist.data
	];
	let _r = client.execute("INSERT INTO private_playlists VALUES ($1::TEXT, $2::TEXT, $3::BYTEA, $4::TIMESTAMP, $5::JSONB)", params)
		.await
		.unwrap();
}

pub async fn claim_new_user_id_unicity(client: &mut tokio_postgres::Client, user_id: &String, spot_id: &String, cookie: &String) -> bool{
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		user_id,
		spot_id,
		cookie,
	];

	let r = client.execute("INSERT INTO users (user_id, spot_id, cookie) VALUES ($1::TEXT, $2::TEXT, $3::TEXT) ON CONFLICT (user_id) DO NOTHING", params).await.unwrap();

	match r {
		0 => false,
		1 => true,
		_ => panic!("[DATABASE] More than one row is affected!"),
	}
	
}

pub async fn veriy_user_from_spot_id(client: &mut tokio_postgres::Client, spot_id: &String) -> Option<User> {
	let params: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &[
		spot_id,
	];

	let r = client.query_opt("SELECT * FROM users WHERE spot_id = $1::TEXT", params)
		.await
		.unwrap();

	let res = match r {
		None => None,
		Some(row) => Some(
			User {
				user_id: row.get("user_id"),
				spot_id: row.get("spot_id"),
				cookie: row.get("cookie"),
			}
		)
	};

	res

}