# Note

# TODO

- Change the way // request are done. [doc](https://docs.rs/reqwest/latest/reqwest/struct.Client.html) state that is use Arc internally. Better do something like 10 spawns that share the work (divided in 10 then). Fist get the number of tracks of all playlist, then divide the work betwenn them. Indeed, for now it doesn ot work because if there is a huge amount track in the playlist, then spotify api will only return part of it. Offset & limit will ahve to be used. Better defined limit to know how much tracks will be returned. Try to find the maximum.
- By default, archive all playlist followed by a user
- Find how to show a playlist to a user (Put a playlist in the user account but need modify rights, Create a playlist from archify account (maybe use the family account I have) and share it with the user or simply give the list on the site)
- Handle errors (from spotify API, from posgre data base and Rust in general (aka unwrap))
- Get private playlist (try to find a way to distinguish between private and public to avoid giving the token for nothing)
- Actually test everything
- Refactor code
- Create the library and the server

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

	//let scope = header::HeaderValue::from_str(String::from("playlist-read-private playlist-read-public playlist-read-collaborative user-follow-read user-library-read") // Maybe user-read-private for search ????
	let scope = header::HeaderValue::from_str(String::from("user-read-private user-read-email") // Maybe user-read-private for search ????
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