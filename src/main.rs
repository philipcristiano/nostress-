use axum::{
    routing::{get, post},
    extract::Path,
    http::StatusCode,
    // response::IntoResponse,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use rss::{ChannelBuilder, Item, ItemBuilder, Guid};

use nostr_sdk::prelude::*;


#[tokio::main]
async fn main() {
    // initialize tracing
    tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root))
        .route("/users/:user_id/rss", get(user_rss))
        // `POST /users` goes to `create_user`
        .route("/users", post(create_user));

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}

async fn user_rss(Path(user_id): Path<String>) -> (StatusCode, String) {

    let profile = nostr_sdk::nips::nip05::get_profile(&user_id, None).await.unwrap();

    let profile_key = Keys::from_public_key(profile.public_key);
    println!("profile: {:?}", profile);
    let client = Client::new(&profile_key);
    for r in profile.relays {
        println!("relay: {r}");
        client.add_relay(r, None).await.unwrap();
    }

    client.connect().await;

    let start = Timestamp::from(0);
    let subscription = Filter::new()
        .pubkeys(vec![profile.public_key])
        .since(start);

    let timeout = Duration::from_secs(10);
    let events = client
        .get_events_of(vec![subscription], Some(timeout))
        .await
        .unwrap();
    client.disconnect().await.unwrap();

    let mut channel = ChannelBuilder::default()
        .title("Channel Title")
        .link("http://example.com")
        .description("An RSS feed.")
        .build();

    let mut items: Vec<Item> = Vec::new();

    for e in events {
        let c = e.content;
        println!("Event: {c}");
        let mut guid = Guid::default();
        guid.set_value(e.id.to_string());
        let i = ItemBuilder::default()
            .content(c)
            .guid(guid)
            .build();
        items.push(i);
    }

    channel.set_items(items);

    (StatusCode::OK, channel.to_string())

}

async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> (StatusCode, Json<User>) {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED, Json(user))
}

// the input to our `create_user` handler
#[derive(Deserialize)]
struct CreateUser {
    username: String,
}

// the output to our `create_user` handler
#[derive(Serialize)]
struct User {
    id: u64,
    username: String,
}
