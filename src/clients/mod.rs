use std::rc::Rc;
use super::conn::Authenticator;

pub struct TwitterClient<T: Authenticator>(pub Rc<T>);

impl<T: Authenticator> TwitterClient<T> {
    pub fn new(authenticator: &T) -> TwitterClient<T> {
        TwitterClient(Rc::new(authenticator.clone()))
    }

    pub fn statuses(&self) -> statuses::StatusesClient<T> {
        statuses::StatusesClient(self.0.clone())
    }
}

pub mod statuses;
