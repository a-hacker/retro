use crate::models::{SharedRetros, SharedUsers, SubscriptionUpdate };
use juniper::Context as JuniperContext;
use tokio::sync::broadcast;

// Define the Context struct that holds the shared state
pub struct Context {
    pub retros: SharedRetros,
    pub users: SharedUsers,
    pub card_addition_sender: broadcast::Sender<SubscriptionUpdate>,
    pub user_update_sender: broadcast::Sender<SubscriptionUpdate>,
    pub step_update_sender: broadcast::Sender<SubscriptionUpdate>,
}

impl Context {
    pub fn new(retros: SharedRetros, users: SharedUsers) -> Self {
        // Initialize broadcast channels with a buffer size of 100
        let (card_addition_sender, _) = broadcast::channel(100);
        let (user_update_sender, _) = broadcast::channel(100);
        let (step_update_sender, _) = broadcast::channel(100);

        Context {
            retros,
            users,
            card_addition_sender,
            user_update_sender,
            step_update_sender,
        }
    }

    pub fn from_self(&self) -> Self {
        Context {
            retros: self.retros.clone(),
            users: self.users.clone(),
            card_addition_sender: self.card_addition_sender.clone(),
            user_update_sender: self.user_update_sender.clone(),
            step_update_sender: self.step_update_sender.clone(),
        }
    }
}

impl JuniperContext for Context {}
