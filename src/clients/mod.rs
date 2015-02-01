use std::rc::Rc;
use conn::Authenticator;

pub mod direct_messages;
pub mod friendships;
pub mod search;
pub mod statuses;

#[derive(Clone, Debug)]
pub struct TwitterClient<T: Authenticator>(pub Rc<T>);

impl<T: Authenticator> TwitterClient<T> {
    pub fn new(authenticator: T) -> TwitterClient<T> {
        TwitterClient(Rc::new(authenticator))
    }

    pub fn direct_messages(&self) -> direct_messages::DirectMessagesClient<T> {
        direct_messages::DirectMessagesClient(self.0.clone())
    }

    pub fn friendships(&self) -> friendships::FriendshipsClient<T> {
        friendships::FriendshipsClient(self.0.clone())
    }

    pub fn search(&self) -> search::SearchClient<T> {
        search::SearchClient(self.0.clone())
    }

    pub fn statuses(&self) -> statuses::StatusesClient<T> {
        statuses::StatusesClient(self.0.clone())
    }
}
