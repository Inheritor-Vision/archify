use crate::{authentication::Token, database::PublicPlaylist};

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

pub async fn get_public_playlist(client: &mut Client, token: &Token, playlist_id: String) -> PublicPlaylist{
	let uri = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);
	let auth_value = format!("{} {}", token.token.token_type, token.token.access_token);
	
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
		id: playlist_id,
		sha256: Box::from(sha256.as_slice()),
		data: json_raw
	};

	res
	
}