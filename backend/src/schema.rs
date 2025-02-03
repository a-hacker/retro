use juniper::{RootNode, graphql_subscription};
use mongodb::bson::oid::ObjectId;
use crate::models::{Retro, RetroStep, RetroParticipant, Card, Lane, SubscriptionUpdate, User, UserListUpdated};
use crate::context::Context;
use std::pin::Pin;
use std::str::FromStr;
use chrono::prelude::*;
use tokio_stream::StreamExt;
use uuid::Uuid;
use std::collections::HashSet;

#[juniper::graphql_object(context = Context)]
impl User {
    fn id(&self) -> String {
        self._id.to_hex()
    }

    fn username(&self) -> &str {
        &self.username
    }

    fn active_user(&self, context: &Context) -> bool {
        self._id == context.active_user._id
    }
}

// GraphQL representation of a Card
#[juniper::graphql_object(context = Context)]
impl Card {
    fn id(&self) -> String {
        self.id.to_hex()
    }

    fn text(&self) -> &str {
        &self.text
    }

    async fn creator(&self, context: &Context) -> User {
        context.persistence_manager.get_user(&self.creator_id).await.unwrap()
    }

    fn subcards(&self) -> &Vec<Card> {
        &self.subcards
    }

    fn owned(&self, context: &Context) -> bool {
        self.creator_id == context.active_user._id
    }

    async fn voted(&self, context: &Context) -> bool {
        let uid = context.active_user._id;
        self.votes.contains(&uid)
    }

    fn votes(&self) -> i32 {
        self.votes.len() as i32
    }
}

