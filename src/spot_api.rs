use std::sync::Arc;

use crate::{authentication::Token, database::{PublicPlaylist, PublicPlaylists}};

use chrono::Utc;
use sha2::{Sha256, Digest};
use serde::{Deserialize};
use serde_json::{Value};
use reqwest::{Client, header};

#[derive(Deserialize)]
struct Id{
	id: String
}

#[derive(Deserialize)]
struct Track {
	track: Id
}

#[derive(Deserialize)]
struct Items {
	items: Vec<Track>
}

#[derive(Deserialize)]
struct Tracks{
	tracks: Items
}

pub async fn get_public_playlist(client: &Client, token: &Token, playlist_id: &String) -> PublicPlaylist{
	let uri = format!("https://api.spotify.com/v1/playlists/{}", &playlist_id);
	let auth_value = format!("{} {}", token.token.token_type, token.token.access_token);
	let timestamp = Utc::now();
	
	let body = client.get(uri)
		.query(&[("fields", "tracks.items(track(id))")])
		.header(header::AUTHORIZATION, &auth_value)
		.send()
		.await
		.unwrap()
		.text()
		.await
		.unwrap();

	let json: Tracks = serde_json::from_str(body.as_str()).unwrap();
	let json_raw: Value = serde_json::from_str(body.as_str()).unwrap();

	let mut hasher = Sha256::new();

	for i in &json.tracks.items{
		hasher.update(i.track.id.as_bytes());
	}

	let sha256 = hasher.finalize();

	let res = PublicPlaylist {
		id: playlist_id.clone(),
		sha256: Box::from(sha256.as_slice()),
		timestamp: timestamp,
		data: json_raw
	};

	res
	
}

pub async fn get_all_public_playlists(client: &Client, token: &Token, playlists: &PublicPlaylists) -> PublicPlaylists{
	let mut res = PublicPlaylists::new();
	let mut handlers = Vec::new();

	let th_client = Arc::new(client.clone());	
	let th_token = Arc::new(token.clone());

	for p in playlists{
		let l_th_client = Arc::clone(&th_client);
		let l_th_token = Arc::clone(&th_token);
		let th_id = Arc::new(p.id.clone());
		handlers.push(tokio::spawn(
			async move{
				let p = get_public_playlist(&l_th_client, &l_th_token, &th_id).await;
				p
			}
		));
	}

	for h in handlers{
		res.push(h.await.unwrap());
	}

	res

}