use sha2::Sha256;
use tokio;
use reqwest::{Client, header};

#[cfg(feature = "proxy")]
use std::fs::File;
#[cfg(feature = "proxy")]
use std::io::prelude::*;
use std::sync::Arc;

mod authentication;
mod database;
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

fn create_client(headers: header::HeaderMap) -> Client{
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

async fn update_all_playlists(){

	let headers = initialize_headers();
	let mut client_spot = create_client(headers);
	let app = String::from("archify");

	let mut client = database::initiliaze_db().await;

	let token = database::get_token(&mut client, &app).await; 
	let token = match token {
		Some(token) => token,
		None => {
			let l_t = authentication::get_token(&mut client_spot).await;
			database::set_token(&mut client, &app, &l_t).await;
			l_t
		}
	};

	
	let playlists = database::get_all_latest_public_playlists(&mut client).await;
	let received_playlists = spot_api::get_all_public_playlists(&client_spot, &token, &playlists).await;

	let mut iter_old = playlists.iter();
	let mut iter_new = received_playlists.iter();

	for _ in 0..playlists.len(){
		let p = iter_new.next().unwrap();
		let old_sha256 = iter_old.next().unwrap().sha256.as_ref();
		let new_sha256 = p.sha256.as_ref();
		if *old_sha256 != *new_sha256 {
			database::set_public_playlist(&mut client, p).await;
		}
	}


}

#[tokio::main]
async fn main() {

	let handle = tokio::spawn(update_all_playlists());
	handle.await.unwrap();

}
