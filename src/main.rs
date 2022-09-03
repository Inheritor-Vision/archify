use tokio;
use reqwest::{Client, header};
use futures::executor::block_on;

#[cfg(feature = "proxy")]
use std::fs::File;
#[cfg(feature = "proxy")]
use std::io::prelude::*;
use std::str::FromStr;

mod authentication;
mod spot_api;

static APP_USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION")
);

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

#[tokio::main]
async fn main() {
	let token: authentication::AppToken;

	let headers = initialize_headers();
	let mut client = get_client(headers);
	
	token = block_on(authentication::get_token(&mut client));
	let playlist = block_on(spot_api::get_public_playlist(&mut client, &token, String::from_str("37i9dQZF1E4xgPGijPOoGv").unwrap()));

	println!("{}", playlist);
	// println!("text: {:?}", token.access_token);

}
