use actix_jwt_auth_middleware::FromRequest;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, FromRequest)]
pub struct Claims {
    #[serde(serialize_with = "mongodb::bson::serde_helpers::serialize_object_id_as_hex_string")]
    pub subject_id: ObjectId,
}
