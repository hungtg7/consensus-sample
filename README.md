# consensus-sample


## Joint Consensus
The joint consensus allows individual servers to transition between configurations at different times without compromising safety. Furthermore, joint consensus allows cluster to continue servicing client requests throughout
the configuration change.

ONGOING: step (up or down) while a node receving a message
TODO: msg.term < self.term

reference:

* [The Secret Lives of Data - Raft](http://thesecretlivesofdata.com/raft/)
* [Raft Paper](https://raft.github.io/raft.pdf)
