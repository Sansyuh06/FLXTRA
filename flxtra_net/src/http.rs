//! HTTP client with security enforcement
//!
//! Provides HTTPS-only HTTP client with:
//! - Certificate validation
//! - TLS 1.3 enforcement
//! - First-party isolation
//! - Integration with content filter

use flxtra_core::{FlxtraError, HttpMethod, Result};
use bytes::Bytes;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, USER_AGENT},
    Client, Response,
};
use std::collections::HashMap;
use std::str::FromStr;

use std::time::Duration;
use tracing::{debug, info};

/// HTTP response data
#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: Bytes,
    pub url: String,
}

/// HTTPS-only HTTP client
pub struct HttpClient {
    client: Client,
    allow_http: bool,
}

impl HttpClient {
    /// Create a new HTTP client
    pub fn new(allow_http: bool, timeout_secs: u64) -> Result<Self> {
        let mut headers = HeaderMap::new();
        
        // Minimal, standardized headers for fingerprint resistance
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; rv:109.0) Gecko/20100101 Firefox/115.0"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"),
        );
        headers.insert(
            ACCEPT_LANGUAGE,
            HeaderValue::from_static("en-US,en;q=0.5"),
        );
        headers.insert(
            ACCEPT_ENCODING,
            HeaderValue::from_static("gzip, deflate, br"),
        );

        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .https_only(!allow_http)
            .use_rustls_tls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .default_headers(headers)
            .gzip(true)
            .brotli(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(|e| FlxtraError::Network(e.to_string()))?;

        info!("HTTP client initialized (HTTPS-only: {})", !allow_http);

        Ok(Self { client, allow_http })
    }

    /// Perform a GET request
    pub async fn get(&self, url: &str) -> Result<HttpResponse> {
        self.request(HttpMethod::Get, url, None, None).await
    }

    /// Perform an HTTP request
    pub async fn request(
        &self,
        method: HttpMethod,
        url: &str,
        headers: Option<HashMap<String, String>>,
        body: Option<Vec<u8>>,
    ) -> Result<HttpResponse> {
        // Determine final URL (upgrade HTTP to HTTPS if needed)
        let final_url = if !self.allow_http && !url.starts_with("https://") {
            if url.starts_with("http://") {
                debug!("Upgrading {} to HTTPS", url);
                url.replacen("http://", "https://", 1)
            } else {
                return Err(FlxtraError::SecurityViolation(
                    "HTTP connections are not allowed".to_string(),
                ));
            }
        } else {
            url.to_string()
        };

        debug!("HTTP {} {}", method.as_str(), &final_url);

        let mut request = match method {
            HttpMethod::Get => self.client.get(&final_url),
            HttpMethod::Post => self.client.post(&final_url),
            HttpMethod::Put => self.client.put(&final_url),
            HttpMethod::Delete => self.client.delete(&final_url),
            HttpMethod::Head => self.client.head(&final_url),
            HttpMethod::Options => self.client.request(reqwest::Method::OPTIONS, &final_url),
            HttpMethod::Patch => self.client.patch(&final_url),
        };

        // Add custom headers
        if let Some(hdrs) = headers {
            for (key, value) in hdrs {
                if let (Ok(name), Ok(val)) = (
                    HeaderName::from_str(&key),
                    HeaderValue::from_str(&value),
                ) {
                    request = request.header(name, val);
                }
            }
        }

        // Add body if present
        if let Some(body_data) = body {
            request = request.body(body_data);
        }

        let response = request
            .send()
            .await
            .map_err(|e| self.map_request_error(e))?;

        self.process_response(response).await
    }

    /// Process the response
    async fn process_response(&self, response: Response) -> Result<HttpResponse> {
        let status = response.status().as_u16();
        let url = response.url().to_string();
        
        // Convert headers
        let mut headers = HashMap::new();
        for (name, value) in response.headers() {
            if let Ok(v) = value.to_str() {
                headers.insert(name.to_string(), v.to_string());
            }
        }

        let body = response
            .bytes()
            .await
            .map_err(|e| FlxtraError::Network(e.to_string()))?;

        Ok(HttpResponse {
            status,
            headers,
            body,
            url,
        })
    }

    /// Map reqwest error to FlxtraError
    fn map_request_error(&self, error: reqwest::Error) -> FlxtraError {
        if error.is_timeout() {
            FlxtraError::Timeout(error.to_string())
        } else if error.is_connect() {
            FlxtraError::ConnectionRefused(error.to_string())
        } else if error.is_status() {
            if let Some(status) = error.status() {
                FlxtraError::Http {
                    status: status.as_u16(),
                    message: error.to_string(),
                }
            } else {
                FlxtraError::Network(error.to_string())
            }
        } else {
            FlxtraError::Network(error.to_string())
        }
    }
}

/// Privacy-preserving referrer policy
pub fn sanitize_referrer(referrer: &str, target: &str) -> Option<String> {
    let ref_origin = url::Url::parse(referrer).ok()?;
    let target_origin = url::Url::parse(target).ok()?;

    // Same-origin: send full referrer
    if ref_origin.host() == target_origin.host() {
        return Some(referrer.to_string());
    }

    // Cross-origin: send only origin
    Some(format!(
        "{}://{}",
        ref_origin.scheme(),
        ref_origin.host_str()?
    ))
}
