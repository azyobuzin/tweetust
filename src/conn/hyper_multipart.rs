use std::io;
use hyper;
use hyper::mime;
use multipart::client as multipart;

type FreshRequest = hyper::client::Request<hyper::net::Fresh>;
type StreamingRequest = hyper::client::Request<hyper::net::Streaming>;

pub struct HyperRequest(pub FreshRequest);

impl multipart::HttpRequest for HyperRequest {
    type Stream = HyperRequestStream;
    type Error = hyper::Error;

    fn apply_headers(&mut self, boundary: &str, content_len: Option<u64>) -> bool {
        let mut headers = self.0.headers_mut();

        headers.set(hyper::header::ContentType(
            mime::Mime(
                mime::TopLevel::Multipart,
                mime::SubLevel::FormData,
                vec![(mime::Attr::Boundary, mime::Value::Ext(boundary.into()))]
            )
        ));

        if let Some(content_len) = content_len {
            headers.set(hyper::header::ContentLength(content_len))
        }

        true
    }

    fn open_stream(self) -> Result<Self::Stream, Self::Error> {
        self.0.start().map(HyperRequestStream)
    }
}

pub struct HyperRequestStream(pub StreamingRequest);

impl multipart::HttpStream for HyperRequestStream {
    type Request = HyperRequest;
    type Response = hyper::client::Response;
    type Error = hyper::Error;

    fn finish(self) -> Result<Self::Response, Self::Error> {
        self.0.send()
    }
}

impl io::Write for HyperRequestStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.0.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

pub fn create_multipart_client(req: FreshRequest) -> hyper::Result<multipart::Multipart<HyperRequestStream>> {
    multipart::Multipart::from_request(HyperRequest(req))
}
