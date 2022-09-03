use tokio;
use base64;
use serde_json::{Value};
use reqwest::{Client, header};
use futures::executor::block_on;

use std::fs::File;
use std::io::prelude::*;

static APP_USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION")
);

static FORM_TOKEN: [(&str, &str); 1] = 
	[
		("grant_type", "client_credentials"), 
	];


fn initialize_headers() -> header::HeaderMap{
	let mut headers = header::HeaderMap::new();

	headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));

	headers

}

#[cfg(feature = "proxy")]
fn get_certificate() -> reqwest::Certificate{
	let mut buff = Vec::new();
	File::open("data/cacert.der").unwrap().read_to_end(&mut buff).unwrap();
	let cert = reqwest::Certificate::from_der(&buff).unwrap();

	cert
}

fn get_client(headers: header::HeaderMap) -> Client{
	let client: reqwest::ClientBuilder;

	#[cfg(not(feature = "proxy"))]{
		client = Client::builder()
			.user_agent(APP_USER_AGENT)
			.default_headers(headers);
	}
	
	#[cfg(feature = "proxy")]{
		client = Client::builder()
			.user_agent(APP_USER_AGENT)
			.default_headers(headers)
			.proxy(reqwest::Proxy::http("http://127.0.0.1:8080").unwrap())
			.proxy(reqwest::Proxy::https("http://127.0.0.1:8080").unwrap())
			.add_root_certificate(get_certificate());
	} 
	
	client.build().unwrap()

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

async fn get_token(client: Client) -> (Client, String){
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

#[tokio::main]
async fn main() {
	let body: String;

	let headers = initialize_headers();
	let client = get_client(headers);
	
	(_, body) = block_on(get_token(client));

	let res: Value = serde_json::from_str(&*body).unwrap();

	println!("text: {:?}", res);

}
