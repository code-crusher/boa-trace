//! Test types and methods to help with testing the Fetch API.

use crate::fetch::request::JsRequest;
use crate::fetch::response::JsResponse;
use boa_engine::{Context, Finalize, JsData, JsResult, JsString, Trace, js_error};
use http::{Request, Response, Uri};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[cfg(test)]
mod e2e;
#[cfg(test)]
mod headers;
#[cfg(test)]
mod request;
#[cfg(test)]
mod response;

/// A [`crate::fetch::Fetcher`] implementation for tests. Maps a URL to a response,
/// and record requests received for later use.
///
/// The actual safety of this implementation is not guaranteed, as it
/// is only intended for testing purposes.
#[derive(Default, Debug, Trace, Finalize, JsData)]
pub struct TestFetcher {
    #[unsafe_ignore_trace]
    requests_received: RefCell<Vec<Request<Option<Vec<u8>>>>>,
    #[unsafe_ignore_trace]
    request_mapper: HashMap<Uri, Response<Vec<u8>>>,
}

impl TestFetcher {
    /// Add a response mapping for a URL.
    pub fn add_response(&mut self, url: Uri, response: Response<Vec<u8>>) {
        self.request_mapper.insert(url, response);
    }
}

impl crate::fetch::Fetcher for TestFetcher {
    fn fetch(
        self: Rc<Self>,
        request: JsRequest,
        _signal: Option<boa_engine::JsObject>,
        _context: &RefCell<&mut Context>,
    ) -> impl Future<Output = JsResult<JsResponse>> {
        let request = request.into_inner();
        self.requests_received.borrow_mut().push(request.clone());
        let url = request.uri();
        std::future::ready((|| {
            // Simulate the redirect handling of a real fetcher: follow
            // redirections (resolving each `location` against the current URL)
            // until a final response is reached, and report the final URL with
            // `redirected` set accordingly.
            // See https://fetch.spec.whatwg.org/#dom-response-url
            // See https://fetch.spec.whatwg.org/#dom-response-redirected
            let mut current = url.clone();
            let mut redirected = false;

            for _ in 0..20 {
                let Some(response) = self.request_mapper.get(&current).cloned() else {
                    return Err(js_error!("No response found for URL"));
                };

                if !response.status().is_redirection() {
                    return Ok(
                        JsResponse::basic(JsString::from(current.to_string()), response)
                            .with_redirected(redirected),
                    );
                }

                let location = response
                    .headers()
                    .get(http::header::LOCATION)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| js_error!("Redirect response without a valid location"))?;

                // `Url::join` resolves the `location` against the current URL,
                // for both relative and absolute locations.
                let base = url::Url::parse(&current.to_string())
                    .map_err(|_| js_error!(TypeError: "Invalid URL: {current}"))?;
                let next = base
                    .join(location)
                    .map_err(|_| js_error!(TypeError: "Invalid redirect location: {location}"))?;
                current = Uri::try_from(next.as_str())
                    .map_err(|_| js_error!(TypeError: "Invalid URL: {next}"))?;

                redirected = true;
            }

            Err(js_error!("Too many redirects"))
        })())
    }
}
