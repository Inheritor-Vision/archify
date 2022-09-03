use tokio;
use serde_json::{Value};
use reqwest::{Client, header};
use futures::executor::block_on;

#[cfg(feature = "proxy")]
use std::fs::File;
#[cfg(feature = "proxy")]
use std::io::prelude::*;

static APP_USER_AGENT: &str = concat!(
	env!("CARGO_PKG_NAME"),
	"/",
	env!("CARGO_PKG_VERSION")
);

// q=remaster track:Doxy artist:Miles Davis&type=track,artist&market=ES&limit=10&offset=5
static QUERY: [(&str, &str); 5] = 
	[
		("q", "remaster track:Doxy artist:Miles Davis"), 
		("type", "track,artist"), 
		("market", "FR"), 
		("limit", "10"),
		("offset", "5")
	];

fn initialize_headers() -> header::HeaderMap{
	let mut headers = header::HeaderMap::new();

	headers.insert(header::ACCEPT, header::HeaderValue::from_static("application/json"));
	let mut auth = header::HeaderValue::from_static("");
	auth.set_sensitive(true);
	headers.insert(header::AUTHORIZATION, auth);
	headers.insert(header::CONTENT_TYPE, header::HeaderValue::from_static("application/json"));

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

async fn get_content(client: Client) -> (Client, String){
	let body = client.get("https://api.spotify.com/v1/search")
		.query(&QUERY)
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
	
	(_, body) = block_on(get_content(client));

	let res: Value = serde_json::from_str(&*body).unwrap();

	println!("text: {:?}", res);

}
