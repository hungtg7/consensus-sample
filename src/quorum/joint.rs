use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
};

use crate::MajorityConfig;

/// Index is a Raft log position.
#[derive(Default, Clone, Copy)]
pub struct Index {
    pub index: u64,
    pub group_id: u64,
}

impl Display for Index {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.group_id {
            0 => match self.index {
                u64::MAX => write!(f, "∞"),
                index => write!(f, "{}", index),
            },
            group_id => match self.index {
                u64::MAX => write!(f, "[{}]∞", group_id),
                index => write!(f, "[{}]{}", group_id, index),
            },
        }
    }
}

impl Debug for Index {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

pub trait AckedIndexer {
    fn acked_index(&self, voter_id: u64) -> Option<Index>;
}

pub type AckIndexer = HashMap<u64, Index>;

impl AckedIndexer for AckIndexer {
    #[inline]
    fn acked_index(&self, voter: u64) -> Option<Index> {
        self.get(&voter).cloned()
    }
}
/// A configuration of two groups of (possibly overlapping) majority configurations.
/// Decisions require the support of both majorities.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    pub(crate) incoming: MajorityConfig,
    pub(crate) outgoing: MajorityConfig,
}

impl Configuration {
    /// Creates a new configuration using the given IDs.
    pub fn with_capacity(voters: usize) -> Configuration {
        Configuration {
            incoming: MajorityConfig::with_capacity(voters),
            outgoing: MajorityConfig::default(),
        }
    }
}
