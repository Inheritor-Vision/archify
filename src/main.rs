use authentication::get_user_tokens_from_code;
use rand::distributions::Alphanumeric;
use rand::{thread_rng,Rng};
use serde_json::Value;
use tokio;
use reqwest::{Client, header};

use std::fs::File;
#[cfg(feature = "proxy")]
use std::fs::File;
use std::io::Read;
#[cfg(feature = "proxy")]
use std::io::prelude::*;

mod authentication;
mod database;
mod spot_api;
mod arguments;

static APP_USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION")
);

fn get_app_data() -> Value{
	let mut buff = String::new();
	File::open("data/app_data.json")
	.unwrap()
	.read_to_string(&mut buff)
	.unwrap();

	let res: Value = serde_json::from_str(&*buff).unwrap();
	
	res

}

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

async fn update_all_playlists(mut client: tokio_postgres::Client, client_spot: Client, token: authentication::Token){
	
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

async fn set_new_playlist(mut client: tokio_postgres::Client, client_spot: Client, token: authentication::Token, url: String){

	let playlist_id = spot_api::parse_spotify_url(&url);

	let playlist = spot_api::get_public_playlist(&client_spot, &token, &playlist_id).await;

	database::set_public_playlist(&mut client, &playlist).await;

}

pub async fn get_client_id(app_data: Value) -> String{
	let client_id = app_data["fd"].to_string();
	client_id

}

pub async fn generate_new_user_id(client: &mut tokio_postgres::Client) -> String{
	let mut client_id;
	let cookie = thread_rng()
		.sample_iter(&Alphanumeric)
		.take(64)
		.map(char::from)
		.collect();

	loop {

		client_id = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(32)
			.map(char::from)
			.collect();

		if database::claim_new_user_id_unicity(client, &client_id, &cookie).await {
			break;
		}
	
	}

	client_id
}

pub async fn authenticate_user(mut client: tokio_postgres::Client, mut client_spot: Client, token: authentication::Token, code: String, redirect_uri: String){

	let token = get_user_tokens_from_code(&mut client_spot, &code, &redirect_uri).await;




}

#[tokio::main]
async fn main() {

	let headers = initialize_headers();
	let mut client_spot = create_client(headers);
	let app = String::from("archify");

	let mut client = database::initiliaze_db().await;

	let token = database::get_access_token(&mut client, &app).await; 
	let token = match token {
		Some(token) => token,
		None => {
			let l_t = authentication::get_app_token(&mut client_spot).await;
			database::update_access_token(&mut client, &app, &l_t).await;
			l_t
		}
	};

	let app_data = get_app_data();

	let args = arguments::parse_args();
	match args{
		arguments::Args::NewUser(_) => println!("Not available yet!"),
		arguments::Args::NewPlaylist(url) => tokio::spawn(set_new_playlist(client, client_spot, token, url)).await.unwrap(),
		arguments::Args::DeletePlaylist(_) => println!("Not available yet!"),
		arguments::Args::Update => tokio::spawn(update_all_playlists(client, client_spot, token)).await.unwrap(),
		arguments::Args::NewUserId => println!("{}", get_client_id(app_data).await),
	}

}
