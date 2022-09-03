use base64;
use std::fs::File;
use serde_json::{Value};
use std::io::prelude::*;
use reqwest::{Client, header};

static FORM_TOKEN: [(&str, &str); 1] = 
	[
		("grant_type", "client_credentials"), 
	];

fn get_app_data() -> Value{
	let mut buff = String::new();
	File::open("data/app_data.json")
	.unwrap()
	.read_to_string(&mut buff)
	.unwrap();

	let res: Value = serde_json::from_str(&*buff).unwrap();
	
	res

}

pub async fn get_token(client: Client) -> (Client, String){
	let mut headers = header::HeaderMap::new();
	let app_data = get_app_data();

	let auth_value = format!("Basic {}", base64::encode(format!("{}:{}", app_data["client_id"].as_str().unwrap(), app_data["client_secret"].as_str().unwrap()).as_bytes()));
	let mut auth = header::HeaderValue::from_str(&auth_value).unwrap();
	auth.set_sensitive(true);
	headers.insert(header::AUTHORIZATION, auth);

	let body = client.post("https://accounts.spotify.com/api/token")
		.form(&FORM_TOKEN)
		.headers(headers)
		.send()
		.await
		.unwrap()
		.text()
		.await
		.unwrap();

	(client, body)
}