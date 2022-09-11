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
