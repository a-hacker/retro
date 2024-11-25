// src/context.rs

use crate::models::SharedRetros;
use juniper::Context as JuniperContext;

// Define the Context struct that holds the shared state
pub struct Context {
    pub retros: SharedRetros,
}

impl Context {
    pub fn new(retros: SharedRetros) -> Self {
        Context { retros }
    }
}

impl JuniperContext for Context {}
