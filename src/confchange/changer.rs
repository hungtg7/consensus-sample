use crate::tracker::{ProgressTracker, Configuration};
use crate::errors::Result;
use raftpb::proto::ConfChangeSingle;

/// Changer facilitates configuration changes.
pub struct Changer<'a> {
    pub tracker: &'a mut ProgressTracker,
}

impl<'a> Changer<'a> {
    pub fn new(tracker: &'a mut ProgressTracker) -> Self {
        Changer { tracker }
    }

    pub fn simple(&mut self, _changes: &[ConfChangeSingle]) -> Result<(Configuration, Vec<(u64, u64)>)> {
        // For now, return the current config to satisfy the compiler
        Ok((self.tracker.conf().clone(), Vec::new()))
    }

    pub fn enter_joint(&mut self, _auto_leave: bool, _changes: &[ConfChangeSingle]) -> Result<(Configuration, Vec<(u64, u64)>)> {
        // Joint consensus logic goes here
        Ok((self.tracker.conf().clone(), Vec::new()))
    }
}
