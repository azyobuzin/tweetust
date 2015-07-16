use std::cmp;
use super::entities::Entities;
use super::users::User;

#[derive(Clone, Debug, RustcDecodable)]
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
    pub entities: Option<Entities>
}

impl cmp::Eq for DirectMessage { }
impl cmp::PartialEq for DirectMessage {
    fn eq(&self, other: &DirectMessage) -> bool { self.id == other.id }
}
