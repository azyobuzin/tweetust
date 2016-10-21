#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub slug: String,
    pub name: String,
    pub created_at: CreatedAt,
    pub uri: String,
    pub subscriber_count: u32,
    pub member_count: u32,
    pub id: i64,
    pub mode: ListMode,
    pub full_name: String,
    pub description: String,
    pub user: User,
    pub following: bool,
}

enum_str!(ListMode {
    Public("public"),
    Private("private"),
});

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorLists {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub lists: Vec<List>,
}
