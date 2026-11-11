use crate::value::Value;
use std::collections::BTreeMap;
use std::io::Read;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::time::Duration;

const MAX_BODY: usize = 8 * 1024 * 1024;
const MAX_TIMEOUT: f64 = 120.0;
const METHODS: &[&str] = &["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

type Error = (String, BTreeMap<String, Value>);

/// Resolves a hostname before connecting. Keeping this boundary separate makes
/// the SSRF policy deterministic and testable without a network.
pub(crate) trait Resolver {
    fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, String>;
}
pub(crate) struct SystemResolver;
impl Resolver for SystemResolver {
    fn resolve(&self, host: &str, port: u16) -> Result<Vec<SocketAddr>, String> {
        (host, port)
            .to_socket_addrs()
            .map(|x| x.collect())
            .map_err(|_| "destination DNS lookup failed".into())
    }
}

pub(crate) struct HttpRequest<'a> {
    pub method: &'a str,
    pub url: &'a str,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
    pub timeout: Duration,
}
pub(crate) struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}
/// Transport is deliberately independent from resolution so tests can provide
/// a fully deterministic fake (including redirects and repeated headers).
pub(crate) trait Transport {
    fn send(
        &self,
        request: HttpRequest<'_>,
        address: SocketAddr,
        max: usize,
    ) -> Result<HttpResponse, Error>;
}

fn production_client(
    host: &str,
    address: SocketAddr,
    timeout: Duration,
) -> Result<reqwest::blocking::Client, Error> {
    reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .timeout(timeout)
        .resolve_to_addrs(host, &[address])
        .build()
        .map_err(|_| simple("network", "HTTP client construction failed"))
}

pub(crate) struct ReqwestTransport;
impl Transport for ReqwestTransport {
    fn send(
        &self,
        request: HttpRequest<'_>,
        address: SocketAddr,
        max: usize,
    ) -> Result<HttpResponse, Error> {
        let host = url::Url::parse(request.url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_owned))
            .ok_or_else(|| simple("policy", "invalid URL"))?;
        let client = production_client(&host, address, request.timeout)?;
        let mut call = client.request(
            request
                .method
                .parse()
                .map_err(|_| simple("syntax", "unsupported HTTP method"))?,
            request.url,
        );
        for (name, value) in request.headers {
            call = call.header(name, value);
        }
        let response = call
            .body(request.body.unwrap_or_default())
            .send()
            .map_err(|error| {
                simple(
                    if error.is_timeout() {
                        "timeout"
                    } else {
                        "network"
                    },
                    "HTTP request failed",
                )
            })?;
        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_str().unwrap_or("").to_owned()))
            .collect();
        let mut body = Vec::new();
        response
            .take((max + 1) as u64)
            .read_to_end(&mut body)
            .map_err(|_| simple("network", "failed to read response"))?;
        if body.len() > max {
            return Err(simple("limit", "response body exceeds maxBytes"));
        }
        Ok(HttpResponse {
            status,
            headers,
            body,
        })
    }
}

pub(crate) fn request(
    method: &str,
    url: &str,
    headers: Option<Value>,
    body: Option<String>,
    timeout: Option<f64>,
    max: Option<usize>,
) -> Result<Value, Error> {
    request_with(
        method,
        url,
        headers,
        body,
        timeout,
        max,
        &SystemResolver,
        &ReqwestTransport,
    )
}
#[derive(Clone, Default)]
pub(crate) struct Options {
    pub headers: Option<Value>,
    pub body: Option<String>,
    pub timeout: Option<f64>,
    pub max_bytes: Option<usize>,
}

fn validate_url(raw_url: &str) -> Result<url::Url, Error> {
    let url = url::Url::parse(raw_url).map_err(|_| simple("policy", "invalid URL"))?;
    if url.scheme() != "https"
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.fragment().is_some()
    {
        return Err(simple(
            "policy",
            "destination must be an HTTPS URL without userinfo or fragment",
        ));
    }
    Ok(url)
}

