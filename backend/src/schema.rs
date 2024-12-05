use juniper::{RootNode, graphql_subscription};
use crate::models::{Retro, RetroStep, RetroParticipant, Card, Lane, SubscriptionUpdate, User, UserListUpdated};
use crate::context::Context;
use std::pin::Pin;
use chrono::prelude::*;
use tokio_stream::StreamExt;
use uuid::Uuid;
use std::collections::{HashMap, HashSet};

// GraphQL representation of a Card
#[juniper::graphql_object(context = Context)]
impl Card {
    fn id(&self) -> Uuid {
        self.id
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn creator(&self, context: &Context) -> User {
        context.users.read().unwrap().get(&self.creator_id).unwrap().clone()
    }

    fn subcards(&self) -> &Vec<Card> {
        &self.subcards
    }
}

// GraphQL representation of Cards
#[juniper::graphql_object(context = Context)]
impl Lane {
    fn id(&self) -> Uuid {
        self.id
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
    fn user(&self, context: &Context) -> User {
        let users = context.users.read().unwrap();
        users.get(&self.user).unwrap().clone()
    }

    fn retro(&self, context: &Context) -> Retro {
        let retros = context.retros.read().unwrap();
        retros.iter().find(|r| r.id == self.retro_id).unwrap().clone()
    }

    fn votes(&self, context: &Context) -> Vec<Card> {
        let retros = context.retros.read().unwrap();
        if let Some(retro) = retros.iter().find(|r| r.id == self.retro_id) {
            let mut cards: HashMap<Uuid, Card> = HashMap::new();

            for lane in retro.lanes.iter() {
                let matched_cards: HashMap<Uuid, Card> = lane.cards.iter().filter_map(|c| self.votes.get(&c.id).map(|id| (*id, c.clone()))).collect();
                cards.extend(matched_cards);
            }
            self.votes.iter().filter_map(|card_id| cards.get(card_id)).cloned().collect()
        } else {
            Vec::new()
        }
    }
}

// GraphQL representation of a Retro
#[juniper::graphql_object(context = Context)]
impl Retro {
    fn id(&self) -> Uuid {
        self.id
    }

    fn retro_name(&self) -> &str {
        &self.retro_name
    }

    fn step(&self) -> &RetroStep {
        &self.step
    }

    fn creator(&self, context: &Context) -> User {
        context.users.read().unwrap().get(&self.creator_id).unwrap().clone()
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
    pub creator_id: Uuid,
}

#[derive(juniper::GraphQLInputObject)]
pub struct AddCardInput {
    pub retro_id: Uuid,
    pub lane_id: Uuid,
    pub creator_id: Uuid,
    pub text: String,
}

#[derive(juniper::GraphQLInputObject)]
pub struct LeaveRetroInput {
    pub retro_id: Uuid,
    pub user_id: Uuid,
}

// Subscription root
type SubStream = Pin<Box<dyn futures::Stream<Item = SubscriptionUpdate> + Send>>;

pub struct SubscriptionRoot;

#[graphql_subscription(context = Context)]
impl SubscriptionRoot {
    // Subscription for added cards
    async fn card_added(context: &Context, retro_id: Uuid) -> SubStream {
        let rx = context.card_addition_sender.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::CardAdded (card) if card.retro_id == retro_id => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }

