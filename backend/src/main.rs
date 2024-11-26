mod models;
mod schema;
mod context;

use std::{env, sync::{Arc, RwLock}, time::Duration};

use actix_cors::Cors;
use actix_web::{
    http::header,
    middleware,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};

use context::Context;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler, subscriptions};
use juniper_graphql_ws::ConnectionConfig;

use models::{Retro, SharedRetros};
use schema::{create_schema, Schema};

async fn playground() -> Result<HttpResponse, Error> {
    playground_handler("/graphql", Some("/subscriptions")).await
}

async fn graphiql() -> Result<HttpResponse, Error> {
    graphiql_handler("/graphql", Some("/subscriptions")).await
}

async fn graphql(
    req: HttpRequest,
    payload: web::Payload,
    schema: Data<Schema>,
    context: Data<Context>,
) -> Result<HttpResponse, Error> {
    let context = Context::from_self(&context);
    graphql_handler(&schema, &context, req, payload).await
}

async fn homepage() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("content-type", "text/html"))
        .message_body(
            "<html><h1>juniper_actix/subscription example</h1>\
                   <div>visit <a href=\"/graphiql\">GraphiQL</a></div>\
                   <div>visit <a href=\"/playground\">GraphQL Playground</a></div>\
             </html>",
        )
}

async fn subscriptions(
    req: HttpRequest,
    stream: web::Payload,
    schema: Data<Schema>,
    context: Data<Context>,
) -> Result<HttpResponse, Error> {
    let context = Context::from_self(&context);
    let config = ConnectionConfig::new(context);
    let schema = schema.into_inner();
    // set the keep alive interval to 15 secs so that it doesn't timeout in playground
    // playground has a hard-coded timeout set to 20 secs
    let config = config.with_keep_alive_interval(Duration::from_secs(15));

    subscriptions::ws_handler(req, stream, schema, config).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let retros: SharedRetros = Arc::new(RwLock::new(vec![]));
    let schema = Arc::new(create_schema());

    let context = Arc::new(Context::new(retros.clone()));

    HttpServer::new(move || {
        App::new()
            .app_data(Data::from(retros.clone()))
            .app_data(Data::from(schema.clone()))
            .app_data(Data::from(context.clone()))
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/subscriptions").route(web::get().to(subscriptions)))
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql))
                    .route(web::get().to(graphql)),
            )
            .service(web::resource("/playground").route(web::get().to(playground)))
            .service(web::resource("/graphiql").route(web::get().to(graphiql)))
            .default_service(web::to(homepage))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}