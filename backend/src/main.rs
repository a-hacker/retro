mod models;
mod schema;
mod context;
mod database;
mod auth;

use std::{collections::HashMap, env, sync::{Arc, RwLock}, time::Duration};

use actix_cors::Cors;
use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp, Authority};
use actix_web::{
    error, middleware, web::{self, Data}, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use auth::Claims;
use config::{Config, Environment, File, FileFormat};
use dotenvy::dotenv;
use juniper::InputValue;
use jwt_compact::{prelude::*, alg::{Hs512, Hs512Key}};

use context::ContextBuilder;
use database::PersistenceManager;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler, subscriptions};
use juniper_graphql_ws::ConnectionConfig;

use derive_more::derive::{Display, Error};

use models::{ServiceMode, SharedRetros, SharedUsers, User};
use mongodb::bson::oid::ObjectId;
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
    context: Data<ContextBuilder>,
    claims: auth::Claims,
) -> Result<HttpResponse, Error> {
    let context_builder = ContextBuilder::from_self(&context);
    
    let active_user = context_builder.persistence_manager.get_user(&claims.subject_id).await.unwrap();
    let context = context_builder.with_active_user(active_user).build();
    
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

#[derive(Debug, Display, Error)]
enum ServiceError {
    #[display("An internal error occurred. Please try again later.")]
    AuthError,
}

impl error::ResponseError for ServiceError {}

fn validate_access_token(token_string: &str, public_key: &Hs512Key) -> Result<Claims, ServiceError> {
    let token = UntrustedToken::new(token_string).map_err(|_| ServiceError::AuthError)?;
    let token: Token<auth::Claims> = Hs512.validator(public_key).validate(&token).map_err(|_| ServiceError::AuthError)?;
    Ok(token.claims().custom.clone())
}

async fn subscriptions(
    req: HttpRequest,
    stream: web::Payload,
    schema: Data<Schema>,
    context: Data<ContextBuilder>,
    public_key: Data<Hs512Key>
) -> Result<HttpResponse, Error> {
    let schema = schema.into_inner();

    subscriptions::ws_handler(req, stream, schema, async move |payload: HashMap<String, InputValue>| {
        // handle_connection_init(payload).await
        let context_builder = ContextBuilder::from_self(&context);
        if let Some(access_token) = payload.get("access_token") {
            let token_string = access_token.as_string_value().ok_or(ServiceError::AuthError)?;
            let claims = validate_access_token(token_string, &public_key.into_inner())?;
            let user = context_builder.persistence_manager.get_user(&claims.subject_id).await.unwrap();
            let new_context = context_builder.with_active_user(user).build();
            Ok(ConnectionConfig::new(new_context).with_keep_alive_interval(Duration::from_secs(15)))
        } else {
            Err(ServiceError::AuthError)
        }
    }).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let config_file = std::env::var("CONFIG_FILE").unwrap_or("services.toml".to_string());
    println!("Starting server from config file at: {:?}", config_file);
    let conf = Config::builder()
        .add_source(File::new(&config_file, FileFormat::Toml).required(false))
        .add_source(Environment::default())
        .build().expect("Failed to build config");
    let service_config: models::ServiceConfig = conf.try_deserialize().expect("Failed to deserialize config");
    let retro_config = service_config.retro.clone().unwrap_or_default();

    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let secret_key = Hs512Key::new(jwt_secret.as_bytes());

    let retros: SharedRetros = Arc::new(RwLock::new(HashMap::new()));
    let default_users = HashMap::from([(ObjectId::new(), User {
        _id: ObjectId::new(),
        username: "admin".to_string(),
    })]);
    let users: SharedUsers = Arc::new(RwLock::new(default_users));

    println!("Starting server in mode: {:?}", retro_config.mode);

    let persistence_manager: PersistenceManager  = match retro_config.mode {
        ServiceMode::Memory => {
            database::PersistenceManager::new_memory(retros.clone(), users.clone())
        }
        ServiceMode::Mongo => {
            database::PersistenceManager::new_mongo(&service_config).await
        }
    };

    let schema = Arc::new(create_schema());

    let context = Arc::new(ContextBuilder::new(persistence_manager));
    
    let address = format!("0.0.0.0:{}", retro_config.port);

    HttpServer::new(move || {
        let authority = Authority::<auth::Claims, Hs512, _, _>::new()
            .refresh_authorizer(|| async move { Err(ServiceError::AuthError.into()) })
            .algorithm(Hs512)
            .time_options(TimeOptions::from_leeway(chrono::Duration::minutes(15)))
            .enable_header_tokens(true)
            .enable_cookie_tokens(false)
            .access_token_name("access_token")
            .verifying_key(secret_key.clone())
            .build()
            .expect("");

        App::new()
            .app_data(Data::from(retros.clone()))
            .app_data(Data::from(schema.clone()))
            .app_data(Data::from(context.clone()))
            .app_data(Data::new(secret_key.clone()))
            .wrap(
                Cors::permissive()
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/api/v1/retro/subscriptions").route(web::get().to(subscriptions)))
            .service(web::resource("/api/v1/retro").route(web::get().to(homepage)))
            .use_jwt(authority, web::scope("")
                .service(
                    web::resource("/api/v1/retro/graphql")
                        .route(web::post().to(graphql))
                        .route(web::get().to(graphql)),
                )
                .service(web::resource("/api/v1/retro/playground").route(web::get().to(playground)))
                .service(web::resource("/api/v1/retro/graphiql").route(web::get().to(graphiql)))
                )
            .default_service(web::to(homepage))
    })
    .bind(address)?
    .run()
    .await
}