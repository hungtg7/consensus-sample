/// A constant represents invalid id of raft.
pub const INVALID_ID: u64 = 0;

pub struct Config {
    /// id of this node
    pub id: u64,

    /// hearbeat tick
    pub hearbeat_tick: usize,

    /// election_tick
    pub election_tick: usize,

    /// min election_tick
    pub min_election_tick: usize,

    /// max election_tick
    pub max_election_tick: usize,
}

impl Config {
    /// Runs validations against the config.
    pub fn validate(&self) -> Result<()> {
        if self.id == INVALID_ID {
            return Err(Error::ConfigInvalid("invalid node id".to_owned()));
        }

        if self.heartbeat_tick == 0 {
            return Err(Error::ConfigInvalid(
                "heartbeat tick must greater than 0".to_owned(),
            ));
        }

        if self.election_tick <= self.heartbeat_tick {
            return Err(Error::ConfigInvalid(
                "election tick must be greater than heartbeat tick".to_owned(),
            ));
        }

        let min_timeout = self.min_election_tick();
        let max_timeout = self.max_election_tick();
        if min_timeout < self.election_tick {
            return Err(Error::ConfigInvalid(format!(
                "min election tick {} must not be less than election_tick {}",
                min_timeout, self.election_tick
            )));
        }

        if min_timeout >= max_timeout {
            return Err(Error::ConfigInvalid(format!(
                "min election tick {} should be less than max election tick {}",
                min_timeout, max_timeout
            )));
        }

        Ok(())
    }
}