    // Subscription for user list updates
    async fn user_list_updated(context: &Context, retro_id: Uuid) -> SubStream {
        let rx = context.user_update_sender.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::UserListUpdated(u) if u.retro_id == retro_id => Some(update),
                            _ => None,
                        }
                    }
                    Err(_) => None,
                }
            });

        Box::pin(stream)
    }

    async fn step_update(context: &Context, retro_id: Uuid) -> SubStream {
        let rx = context.step_update_sender.subscribe();

        let stream = tokio_stream::wrappers::BroadcastStream::new(rx)
            .filter_map(move |result| {
                match result {
                    Ok(update) => {
                        match update.clone() {
                            SubscriptionUpdate::StepUpdated(s) if s.retro_id == retro_id => Some(update),
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
    fn all_retros(context: &Context) -> Vec<Retro> {
        let retros = context.retros.read().unwrap();
        retros.clone()
    }

    // Fetch a specific retro by ID
    fn retro_by_id(context: &Context, id: Uuid) -> Option<Retro> {
        let retros = context.retros.read().unwrap();
        retros.iter().find(|retro| retro.id == id).cloned()
    }

    fn all_users(context: &Context) -> Vec<User> {
        let users = context.users.read().unwrap();
        users.values().cloned().collect()
    }

    fn user_by_id(context: &Context, id: Uuid) -> Option<User> {
        let users = context.users.read().unwrap();
        users.get(&id).cloned()
    }
}

// Root Mutation object
pub struct MutationRoot;

#[juniper::graphql_object(context = Context)]
impl MutationRoot {
    fn create_user(context: &Context, input: CreateUserInput) -> User {
        let mut users = context.users.write().unwrap();
        let user_id = Uuid::new_v4();
        let user = User {
            username: input.username,
            id: user_id
        };
        users.insert(user_id, user.clone());
        user
    }

    // Create a new retro
    fn create_retro(context: &Context, input: CreateRetroInput) -> Retro {
        let mut retros = context.retros.write().unwrap();
        let new_id = Uuid::new_v4();
        let created_at = Utc::now().to_rfc3339();

        let default_lanes = vec![
            Lane {
                id: Uuid::new_v4(),
                title: "Good".to_string(),
                cards: Vec::new(),
                priority: 1
            },
            Lane {
                id: Uuid::new_v4(),
                title: "Bad".to_string(),
                cards: Vec::new(),
                priority: 2
            },
            Lane {
                id: Uuid::new_v4(),
                title: "Needs Improvement".to_string(),
                cards: Vec::new(),
                priority: 3
            }
        ];

        let new_retro = Retro {
            id: new_id,
            retro_name: input.retro_name,
            step: RetroStep::Writing,
            creator_id: input.creator_id,
            created_at,
            participants: vec![],
            lanes: default_lanes,
        };

        retros.push(new_retro.clone());

        // Broadcast user list update for the new retro (initially empty)
        let _ = context.user_update_sender.send(SubscriptionUpdate::UserListUpdated ( UserListUpdated {
            retro_id: new_retro.id,
            participants: new_retro.participants.clone(),
        }));

        new_retro
    }

    // Add a user to a retro
    fn enter_retro(context: &Context, retro_id: Uuid, user_id: Uuid) -> Vec<RetroParticipant> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            if retro.participants.iter().find(|p| p.user == user_id).is_none() {
                let participant = RetroParticipant {
                    user: user_id,
                    retro_id: retro_id,
                    votes: HashSet::new()
                };
                retro.participants.push(participant);

                // Broadcast user list update
                let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                    retro_id,
                    retro.participants.clone(),
                ));
            }
            retro.participants.clone()
        } else {
            Vec::new()
        }
    }

    // Remove a user from a retro
    fn leave_retro(context: &Context, retro_id: Uuid, user_id: Uuid) -> Vec<RetroParticipant> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            retro.participants.retain(|p| p.user != user_id);

            // Broadcast user list update
            let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                retro.id,
                retro.participants.clone(),
            ));

            retro.participants.clone()
        } else {
            vec![]
        }
    }

    // Add a card to a retro
    fn add_card(context: &Context, input: AddCardInput) -> Option<Card> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == input.retro_id) {
            let new_card = Card {
                id: Uuid::new_v4(),
                creator_id: input.creator_id,
                text: input.text.clone(),
                subcards: Vec::new()
            };

            let lane = retro.lanes.iter_mut().filter(|l| l.id == input.lane_id).next();

            if let Some(l) = lane {
                l.cards.push(new_card.clone());

                let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                    retro.id,
                    l.id,
                    new_card.clone(),
                ));

                Some(new_card)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn edit_card(context: &Context,  retro_id: Uuid, card_id: Uuid, text: String) -> Option<Card> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            let lane = retro.lanes.iter_mut().find(|l| l.cards.iter().any(|c| c.id == card_id));

            if let Some(l) = lane {
                let card = l.cards.iter_mut().find(|c| c.id == card_id).unwrap();
                card.text = text;
                let _ = context.card_addition_sender.send(SubscriptionUpdate::create_card_added(
                    retro.id,
                    l.id,
                    card.clone(),
                ));
                Some(card.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    // Vote for a card in the retro
    fn vote_card(context: &Context, retro_id: Uuid, user_id: Uuid, card_id: Uuid, vote: bool) -> Option<Vec<RetroParticipant>> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            if let Some(participant) = retro.participants.iter_mut().find(|p| p.user == user_id) {
                if vote {
                    participant.votes.insert(card_id);
                } else {
                    participant.votes.remove(&card_id);
                }

                let _ = context.user_update_sender.send(SubscriptionUpdate::create_user_list_update(
                    retro.id,
                    retro.participants.clone()
                ));
                Some(retro.participants.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    fn update_retro_step(context: &Context, retro_id: Uuid, step: RetroStep) -> Option<Retro> {
        let mut retros = context.retros.write().unwrap();
        if let Some(retro) = retros.iter_mut().find(|retro| retro.id == retro_id) {
            retro.step = step.clone();

            let _ = context.step_update_sender.send(SubscriptionUpdate::create_step_update(
                retro.id,
                step,
            ));

            Some(retro.clone())
        } else {
            None
        }

    }
}

// Define the schema
pub type Schema = RootNode<'static, QueryRoot, MutationRoot, SubscriptionRoot>;

pub fn create_schema() -> Schema {
    Schema::new(QueryRoot, MutationRoot, SubscriptionRoot)
}