fn request_response(
    method: &str,
    raw_url: &str,
    options: &Options,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> Result<HttpResponse, Error> {
    let url = validate_url(raw_url)?;
    let timeout = options.timeout.unwrap_or(30.0);
    if !(timeout.is_finite() && timeout > 0.0 && timeout <= MAX_TIMEOUT) {
        return Err(simple("limit", "timeout exceeds limit"));
    }
    let max = options.max_bytes.unwrap_or(MAX_BODY);
    if max == 0 || max > MAX_BODY {
        return Err(simple("limit", "maxBytes exceeds limit"));
    }
    let addresses = resolver
        .resolve(url.host_str().unwrap(), url.port().unwrap_or(443))
        .map_err(|m| simple("policy", &m))?;
    if addresses.iter().any(|address| blocked(address.ip())) {
        return Err(simple(
            "policy",
            "destination DNS answer includes a blocked address",
        ));
    }
    let address = addresses
        .into_iter()
        .next()
        .ok_or_else(|| simple("policy", "destination address is not public"))?;
    let mut hs = Vec::new();
    if let Some(Value::Dict(values)) = options.headers.clone() {
        for (name, value) in values {
            if let Value::Scalar(value) = value {
                hs.push((name, value));
            }
        }
    }
    transport.send(
        HttpRequest {
            method,
            url: raw_url,
            headers: hs,
            body: options.body.clone(),
            timeout: Duration::from_secs_f64(timeout),
        },
        address,
        max,
    )
}

fn response_value(raw_url: &str, response: HttpResponse) -> Value {
    let mut out = BTreeMap::new();
    out.insert("status".into(), Value::Scalar(response.status.to_string()));
    out.insert(
        "body".into(),
        Value::Scalar(String::from_utf8_lossy(&response.body).into()),
    );
    out.insert("url".into(), Value::Scalar(raw_url.into()));
    let mut hm = BTreeMap::new();
    for (name, value) in response.headers {
        hm.entry(name.clone())
            .or_insert_with(|| Value::List(Vec::new()));
        if let Some(Value::List(values)) = hm.get_mut(&name) {
            values.push(Value::Scalar(value));
        }
    }
    out.insert("headers".into(), Value::Dict(hm));
    Value::Dict(out)
}

fn request_with(
    method: &str,
    raw_url: &str,
    headers: Option<Value>,
    body: Option<String>,
    timeout: Option<f64>,
    max: Option<usize>,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> Result<Value, Error> {
    if !METHODS.contains(&method) {
        return Err(simple("syntax", "unsupported HTTP method"));
    }
    let options = Options {
        headers,
        body,
        timeout,
        max_bytes: max,
    };
    let response = request_response(method, raw_url, &options, resolver, transport)?;
    let value = response_value(raw_url, response);
    let status = match &value {
        Value::Dict(map) => match &map["status"] {
            Value::Scalar(s) => s.parse().unwrap_or(500),
            _ => 500,
        },
        _ => 500,
    };
    if status >= 400 {
        return Err(simple("http", "HTTP request returned a non-success status"));
    }
    Ok(value)
}

fn next_link(headers: &[(String, String)]) -> Result<Option<String>, Error> {
    for (name, value) in headers {
        if !name.eq_ignore_ascii_case("link") {
            continue;
        }
        for part in value.split(',') {
            let Some((target, params)) = part.trim().split_once(';') else {
                continue;
            };
            let is_next = params.split(';').any(|param| {
                let Some((key, value)) = param.trim().split_once('=') else {
                    return false;
                };
                key.trim().eq_ignore_ascii_case("rel")
                    && value
                        .trim()
                        .trim_matches('"')
                        .split_whitespace()
                        .any(|rel| rel.eq_ignore_ascii_case("next"))
            });
            if is_next {
                let target = target.trim();
                if !target.starts_with('<') || !target.ends_with('>') || target.len() < 2 {
                    return Err(simple("policy", "malformed Link next URL"));
                }
                return Ok(Some(target.to_owned()));
            }
        }
    }
    Ok(None)
}

pub(crate) fn request_pages(
    method: &str,
    raw_url: &str,
    options: &Options,
    max_pages: usize,
    max_total: usize,
    resolver: &dyn Resolver,
    transport: &dyn Transport,
) -> Result<Value, Error> {
    if method != "GET" {
        return Err(simple("syntax", "request_pages only supports GET"));
    }
    if max_pages == 0 {
        return Err(simple("limit", "maxPages exceeds limit"));
    }
    if max_total == 0 {
        return Err(simple("limit", "maxTotalBytes exceeds limit"));
    }
    let mut url = raw_url.to_owned();
    let mut seen = std::collections::HashSet::new();
    let mut pages = Vec::new();
    let mut total = 0usize;
    for page_index in 0..max_pages {
        if !seen.insert(url.clone()) {
            return Err(simple("policy", "pagination cycle detected"));
        }
        let response = request_response(method, &url, options, resolver, transport)?;
        total = total
            .checked_add(response.body.len())
            .ok_or_else(|| simple("limit", "maxTotalBytes exceeds limit"))?;
        if total > max_total {
            return Err(simple("limit", "maxTotalBytes exceeds limit"));
        }
        let next = next_link(&response.headers)?;
        let status = response.status;
        pages.push(response_value(&url, response));
        if status >= 400 {
            return Err(simple("http", "HTTP request returned a non-success status"));
        }
        let Some(next) = next else { break };
        if page_index + 1 == max_pages {
            return Err(simple("limit", "maxPages exceeds limit"));
        }
        let target = &next[1..next.len() - 1];
        let next_url = url::Url::parse(target)
            .or_else(|_| url::Url::parse(&url).and_then(|base| base.join(target)))
            .map_err(|_| simple("policy", "malformed Link next URL"))?;
        validate_url(next_url.as_str())?;
        url = next_url.into();
    }
    Ok(Value::List(pages))
}

fn simple(kind: &str, message: &str) -> Error {
    let mut fields = BTreeMap::new();
    fields.insert("kind".into(), Value::Scalar(kind.into()));
    fields.insert("message".into(), Value::Scalar(message.into()));
    (message.into(), fields)
}
#[doc(hidden)]
pub fn test_rejection(method: &str, url: &str) -> Result<(), String> {
    request(method, url, None, None, None, None)
        .map(|_| ())
        .map_err(|(m, _)| m)
}
#[doc(hidden)]
pub fn test_blocked(ip: IpAddr) -> bool {
    blocked(ip)
}
fn blocked(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            v.is_private()
                || v.is_loopback()
                || v.is_link_local()
                || v.is_multicast()
                || v.is_unspecified()
                || v.octets()[0] == 198
        }
        IpAddr::V6(v) => {
            v.is_loopback()
                || v.is_unspecified()
                || v.is_multicast()
                || v.segments()[0] & 0xfe00 == 0xfc00
                || v.segments()[0] & 0xffc0 == 0xfe80
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct FakeResolver {
        addresses: Result<Vec<SocketAddr>, String>,
    }
    impl Resolver for FakeResolver {
        fn resolve(&self, _: &str, _: u16) -> Result<Vec<SocketAddr>, String> {
            self.addresses.clone()
        }
    }

    struct FakeTransport {
        calls: RefCell<
            Vec<(
                String,
                String,
                Vec<(String, String)>,
                Option<String>,
                SocketAddr,
                usize,
            )>,
        >,
        result: RefCell<Option<Result<HttpResponse, Error>>>,
    }
    impl Transport for FakeTransport {
        fn send(
            &self,
            request: HttpRequest<'_>,
            address: SocketAddr,
            max: usize,
        ) -> Result<HttpResponse, Error> {
            self.calls.borrow_mut().push((
                request.method.into(),
                request.url.into(),
                request.headers,
                request.body,
                address,
                max,
            ));
            self.result.borrow_mut().take().unwrap()
        }
    }
    fn resolver(addresses: &[&str]) -> FakeResolver {
        FakeResolver {
            addresses: Ok(addresses.iter().map(|s| s.parse().unwrap()).collect()),
        }
    }
    fn transport(response: Result<HttpResponse, Error>) -> FakeTransport {
        FakeTransport {
            calls: RefCell::new(Vec::new()),
            result: RefCell::new(Some(response)),
        }
    }
    struct QueueTransport {
        responses: RefCell<Vec<HttpResponse>>,
        calls: RefCell<Vec<String>>,
    }
    impl Transport for QueueTransport {
        fn send(
            &self,
            request: HttpRequest<'_>,
            _: SocketAddr,
            _: usize,
        ) -> Result<HttpResponse, Error> {
            self.calls.borrow_mut().push(request.url.to_owned());
            self.responses
                .borrow_mut()
                .pop()
                .ok_or_else(|| simple("network", "missing fake response"))
        }
    }
    fn pages(
        responses: Vec<HttpResponse>,
        max_pages: usize,
        max_total: usize,
    ) -> Result<Value, Error> {
        request_pages(
            "GET",
            "https://example.test/1",
            &Options::default(),
            max_pages,
            max_total,
            &resolver(&["8.8.8.8:443"]),
            &QueueTransport {
                responses: RefCell::new(responses),
                calls: RefCell::new(Vec::new()),
            },
        )
    }
    fn headers(pairs: &[(&str, &str)]) -> Value {
        Value::Dict(
            pairs
                .iter()
                .map(|(k, v)| ((*k).into(), Value::Scalar((*v).into())))
                .collect(),
        )
    }

    #[test]
    fn production_client_receives_resolved_socket_address() {
        let address = "8.8.8.8:443".parse().unwrap();
        assert!(production_client("example.test", address, Duration::from_secs(1)).is_ok());
    }

    #[test]
    fn fake_success_preserves_repeated_response_headers() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![
                ("x-tag".into(), "one".into()),
                ("x-tag".into(), "two".into()),
            ],
            body: b"ok".to_vec(),
        }));
        let out = request_with(
            "GET",
            "https://example.test/",
            None,
            None,
            None,
            None,
            &resolver(&["8.8.8.8:443"]),
            &t,
        )
        .unwrap();
        let Value::Dict(map) = out else { panic!() };
        let Value::Dict(h) = &map["headers"] else {
            panic!()
        };
        assert_eq!(
            format!("{:?}", h["x-tag"]),
            "List([Scalar(\"one\"), Scalar(\"two\")])"
        );
    }

    #[test]
    fn request_constructs_method_headers_body_timeout_and_limit() {
        let t = transport(Ok(HttpResponse {
            status: 201,
            headers: vec![],
            body: vec![],
        }));
        request_with(
            "POST",
            "https://example.test/api",
            Some(headers(&[
                ("content-type", "text/plain"),
                ("x-test", "yes"),
            ])),
            Some("payload".into()),
            Some(2.5),
            Some(77),
            &resolver(&["8.8.8.8:443"]),
            &t,
        )
        .unwrap();
        let call = &t.calls.borrow()[0];
        assert_eq!(call.0, "POST");
        assert_eq!(call.2.len(), 2);
        assert_eq!(call.3.as_deref(), Some("payload"));
        assert_eq!(call.5, 77);
    }

    #[test]
    fn redirect_and_proxy_are_blocked_by_single_transport_dispatch() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
        }));
        request_with(
            "GET",
            "https://example.test/",
            None,
            None,
            None,
            None,
            &resolver(&["8.8.8.8:443"]),
            &t,
        )
        .unwrap();
        assert_eq!(
            t.calls.borrow().len(),
            1,
            "redirects and proxy fallback must not cause another dispatch"
        );
    }

    #[test]
    fn resolver_rejects_mixed_private_and_public_addresses_without_transport() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
        }));
        let error = request_with(
            "GET",
            "https://example.test/",
            None,
            None,
            None,
            None,
            &resolver(&["127.0.0.1:443", "8.8.8.8:443"]),
            &t,
        )
        .unwrap_err();
        assert!(matches!(&error.1["kind"], Value::Scalar(kind) if kind == "policy"));
        assert!(t.calls.borrow().is_empty());
    }

    #[test]
    fn all_private_resolved_addresses_are_rejected_without_transport() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
        }));
        let error = request_with(
            "GET",
            "https://example.test/",
            None,
            None,
            None,
            None,
            &resolver(&["10.0.0.1:443"]),
            &t,
        )
        .unwrap_err();
        assert!(matches!(&error.1["kind"], Value::Scalar(kind) if kind == "policy"));
        assert!(t.calls.borrow().is_empty());
    }

    #[test]
    fn timeout_and_response_size_errors_are_mapped_without_rewriting() {
        for error in [
            simple("timeout", "HTTP request failed"),
            simple("limit", "response body exceeds maxBytes"),
        ] {
            let kind = format!("{:?}", error.1["kind"]);
            let t = transport(Err(error));
            let result = request_with(
                "GET",
                "https://example.test/",
                None,
                None,
                None,
                None,
                &resolver(&["8.8.8.8:443"]),
                &t,
            )
            .unwrap_err();
            assert_eq!(format!("{:?}", result.1["kind"]), kind);
        }
    }

    #[test]
    fn non_success_response_redacts_response_fields() {
        let t = transport(Ok(HttpResponse {
            status: 403,
            headers: vec![("secret".into(), "value".into())],
            body: b"private".to_vec(),
        }));
        let error = request_with(
            "GET",
            "https://example.test/",
            None,
            None,
            None,
            None,
            &resolver(&["8.8.8.8:443"]),
            &t,
        )
        .unwrap_err();
        assert_eq!(error.1.len(), 2);
        assert!(!error.1.contains_key("body"));
        assert!(matches!(&error.1["kind"], Value::Scalar(kind) if kind == "http"));
    }

    #[test]
    fn pages_traverse_and_return_complete_response_dictionaries() {
        let out = pages(
            vec![
                HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: b"two".to_vec(),
                },
                HttpResponse {
                    status: 200,
                    headers: vec![
                        (
                            "Link".into(),
                            "<https://example.test/2>; rel=\"next\"".into(),
                        ),
                        ("X-Test".into(), "yes".into()),
                    ],
                    body: b"one".to_vec(),
                },
            ],
            3,
            100,
        )
        .unwrap();
        let Value::List(items) = out else { panic!() };
        assert_eq!(items.len(), 2);
        let Value::Dict(first) = &items[0] else {
            panic!()
        };
        assert!(matches!(&first["url"], Value::Scalar(value) if value == "https://example.test/1"));
        assert!(
            first.contains_key("status")
                && first.contains_key("body")
                && first.contains_key("headers")
        );
    }

    #[test]
    fn pages_parse_repeated_links_and_relation_lists() {
        let out = pages(
            vec![
                HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: b"two".to_vec(),
                },
                HttpResponse {
                    status: 200,
                    headers: vec![
                        ("Link".into(), "<https://example.test/2>; foo=bar".into()),
                        (
                            "link".into(),
                            "<https://example.test/3>; title=next; rel=\"prev next\"".into(),
                        ),
                    ],
                    body: b"one".to_vec(),
                },
            ],
            3,
            100,
        )
        .unwrap();
        let Value::List(items) = out else { panic!() };
        let Value::Dict(page) = &items[0] else {
            panic!()
        };
        assert!(matches!(&page["url"], Value::Scalar(value) if value == "https://example.test/1"));
        let Value::Dict(page) = &items[1] else {
            panic!()
        };
        assert!(matches!(&page["url"], Value::Scalar(value) if value == "https://example.test/3"));
    }

    #[test]
    fn pages_reject_non_get_zero_total_and_malformed_next_target() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
        }));
        assert!(
            request_pages(
                "POST",
                "https://example.test/1",
                &Options::default(),
                1,
                1,
                &resolver(&["8.8.8.8:443"]),
                &t
            )
            .is_err()
        );
        assert!(
            request_pages(
                "GET",
                "https://example.test/1",
                &Options::default(),
                1,
                0,
                &resolver(&["8.8.8.8:443"]),
                &t
            )
            .is_err()
        );
        let error = pages(
            vec![HttpResponse {
                status: 200,
                headers: vec![
                    ("Link".into(), "<https://example.test/2>; rel=prev".into()),
                    ("Link".into(), "not-a-target; rel=next".into()),
                ],
                body: b"x".to_vec(),
            }],
            2,
            100,
        )
        .unwrap_err();
        assert_eq!(error.0, "malformed Link next URL");
    }

    #[test]
    fn pages_reject_malformed_links_cycles_and_bounds() {
        let malformed = pages(
            vec![HttpResponse {
                status: 200,
                headers: vec![("Link".into(), "<https://[invalid>; rel=\"next\"".into())],
                body: b"x".to_vec(),
            }],
            2,
            0,
        )
        .unwrap_err();
        assert!(matches!(&malformed.1["kind"], Value::Scalar(value) if value == "limit"));
        let cycle = pages(
            vec![
                HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: vec![],
                },
                HttpResponse {
                    status: 200,
                    headers: vec![(
                        "Link".into(),
                        "<https://example.test/1>; rel=\"next\"".into(),
                    )],
                    body: vec![],
                },
            ],
            3,
            100,
        )
        .unwrap_err();
        assert!(cycle.0.contains("cycle"));
        let max_pages = pages(
            vec![
                HttpResponse {
                    status: 200,
                    headers: vec![],
                    body: vec![],
                },
                HttpResponse {
                    status: 200,
                    headers: vec![(
                        "Link".into(),
                        "<https://example.test/2>; rel=\"next\"".into(),
                    )],
                    body: vec![],
                },
            ],
            1,
            100,
        )
        .unwrap_err();
        assert_eq!(max_pages.0, "maxPages exceeds limit");
        let bytes = pages(
            vec![HttpResponse {
                status: 200,
                headers: vec![],
                body: b"1234".to_vec(),
            }],
            2,
            3,
        )
        .unwrap_err();
        assert!(matches!(&bytes.1["kind"], Value::Scalar(value) if value == "limit"));
    }

    #[test]
    fn pages_redact_non_success_page_errors() {
        let error = pages(
            vec![HttpResponse {
                status: 500,
                headers: vec![("secret".into(), "value".into())],
                body: b"private".to_vec(),
            }],
            2,
            100,
        )
        .unwrap_err();
        assert_eq!(error.1.len(), 2);
        assert!(!error.1.contains_key("body"));
        assert!(matches!(&error.1["kind"], Value::Scalar(value) if value == "http"));
    }

    #[test]
    fn rejected_resolution_proves_no_network_execution() {
        let t = transport(Ok(HttpResponse {
            status: 200,
            headers: vec![],
            body: vec![],
        }));
        let r = FakeResolver {
            addresses: Err("destination DNS lookup failed".into()),
        };
        assert!(
            request_with(
                "GET",
                "https://example.test/",
                None,
                None,
                None,
                None,
                &r,
                &t
            )
            .is_err()
        );
        assert!(t.calls.borrow().is_empty());
    }
}
