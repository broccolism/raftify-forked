use crate::{Error, Peers};

#[derive(Debug)]
pub enum ConfChangeResponseResult {
    JoinSuccess {
        assigned_ids: Vec<u64>,
        peers: Peers,
    },
    RemoveSuccess,
    Error(Error),
    WrongLeader {
        leader_id: u64,
        leader_addr: String,
    },
}
