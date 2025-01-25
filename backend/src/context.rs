use crate::models::{SubscriptionUpdate, User};
use juniper::Context as JuniperContext;
use tokio::sync::broadcast;
use crate::database::PersistenceManager;


#[derive(Clone)]
pub struct ContextBuilder {
    pub persistence_manager: PersistenceManager,
    pub active_user: Option<User>,
    pub card_addition_sender: broadcast::Sender<SubscriptionUpdate>,
    pub user_update_sender: broadcast::Sender<SubscriptionUpdate>,
    pub step_update_sender: broadcast::Sender<SubscriptionUpdate>,
}

impl ContextBuilder {
    pub fn new(persistence_manager: PersistenceManager) -> Self {
        let (card_addition_sender, _) = broadcast::channel(100);
        let (user_update_sender, _) = broadcast::channel(100);
        let (step_update_sender, _) = broadcast::channel(100);

        ContextBuilder {
            persistence_manager,
            active_user: None,
            card_addition_sender,
            user_update_sender,
            step_update_sender,
        }
    }

    pub fn from_self(&self) -> Self {
        ContextBuilder {
            persistence_manager: self.persistence_manager.clone(),
            active_user: self.active_user.clone(),
            card_addition_sender: self.card_addition_sender.clone(),
            user_update_sender: self.user_update_sender.clone(),
            step_update_sender: self.step_update_sender.clone(),
        }
    }

    pub fn with_active_user(mut self, active_user: User) -> Self {
        self.active_user = Some(active_user);
        self
    }

    pub fn build(self) -> Context {
        Context {
            persistence_manager: self.persistence_manager,
            active_user: self.active_user.unwrap(),
            card_addition_sender: self.card_addition_sender,
            user_update_sender: self.user_update_sender,
            step_update_sender: self.step_update_sender,
        }
    }
}

// Define the Context struct that holds the shared state
pub struct Context {
    pub persistence_manager: PersistenceManager,
    pub active_user: User,
    pub card_addition_sender: broadcast::Sender<SubscriptionUpdate>,
    pub user_update_sender: broadcast::Sender<SubscriptionUpdate>,
    pub step_update_sender: broadcast::Sender<SubscriptionUpdate>,
}

impl JuniperContext for Context {}
