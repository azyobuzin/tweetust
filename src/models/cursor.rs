#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorIds {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub ids: Vec<i64>
}
