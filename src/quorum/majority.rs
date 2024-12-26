use std::collections::HashSet;
use std::fmt::Formatter;

/// A set of IDs that uses majority quorums to make decisions.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Configuration {
    voters: HashSet<u64>,
}

impl std::fmt::Display for Configuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.voters
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        )
    }
}

impl Configuration {
    /// Creates a new configuration using the given IDs.
    pub fn with_capacity(voters: usize) -> Configuration {
        Configuration { voters: HashSet::with_capacity(voters) }
    }
}
