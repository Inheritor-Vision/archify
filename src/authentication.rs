use base64;
use std::fs::File;
use serde_json::{Value};
use std::io::prelude::*;
use serde::{Deserialize};
use reqwest::{Client, header};
use std::time::{SystemTime, UNIX_EPOCH};

static FORM_TOKEN: [(&str, &str); 1] = 
	[
		("grant_type", "client_credentials"), 
	];

#[derive(Deserialize, Clone)]
pub struct AppToken {
    pub access_token:  String,
    pub expires_in:  u64,
    pub token_type:  String
}

#[derive(Clone)]
pub struct Token {
	pub token: AppToken,
	pub received_at: u64
}

fn get_app_data() -> Value{
	let mut buff = String::new();
	File::open("data/app_data.json")
	.unwrap()
	.read_to_string(&mut buff)
	.unwrap();

	let res: Value = serde_json::from_str(&*buff).unwrap();
	
	res

}

pub async fn get_token(client: &mut Client) -> Token{
	let mut headers = header::HeaderMap::new();
	let app_data = get_app_data();

	let auth_value = format!("Basic {}", base64::encode(format!("{}:{}", app_data["client_id"].as_str().unwrap(), app_data["client_secret"].as_str().unwrap()).as_bytes()));
	let mut auth = header::HeaderValue::from_str(&auth_value).unwrap();
	auth.set_sensitive(true);
	headers.insert(header::AUTHORIZATION, auth);

	let time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();

	let body = client.post("https://accounts.spotify.com/api/token")
		.form(&FORM_TOKEN)
		.headers(headers)
		.send()
		.await
		.unwrap()
		.text()
		.await
		.unwrap();
	
	let app_token: AppToken = serde_json::from_str(&body).unwrap();
	let token = Token {
		token: app_token,
		received_at: time
	};

	token
}