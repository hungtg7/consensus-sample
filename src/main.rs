use slog::{Drain, o};

mod node;
mod raft;
mod config;

use node::Node;
use crate::raft::Raft;

fn main() {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(drain, o!());

    let conf = config::Config{
        id: 1,
        heartbeat_tick: 15,
        election_tick: 5,
        min_election_tick: 0,
        max_election_tick: 10,
    };
    let raft = Raft::new(&conf, logger);
    let node = Node{raft: raft.unwrap()};
    println!("Hello, world!");
}
