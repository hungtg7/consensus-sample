use crate::config::Config;
use crate::raft::Raft;
use crate::storage::Storage;
use anyhow::Result;
use slog::{info, Logger};

/// Node server
pub struct Node<T: Storage> {
    pub raft: Raft<T>,
}

impl<T: Storage> Node<T> {
    #[allow(clippy::new_ret_no_self)]
    /// Create a new RawNode given some [`Config`].
    pub fn new(config: &Config, storage: T, logger: &Logger) -> Result<Self> {
        let r = Raft::new(config, storage, logger)?;
        let rn = Node { raft: r };
        info!(
            rn.raft.logger,
            "RawNode created with id {id}.",
            id = rn.raft.id
        );
        Ok(rn)
    }

    pub fn has_ready(&self) -> bool {
        let raft = &self.raft;
        if !raft.msg.is_empty() {
            return true;
        }
        false
    }

    pub fn tick(&mut self) -> bool {
        self.raft.tick()
    }
}
