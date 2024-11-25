// src/main.rs

#[macro_use]
extern crate rocket;

mod models;
mod schema;
mod context;

use rocket::{response::content::RawHtml, routes, State, http::Method};
use rocket_cors::{AllowedOrigins, CorsOptions};
use schema::{create_schema, Schema};
use context::Context;
use juniper_rocket::{GraphQLResponse, GraphQLRequest};
use models::SharedRetros;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use futures::{SinkExt, Stream};
use rocket::tokio::select;
use juniper_graphql_ws::graphql_transport_ws::Connection;

#[launch]
fn rocket() -> _ {
    // Initialize shared retrospectives state
    let retros: SharedRetros = Arc::new(RwLock::new(vec![]));
    
    // Create Juniper Context
    let context = Context::new(Arc::clone(&retros));
    
    // Create Juniper Schema
    let schema = create_schema();

    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true);
    
    rocket::build()
        .manage(retros)
        .manage(context)
        .manage(schema)
        .attach(cors.to_cors().unwrap())
        // Route for GraphQL
        .mount(
            "/",
            routes![
                graphql_handler, graphiql_handler, subscriptions_handler
            ]
        )
}

#[post("/graphql", data = "<request>")]
async fn graphql_handler(
    request: GraphQLRequest,
    schema: &State<Schema>,
    context: &State<Context>,
) -> GraphQLResponse {
    request.execute(&schema, context).await
}

#[get("/graphiql")]
fn graphiql_handler() -> RawHtml<String> {
    juniper_rocket::graphiql_source("/graphql", Some("/subscriptions"))
}


#[get("/subscriptions")]
fn subscriptions_handler(
    schema: &State<Schema>,
    context: &State<Context>,
    ws: rocket_ws::WebSocket
) -> rocket_ws::Stream!['static] {
    let mut rx_card = context.card_addition_sender.subscribe();
    let mut rx_user = context.user_update_sender.subscribe();

    let conn_config = juniper_graphql_ws::ConnectionConfig::new(*(context.inner().clone()));
    let schema = Arc::new(*schema.inner());

    let conn= Connection::new(schema, conn_config);

    let sock = ws.config(rocket_ws::Config {
        ..Default::default()
    });

    rocket_ws::Stream! { sock =>
        loop {
            select! {
                result = rx_card.recv() => {
                    match result {
                        Ok(update) => {
                            let message = serde_json::to_string(&update).unwrap();
                            yield rocket_ws::Message::Text(message);
                        },
                        Err(e) => {
                            eprintln!("Error receiving card update: {}", e);
                            break;
                        }
                    }
                },
                result = rx_user.recv() => {
                    match result {
                        Ok(update) => {
                            let message = serde_json::to_string(&update).unwrap();
                            yield rocket_ws::Message::Text(message);
                        },
                        Err(e) => {
                            eprintln!("Error receiving user update: {}", e);
                            break;
                        }
                    }
                },
                else => break,
            }
        }
    }
}