mod agent;
mod agent_definition;
mod context_engine;
mod conversation;
mod database;
mod goblin_repo;
mod fs_snap;
mod fuzzy_search;
mod provider;
mod skill;
mod validation;

mod proto_generated {
    tonic::include_proto!("goblin.v1");
}

// Only expose goblin_repo container
pub use goblin_repo::*;
