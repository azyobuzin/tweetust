#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct List {
    pub slug: String,
    pub name: String,
    pub created_at: String,
    pub uri: String,
    pub subscriber_count: u32,
    pub member_count: u32,
    pub id: i64,
    pub mode: ListMode,
    pub full_name: String,
    pub description: String,
    pub user: User,
    pub following: bool
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListMode {
    Public,
    Private
}

impl Serialize for ListMode {
    fn serialize<S: Serializer>(&self, serializer: &mut S) -> Result<(), S::Error> {
        serializer.serialize_str(match *self {
            ListMode::Public => "public",
            ListMode::Private => "private"
        })
    }
}

impl Deserialize for ListMode {
    fn deserialize<D: Deserializer>(deserializer: &mut D) -> Result<Self, D::Error> {
        struct Visitor;
        impl de::Visitor for Visitor {
            type Value = ListMode;
            fn visit_str<E: de::Error>(&mut self, v: &str) -> Result<Self::Value, E> {
                Ok(if v == "private" { ListMode::Private } else { ListMode::Public })
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CursorLists {
    pub previous_cursor: i64,
    pub next_cursor: i64,
    pub lists: Vec<List>
}
