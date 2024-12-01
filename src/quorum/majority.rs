use std::collections::HashSet;

/// A set of IDs that uses majority quorums to make decisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    voters: HashSet<u64>,
}
