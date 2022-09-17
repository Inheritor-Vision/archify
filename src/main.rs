use authentication::{get_user_tokens_from_code, Token, FullToken};
use database::{veriy_user_from_spot_id, User, set_full_token};
use rand::distributions::Alphanumeric;
use rand::{thread_rng,Rng};
use serde_json::Value;
use spot_api::get_spot_id;
use tokio;
use reqwest::{Client, header};

use std::fs::File;
#[cfg(feature = "proxy")]
use std::fs::File;
use std::io::Read;
#[cfg(feature = "proxy")]
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

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
	let client_id = app_data["client_id"].to_string();
	client_id

}

pub async fn generate_new_user(client: &mut tokio_postgres::Client, spot_id: String) -> User{
	let mut user_id: String;
	let cookie: String = thread_rng()
		.sample_iter(&Alphanumeric)
		.take(64)
		.map(char::from)
		.collect();

	loop {

		user_id = thread_rng()
			.sample_iter(&Alphanumeric)
			.take(32)
			.map(char::from)
			.collect();

		if database::claim_new_user_id_unicity(client, &user_id, &spot_id, &cookie).await {
			break;
		}
	
	}

	User{
		user_id: user_id,
		spot_id: spot_id,
		cookie: cookie
	}

}

pub async fn authenticate_user(mut client: tokio_postgres::Client, mut client_spot: Client, code: String, redirect_uri: String) -> String {

	let token = get_user_tokens_from_code(&mut client_spot, &code, &redirect_uri).await;

	//Immediatly verify if user has already been connected
	let spot_id = get_spot_id(&client_spot, &token.access_token).await;

	let user = match veriy_user_from_spot_id(&mut client, &spot_id).await {
		None => generate_new_user(&mut client, spot_id).await,
		Some(user) => user,
	};

	// Update FullToken anyway because a new refresh token has been issued
	set_full_token(&mut client, &user.user_id, &token).await;

	user.cookie

}

fn is_access_token_expired(token: &Token) -> bool {
	let time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();
	if time < token.received_at + token.token.expires_in {
		true
	}else{
		false
	}
}

fn is_full_token_expired(token: &FullToken) -> bool {
	let time = SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.unwrap()
		.as_secs();
	if time < token.access_token.received_at + token.access_token.token.expires_in {
		true
	}else{
		false
	}
}

#[tokio::main]
async fn main() {

	let headers = initialize_headers();
	let mut client_spot = create_client(headers);
	let app = String::from("archify");

	let mut client = database::initiliaze_db().await;

	let token = database::get_access_token(&mut client, &app).await; 
	let token = match token {
		Some(token) => {
			if is_access_token_expired(&token){
				let l_t = authentication::get_app_token(&mut client_spot).await;
				database::update_access_token(&mut client, &app, &l_t).await;
				l_t
			}else{
				token
			}
		},
		None => {
			let l_t = authentication::get_app_token(&mut client_spot).await;
			database::update_access_token(&mut client, &app, &l_t).await;
			l_t
		}
	};

	let app_data = get_app_data();

	let args = arguments::parse_args();
	match args{
		arguments::Args::NewUser(code, redirect_uri) => println!("Cookie: {}", authenticate_user(client, client_spot, code, redirect_uri).await),
		arguments::Args::NewPlaylist(url) => tokio::spawn(set_new_playlist(client, client_spot, token, url)).await.unwrap(),
		arguments::Args::DeletePlaylist(_) => println!("Not available yet!"),
		arguments::Args::Update => tokio::spawn(update_all_playlists(client, client_spot, token)).await.unwrap(),
		arguments::Args::GetClientId => println!("{}", get_client_id(app_data).await),
	}

}
