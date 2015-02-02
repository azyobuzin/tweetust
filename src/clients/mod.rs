use std::rc::Rc;
use conn::Authenticator;

pub mod direct_messages;
pub mod followers;
pub mod friends;
pub mod friendships;
pub mod search;
pub mod statuses;

#[derive(Clone, Debug)]
pub struct TwitterClient<T: Authenticator>(pub Rc<T>);

impl<T: Authenticator> TwitterClient<T> {
    #[inline]
    pub fn new(authenticator: T) -> TwitterClient<T> {
        TwitterClient(Rc::new(authenticator))
    }

    pub fn direct_messages(self) -> direct_messages::DirectMessagesClient<T> {
        direct_messages::DirectMessagesClient(self.0)
    }

    pub fn followers(self) -> followers::FollowersClient<T> {
        followers::FollowersClient(self.0)
    }

    pub fn friends(self) -> friends::FriendsClient<T> {
        friends::FriendsClient(self.0)
    }

    pub fn friendships(self) -> friendships::FriendshipsClient<T> {
        friendships::FriendshipsClient(self.0)
    }

    pub fn search(self) -> search::SearchClient<T> {
        search::SearchClient(self.0)
    }

    pub fn statuses(self) -> statuses::StatusesClient<T> {
        statuses::StatusesClient(self.0)
    }
}
