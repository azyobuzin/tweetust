use super::conn::Authenticator;

pub struct TwitterClient<T: Authenticator>(pub T);
