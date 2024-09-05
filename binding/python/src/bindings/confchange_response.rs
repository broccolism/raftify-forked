#[pyclass(name = "ConfChangeResponseResult")]
pub enum PyConfChangeResponseResult {
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