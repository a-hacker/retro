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
                graphql_handler, graphiql_handler
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
    juniper_rocket::graphiql_source("/graphql", None)
}
