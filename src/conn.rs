use hyper::method::Method;

pub enum ParameterValue {
    StringValue(String),
    File()
}

pub trait Authenticator {
    fn send_request(&self, method: Method, url: &str, params: Vec<(String, ParameterValue)>)
        -> hyper::HttpResult<hyper::client::response::Response>;
}
