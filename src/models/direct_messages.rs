use super::entities::Entities;
use super::users::User;

#[derive(Clone, Debug, RustcDecodable)]
#[id_eq]
pub struct DirectMessage {
    pub id: u64,
    pub text: String,
    pub sender: User,
    pub sender_id: u64,
    pub sender_screen_name: String,
    pub recipient: User,
    pub recipient_id: u64,
    pub recipient_screen_name: String,
    pub created_at: String,
    pub entities: Option<Entities>
}
