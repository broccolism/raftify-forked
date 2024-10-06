use crate::raftify::ConfChangeResponseResult;

#[derive(Clone)]
#[pyclass(name = "ConfChangeResponseResult")]
pub struct PyConfChangeResponseResult(pub ConfChangeResponseResult);

impl From<PyConfChangeResponseResult> for ConfChangeResponseResult {
    fn from(val: PyConfChangeResponseResult) -> Self {
        val.0
    }
}