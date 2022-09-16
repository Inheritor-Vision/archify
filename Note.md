# Note

# TODO

- Ad new playlist (share token from spotify, but not client from spot or db)

# Knowledge

## Dashboard

See [here](https://developer.spotify.com/dashboard/applications/01d4bc1059ff4078b507a6efff9910ae).

## Test request

See online API request generator [here](https://developer.spotify.com/console/).

## Public API

General idea [here](https://community.spotify.com/t5/Spotify-for-Developers/Accessing-Spotify-API-without-Logging-In/td-p/5063968).
Get client (i.e. app, not end user) token [here](https://developer.spotify.com/documentation/general/guides/authorization/client-credentials/).
Example in JS [here](https://github.com/spotify/web-api-auth-examples/tree/master/client_credentials).

## Parameters for query_raw, execute_raw etc.

```Rust
let mut params: Vec<&(dyn tokio_postgres::types::ToSql + Sync)> = Vec::new();
params.push(&user_id as &(dyn tokio_postgres::types::ToSql + Sync));

let params = params.iter().map(|s| (*s as &(dyn tokio_postgres::types::ToSql + Sync)));
```

## Rust partial implem of redirect

```Rust
pub async fn get_authorize_code(client: &mut tokio_postgres::Client, client_spot: &Client, token: &Token, user_id: &String){
	let mut headers = header::HeaderMap::new();

	let state = header::HeaderValue::from_str(thread_rng()
			.sample_iter(&Alphanumeric)
			.take(32)
			.map(char::from)
			.collect::<String>()
			.as_str())
		.unwrap();

	let scope = header::HeaderValue::from_str(String::from("playlist-modify-private playlist-read-private playlist-read-collaborative user-read-private")
			.as_str())
		.unwrap();

	let client_id = header::HeaderValue::from_str(
		generate_new_client_id(client)
		.await
		.as_str()
	).unwrap();

	let response_type = header::HeaderValue::from_str("code").unwrap();

	let redirect_uri = header::HeaderValue::from_str("http://localhost:5901/authorize").unwrap();

	headers.insert("Scope", scope);
	headers.insert("State", state);
	headers.insert("Client_id", client_id);
	headers.insert("Response_type", response_type);
	headers.insert("Redirect_uri", redirect_uri);
}
```