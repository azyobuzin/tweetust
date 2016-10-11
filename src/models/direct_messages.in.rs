#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectMessage {
    pub id: i64,
    pub text: String,
    pub sender: User,
    pub sender_id: i64,
    pub sender_screen_name: String,
    pub recipient: User,
    pub recipient_id: i64,
    pub recipient_screen_name: String,
    pub created_at: String,
    pub entities: Option<Box<Entities>>
}
