use base64;
use serde::{Deserialize};
use reqwest::{Client, header};
use serde_json::Value;
use std::{time::{SystemTime, UNIX_EPOCH}, collections::HashMap};

use crate::get_app_data;

static FORM_TOKEN: [(&str, &str); 1] = 
	[
		("grant_type", "client_credentials"), 
	];

#[derive(Deserialize, Clone)]
pub struct AppToken {
    pub access_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

#[derive(Clone)]
pub struct Token {
	pub token: AppToken,
	pub received_at: u64,
	pub client_id: String,
}

pub struct FullToken {
	pub access_token: Token,
	pub refresh_token: String,
}

fn add_app_authorization(headers: &mut header::HeaderMap, app_data: &Value){
	let auth_value = format!("Basic {}", base64::encode(format!("{}:{}", app_data["client_id"].as_str().unwrap(), app_data["client_secret"].as_str().unwrap()).as_bytes()));

	let mut auth = header::HeaderValue::from_str(&auth_value).unwrap();
	auth.set_sensitive(true);

	headers.insert(header::AUTHORIZATION, auth);
}

pub async fn get_user_tokens_from_code(client: &mut Client, code: &String, redirect_uri: &String) -> FullToken{
	let mut headers = header::HeaderMap::new();
	let mut form = HashMap::new();
	let app_data = get_app_data();

	form.insert("grant_type", "authorization_code");
	form.insert("code", code.as_str());
	form.insert("redirect_uri", redirect_uri.as_str());

	add_app_authorization(&mut headers, &app_data);

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

	let r: Value = serde_json::from_str(&body).unwrap();

	let token = FullToken {
		refresh_token: r["refresh_token"].to_string(),
		access_token: Token { 
			token: AppToken { 
				access_token: r["access_token"].to_string(), 
				expires_in: r["expires_in"].as_u64().unwrap(), 
				token_type: r["token_type"].to_string() 
			}, 
			received_at: time, 
			client_id: String::from("-1") // To be handled better with an Option
		}
	};

	token

}

pub async fn get_app_token(client: &mut Client) -> Token{
	let mut headers = header::HeaderMap::new();
	let app_data = get_app_data();

	add_app_authorization(&mut headers, &app_data);

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
		received_at: time,
		client_id: app_data["client_id"].to_string(), 
	};

	token
}