use std::{io, env};
use crate::types::TokenInfo;
use crate::error::Error;

use http::Method;
use hyper::header::{HeaderValue, HeaderName};

pub struct MetaDataServerFlow {}

impl MetaDataServerFlow {
    pub(crate) fn new() -> Self {
        MetaDataServerFlow {}
    }

    /// Send a request for a new Bearer token to the OAuth provider.
    pub(crate) async fn token<C, T>(
        &self,
        hyper_client: &hyper::Client<C>,
        scopes: &[T],
    ) -> Result<TokenInfo, Error>
        where
            T: AsRef<str>,
            C: hyper::client::connect::Connect + Clone + Send + Sync + 'static,
    {

        const METADATA_HOST_ENV: &str = "GCE_METADATA_HOST";
        let host = match env::var(METADATA_HOST_ENV) {
            Ok(ref val) if val != "" => val.clone(),
            _ => "169.254.169.254".to_string(),
        };
        let url = [
            "http://",
            &host,
            "/computeMetadata/v1/",
            "instance/service-accounts/default/token",
        ]
            .concat();

        let https = HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        let req = Request::builder()
            .method(Method::GET)
            .uri(url)
            .header(
                "Metadata-Flavor".parse::<HeaderName>().unwrap(),
                HeaderValue::from_str("Google").unwrap(),
            )
            .header(
                "User-Agent".parse::<HeaderName>().unwrap(),
                HeaderValue::from_str("gcloud-rust/0.1").unwrap(),
            )
            .body("".into())
            .unwrap();

        let (head, body) = hyper_client.request(req).await?.into_parts();
        let body = hyper::body::to_bytes(body).await?;
        TokenInfo::from_json(&body)
    }
}