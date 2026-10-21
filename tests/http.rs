use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn check(source: &str) -> std::process::Output {
    let name = format!("shelly-http-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos());
    let path = std::env::temp_dir().join(name).with_extension("shelly");
    fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_shelly")).arg("check").arg(&path).output().unwrap();
    let _ = fs::remove_file(path);
    output
}

#[test]
fn v1_allows_query_strings_and_http_options() {
    let output = check("http GET https://example.test/resource?token=not-a-secret timeout 1 maxBytes 1024 -> $response\n");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn http_checker_accepts_dynamic_option_variables() {
    let output = check("$headers = {\"x-test\": \"yes\"}\n$body = \"payload\"\n$timeout = 1\n$limit = 1024\nhttp POST https://example.test/ headers $headers body $body timeout $timeout maxBytes $limit -> $response\n");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn http_checker_rejects_invalid_option_forms() {
    for source in [
        "http GET https://example.test/ headers -> $response\n",
        "http GET https://example.test/ body 123 -> $response\n",
        "http GET https://example.test/ timeout nope -> $response\n",
        "http GET https://example.test/ maxBytes 1.5 -> $response\n",
    ] {
        let output = check(source);
        assert!(!output.status.success(), "accepted invalid source: {source}");
    }
}

#[test]
fn http_rejects_repeated_options_deterministically() {
    let output = check("http GET https://example.test/ timeout 1 timeout 2 -> $response\n");
    assert!(!output.status.success());
}

#[test]
fn resolver_blocks_private_and_special_ipv4_addresses() {
    use std::net::{IpAddr, Ipv4Addr};
    for octets in [[10, 0, 0, 1], [127, 0, 0, 1], [169, 254, 1, 1], [198, 18, 0, 1], [224, 0, 0, 1]] {
        assert!(shelly::http::test_blocked(IpAddr::V4(Ipv4Addr::from(octets))), "{octets:?}");
    }
    assert!(!shelly::http::test_blocked(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
}

#[test]
fn resolver_blocks_private_and_special_ipv6_addresses() {
    use std::net::{IpAddr, Ipv6Addr};
    for address in [Ipv6Addr::LOCALHOST, Ipv6Addr::UNSPECIFIED, "fc00::1".parse().unwrap(), "fe80::1".parse().unwrap(), "ff02::1".parse().unwrap()] {
        assert!(shelly::http::test_blocked(IpAddr::V6(address)));
    }
    assert!(!shelly::http::test_blocked(IpAddr::V6("2001:4860:4860::8888".parse().unwrap())));
}

#[test]
fn runtime_rejects_invalid_url_and_method_without_network() {
    for (method, url) in [("GET", "http://example.test/"), ("GET", "https://user:pass@example.test/"), ("GET", "https://example.test/#fragment"), ("TRACE", "https://example.test/")] {
        let error = shelly::http::test_rejection(method, url).unwrap_err();
        assert!(error.contains("HTTPS") || error.contains("unsupported"), "{method} {url}: {:?}", error);
    }
}
