use std::{sync::Arc, convert::TryInto};
use std::process::exit;

use crate::{authentication::Token, database::{Playlist, Playlists, User}};

use chrono::Utc;
use url::{Url};
use sha2::{Sha256, Digest};
use serde::{Deserialize};
use serde_json::{Value};
use reqwest::{Client, header::{self, HeaderMap}};

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

fn add_user_authorization(headers: &mut header::HeaderMap, token: &Token){
	let auth_value = format!("Basic {}", token.token.access_token);

	let mut auth = header::HeaderValue::from_str(&auth_value).unwrap();
	auth.set_sensitive(true);

	headers.insert(header::AUTHORIZATION, auth);
}

pub async fn get_spot_id(client: &Client, token: &Token) -> String {
	let mut headers = HeaderMap::new();
	let uri = "https://api.spotify.com/v1/me";

	add_user_authorization(&mut headers, &token);

	let body = client.get(uri)
		.headers(headers)
		.send()
		.await 
		.unwrap()
		.text()
		.await
		.unwrap();
	
	let json_raw: Value = serde_json::from_str(body.as_str()).unwrap();

	let res = json_raw["id"].to_string();

	res

}

pub async fn get_public_playlist(client: &Client, token: &Token, playlist_id: &String) -> Playlist{
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

	let res = Playlist {
		id: playlist_id.clone(),
		sha256: Box::from(sha256.as_slice()),
		timestamp: timestamp,
		data: json_raw
	};

	res
	
}

pub async fn get_all_public_playlists(client: &Client, token: &Token, playlists: &Playlists) -> Playlists{
	let mut res = Playlists::new();
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

pub async fn get_all_followed_playlist_id(client: &Client, user_token: &Token, user: User) -> Vec<String>{
	let mut headers = HeaderMap::new();
	let mut playlists_id = Vec::<String>::new();
	let uri = "https://api.spotify.com/v1/me/playlists";

	add_user_authorization(&mut headers, &user_token);

	let mut cpt:u64 = 0;

	loop{
		let cpt_str = cpt.to_string();
		let query = &[("offset", cpt_str.as_str()), ("limit", "20")];
		let body = client.get(uri)
			.query(query)
			.headers(headers.clone())
			.send()
			.await
			.unwrap()
			.text()
			.await
			.unwrap();

		let json_raw: Value = serde_json::from_str(body.as_str()).unwrap();
		let max = json_raw["total"].as_u64().unwrap();
		let items: &Vec<Value> = json_raw["items"].as_array().unwrap();
		let current_size: u64 = items.len().try_into().unwrap();

		for val in items{
			let id = val["id"].to_string();
			playlists_id.push(id);
		}

		if cpt + current_size >= max{
			break;
		}else{
			cpt += 20;
		}
	}

	playlists_id

}

pub fn parse_spotify_url(url: &String) -> String{
	let parsed_url = Url::parse(url).unwrap();
	let segments = parsed_url.path_segments().map(|c| c.collect::<Vec<_>>()).unwrap();
	let res;
	if segments[0] == "playlist"{
		res = String::from(segments[1]);
	}else{
		println!("Error in parsing the URL");
		exit(1);
	}
	res
}