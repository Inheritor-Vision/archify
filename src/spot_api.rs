use crate::authentication::Token;

use reqwest::{Client, header};

pub async fn get_public_playlist(client: &mut Client, token: &Token, playlist_id: String) -> String{
	let uri = format!("https://api.spotify.com/v1/playlists/{}", playlist_id);
	let auth_value = format!("{} {}", token.token.token_type, token.token.access_token);
	
	let body = client.get(uri)
		.query(&[("fields", "tracks.items(track(name,id))")])
		.header(header::AUTHORIZATION, &auth_value)
		.send()
		.await
		.unwrap()
		.text()
		.await
		.unwrap();
	
	body

}