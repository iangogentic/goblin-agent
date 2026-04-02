mod agent;
mod agent_definition;
mod context_engine;
mod conversation;
mod database;
mod fs_snap;
mod fuzzy_search;
mod provider;
mod repo;
mod skill;
mod validation;

mod proto_generated {
    tonic::include_proto!("goblin.v1");
}

pub use repo::GoblinRepo;
