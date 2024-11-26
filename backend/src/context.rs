// src/context.rs

use crate::models::{SharedRetros, SubscriptionUpdate };
use juniper::Context as JuniperContext;
use tokio::sync::broadcast;

// Define the Context struct that holds the shared state
pub struct Context {
    pub retros: SharedRetros,
    pub card_addition_sender: broadcast::Sender<SubscriptionUpdate>,
    pub user_update_sender: broadcast::Sender<SubscriptionUpdate>,
    pub card_addition_receiver: broadcast::Receiver<SubscriptionUpdate>,
    pub user_update_receiver: broadcast::Receiver<SubscriptionUpdate>,
}

impl Context {
    pub fn new(retros: SharedRetros) -> Self {
        // Initialize broadcast channels with a buffer size of 100
        let (card_addition_sender, card_addition_receiver) = broadcast::channel(100);
        let (user_update_sender, user_update_receiver) = broadcast::channel(100);

        Context {
            retros,
            card_addition_sender,
            user_update_sender,
            card_addition_receiver,
            user_update_receiver
        }
    }

    pub fn from_self(&self) -> Self {
        Context {
            retros: self.retros.clone(),
            card_addition_receiver: self.card_addition_sender.subscribe(),
            card_addition_sender: self.card_addition_sender.clone(),
            user_update_sender: self.card_addition_sender.clone(),
            user_update_receiver: self.card_addition_sender.subscribe(),
        }
    }
}

impl JuniperContext for Context {}