// GraphQL representation of Cards
#[juniper::graphql_object(context = Context)]
impl Lane {
    fn id(&self) -> String {
        self.id.to_hex()
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn cards(&self) -> &Vec<Card> {
        &self.cards
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

// GraphQL representation of a RetroParticipant
#[juniper::graphql_object(context = Context)]
impl RetroParticipant {
    async fn user(&self, context: &Context) -> User {
        context.persistence_manager.get_user(&self.user).await.unwrap()
    }

    async fn retro(&self, context: &Context) -> Retro {
        context.persistence_manager.get_retro(&self.retro_id).await.unwrap()
    }
}

// GraphQL representation of a Retro
#[juniper::graphql_object(context = Context)]
impl Retro {
    fn id(&self) -> String {
        self._id.to_hex()
    }

    fn retro_name(&self) -> &str {
        &self.retro_name
    }

    fn step(&self) -> &RetroStep {
        &self.step
    }

    async fn creator(&self, context: &Context) -> User {
        context.persistence_manager.get_user(&self.creator_id).await.unwrap()
    }

    fn created_at(&self) -> &str {
        &self.created_at
    }

    fn participants(&self) -> &Vec<RetroParticipant> {
        &self.participants
    }

    fn lanes(&self) -> &Vec<Lane> {
        &self.lanes
    }
}

#[derive(juniper::GraphQLInputObject)]
pub struct CreateUserInput {
    pub username: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct CreateRetroInput {
    pub retro_name: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddCardInput {
    pub retro_id: String,
    pub lane_id: String,
    pub text: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct LeaveRetroInput {
    pub retro_id: Uuid,
    pub user_id: String,
}

// Subscription root
type SubStream = Pin<Box<dyn futures::Stream<Item = SubscriptionUpdate> + Send>>;

pub struct SubscriptionRoot;

#[graphql_subscription(context = Context)]
impl SubscriptionRoot {
    // Subscription for added cards
    async fn card_added(context: &Context, retro_id: String) -> SubStream {
        let rx = context.card_addition_sender.subscribe();
        let rid = ObjectId::from_str(&retro_id).unwrap();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::CardAdded (card) if card.retro_id == rid => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }

    // Subscription for user list updates
    async fn user_list_updated(context: &Context, retro_id: String) -> SubStream {
        let rx = context.user_update_sender.subscribe();
        let rid = ObjectId::from_str(&retro_id).unwrap();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::UserListUpdated(u) if u.retro_id == rid => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }

    async fn step_update(context: &Context, retro_id: String) -> SubStream {
        let rx = context.step_update_sender.subscribe();
        let rid = ObjectId::from_str(&retro_id).unwrap();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::StepUpdated(s) if s.retro_id == rid => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)

    }
}

// Root Query object
pub struct QueryRoot;

#[juniper::graphql_object(context = Context)]
impl QueryRoot {
    // Fetch all retrospectives
    async fn all_retros(context: &Context) -> Vec<Retro> {
        context.persistence_manager.get_retros().await.unwrap()
    }

    // Fetch a specific retro by ID
    async fn retro_by_id(context: &Context, id: String) -> Option<Retro> {
        let rid = ObjectId::from_str(&id).unwrap();
        context.persistence_manager.get_retro(&rid).await.ok()
    }

    async fn all_users(context: &Context) -> Vec<User> {
        context.persistence_manager.get_users().await.unwrap()
    }

    async fn user_by_id(context: &Context, id: String) -> Option<User> {
        let uid = ObjectId::from_str(&id).unwrap();
        context.persistence_manager.get_user(&uid).await.ok()
    }
}

// Root Mutation object
pub struct MutationRoot;

#[juniper::graphql_object(context = Context)]
impl MutationRoot {
    // Create a new retro
    async fn create_retro(context: &Context, input: CreateRetroInput) -> Retro {
        let new_id = ObjectId::new();
        let created_at = Utc::now().to_rfc3339();

        let default_lanes = vec![
            Lane {
                id: ObjectId::new(),
                title: "Good".to_string(),
                cards: Vec::new(),
                priority: 1
            },
            Lane {
                id: ObjectId::new(),
                title: "Bad".to_string(),
                cards: Vec::new(),
                priority: 2
            },
            Lane {
                id: ObjectId::new(),
                title: "Needs Improvement".to_string(),
                cards: Vec::new(),
                priority: 3
            }
        ];

        let new_retro = Retro {
            _id: new_id,
            retro_name: input.retro_name,
            step: RetroStep::Writing,
            creator_id: context.active_user._id,
            created_at,
            participants: vec![],
            lanes: default_lanes,
        };
        context.persistence_manager.create_retro(new_retro.clone()).await.unwrap();

        // Broadcast user list update for the new retro (initially empty)
        let _ = context.user_update_sender.send(SubscriptionUpdate::UserListUpdated ( UserListUpdated {
            retro_id: new_retro._id,
            participants: new_retro.participants.clone(),
        }));

        new_retro
    }

    // Add a user to a retro
    async fn enter_retro(context: &Context, retro_id: String) -> Vec<RetroParticipant> {
        let uid = context.active_user._id;
        let rid = ObjectId::from_str(&retro_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        if !retro.participants.iter().any(|p| p.user == uid) {
            let participant = RetroParticipant {
                user: uid,
                retro_id: rid,
            };
            retro.participants.push(participant.clone());
            context.persistence_manager.update_retro(retro.clone()).await.unwrap();

            // Broadcast user list update
            let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                rid,
                retro.participants.clone(),
            ));
        }
        retro.participants.clone()
    }

    // Remove a user from a retro
    async fn leave_retro(context: &Context, retro_id: String) -> Vec<RetroParticipant> {
        let uid = context.active_user._id;
        let rid = ObjectId::from_str(&retro_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        retro.participants.retain(|p| p.user != uid);
        context.persistence_manager.update_retro(retro.clone()).await.unwrap();

        // Broadcast user list update
        let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
            retro._id,
            retro.participants.clone(),
        ));

        retro.participants.clone()
    }

    // Add a card to a retro
    async fn add_card(context: &Context, input: AddCardInput) -> Option<Card> {
        let uid = context.active_user._id;
        let rid = ObjectId::from_str(&input.retro_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        let new_card = Card {
            id: ObjectId::new(),
            retro_id: rid,
            creator_id: uid,
            text: input.text.clone(),
            subcards: Vec::new(),
            votes: HashSet::new(),
        };

        let lane_id = ObjectId::from_str(&input.lane_id).unwrap();
        let lane = retro.lanes.iter_mut().find(|l| l.id == lane_id);

        if let Some(l) = lane {
            l.cards.push(new_card.clone());
            let lane_id = l.id;
            context.persistence_manager.update_retro(retro.clone()).await.unwrap();

            let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                retro._id,
                lane_id,
                new_card.clone(),
            ));

            Some(new_card)
        } else {
            None
        }
    }

    async fn edit_card(context: &Context,  retro_id: String, card_id: String, text: String) -> Option<Card> {
        let rid = ObjectId::from_str(&retro_id).unwrap();
        let cid = ObjectId::from_str(&card_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        let lane = retro.lanes.iter_mut().find(|l| l.cards.iter().any(|c| c.id == cid));

        if let Some(l) = lane {
            let card = l.cards.iter_mut().find(|c| c.id == cid).unwrap();
            card.text = text;
            let lane_id = l.id;
            let new_card = card.clone();
            context.persistence_manager.update_retro(retro.clone()).await.unwrap();

            let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                retro._id,
                lane_id,
                new_card.clone(),
            ));
            Some(new_card)
        } else {
            None
        }
    }

    // Vote for a card in the retro
    async fn vote_card(context: &Context, retro_id: String, card_id: String, vote: bool) -> Option<Card> {
        let uid = context.active_user._id;
        let rid = ObjectId::from_str(&retro_id).unwrap();
        let cid = ObjectId::from_str(&card_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        if let Some(lane) = retro.lanes.iter_mut().find(|l| l.cards.iter().any(|c| c.id == cid)) {
            let lane_id = lane.id;
            let card = lane.cards.iter_mut().find(|c| c.id == cid).unwrap();

            if card.creator_id != uid {
                return None;
            }

            if vote {
                card.votes.insert(uid);
            } else {
                card.votes.remove(&uid);
            }

            let new_card = card.clone();

            context.persistence_manager.update_retro(retro.clone()).await.unwrap();

            let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                retro._id,
                lane_id,
                new_card.clone(),
            ));
            Some(new_card)
        } else {
            None
        }
    }

    async fn update_retro_step(context: &Context, retro_id: String, step: RetroStep) -> Option<Retro> {
        let rid = ObjectId::from_str(&retro_id).unwrap();
        let mut retro = context.persistence_manager.get_retro(&rid).await.unwrap();
        retro.step = step.clone();
        context.persistence_manager.update_retro(retro.clone()).await.unwrap();

        let _ = context.step_update_sender.send(SubscriptionUpdate::create_step_update(
            retro._id,
            step,
        ));

        Some(retro.clone())
    }
}

// Define the schema
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, SubscriptionRoot)
}
