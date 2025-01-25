#![feature(async_closure)]

mod models;
mod schema;
mod context;
mod database;
mod auth;

use std::{collections::HashMap, env, sync::{Arc, RwLock}, time::Duration};

use actix_cors::Cors;
use actix_jwt_auth_middleware::{use_jwt::UseJWTOnApp, AuthError, AuthResult, Authority, TokenSigner};
use actix_web::{
    error, middleware, web::{self, Data}, App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use auth::Claims;
use ed25519_compact::{KeyPair, PublicKey};
use juniper::InputValue;
use jwt_compact::{prelude::*, alg::Ed25519};

use context::ContextBuilder;
use database::PersistenceManager;
use juniper_actix::{graphiql_handler, graphql_handler, playground_handler, subscriptions};
use juniper_graphql_ws::ConnectionConfig;

use derive_more::derive::{Display, Error};

use models::{ServiceConfig, ServiceMode, SharedRetros, SharedUsers};
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

fn validate_access_token(token_string: &str, public_key: &PublicKey) -> Result<Claims, ServiceError> {
    let token = UntrustedToken::new(token_string).map_err(|_| ServiceError::AuthError)?;
    let token: Token<auth::Claims> = Ed25519.validator(public_key).validate(&token).map_err(|_| ServiceError::AuthError)?;
    Ok(token.claims().custom.clone())
}

async fn subscriptions(
    req: HttpRequest,
    stream: web::Payload,
    schema: Data<Schema>,
    context: Data<ContextBuilder>,
    public_key: Data<PublicKey>
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

async fn login(request: web::Json<models::LoginRequest>, context: Data<ContextBuilder>, cookie_signer: web::Data<TokenSigner<Claims, Ed25519>>) -> AuthResult<HttpResponse> {
    let user = context.persistence_manager.validate_user(&request).await.map_err(|_| AuthError::NoTokenSigner)?;
    let claim = auth::Claims::new(user._id);
    Ok(HttpResponse::Ok()
        .append_header((cookie_signer.access_token_name(), cookie_signer.create_access_header_value(&claim)?))
        .append_header((cookie_signer.refresh_token_name(), cookie_signer.create_refresh_header_value(&claim)?))
        .body("You are now logged in"))
}

async fn create_user(request: web::Json<models::LoginRequest>, context: Data<ContextBuilder>, cookie_signer: web::Data<TokenSigner<Claims, Ed25519>>) -> AuthResult<HttpResponse> {
    let new_user = models::User {
        _id: ObjectId::new(),
        username: request.username.clone(),
    };
    let user = context.persistence_manager.create_user(new_user).await.map_err(|_| AuthError::NoTokenSigner)?;
    let claim = auth::Claims::new(user._id);
    Ok(HttpResponse::Ok()
        .append_header((cookie_signer.access_token_name(), cookie_signer.create_access_header_value(&claim)?))
        .append_header((cookie_signer.refresh_token_name(), cookie_signer.create_refresh_header_value(&claim)?))
        .body("You are now logged in"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let KeyPair {
        pk: public_key,
        sk: secret_key,
    } = KeyPair::generate();

    let service_config: ServiceConfig = confy::load("retro", Some("services")).expect("Failed to load configuration");
    let retros: SharedRetros = Arc::new(RwLock::new(HashMap::new()));
    let users: SharedUsers = Arc::new(RwLock::new(HashMap::new()));

    println!("Starting server from config file at: {:?}", confy::get_configuration_file_path("retro", Some("services")).unwrap());
    println!("Starting server in mode: {:?}", service_config.mode);

    let persistence_manager: PersistenceManager  = match service_config.mode {
        ServiceMode::MEMORY => {
            database::PersistenceManager::new_memory(&service_config, retros.clone(), users.clone())
        }
        ServiceMode::MONGO => {
            database::PersistenceManager::new_mongo(&service_config).await
        }
    };

    let schema = Arc::new(create_schema());

    let context = Arc::new(ContextBuilder::new(persistence_manager));

    HttpServer::new(move || {
        let authority = Authority::<auth::Claims, Ed25519, _, _>::new()
            .refresh_authorizer(|| async move { Ok(()) })
            .enable_header_tokens(true)
            .enable_cookie_tokens(false)
            .access_token_name("Authorization")
            .token_signer(Some(
                TokenSigner::new()
                    .signing_key(secret_key.clone())
                    .algorithm(Ed25519)
                    .build()
                    .expect(""),
            ))
            .verifying_key(public_key)
            .build()
            .expect("");

        App::new()
            .app_data(Data::from(retros.clone()))
            .app_data(Data::from(schema.clone()))
            .app_data(Data::from(context.clone()))
            .app_data(Data::new(public_key))
            .wrap(
                Cors::permissive()
            )
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .service(web::resource("/login").route(web::post().to(login)))
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/subscriptions").route(web::get().to(subscriptions)))
            .use_jwt(authority, web::scope("")
                .service(
                    web::resource("/graphql")
                        .route(web::post().to(graphql))
                        .route(web::get().to(graphql)),
                )
                .service(web::resource("/playground").route(web::get().to(playground)))
                .service(web::resource("/graphiql").route(web::get().to(graphiql)))
                )
            .default_service(web::to(homepage))
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}