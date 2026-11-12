# Make bounded HTTPS requests

Use Shelly's HTTP v1 command for one bounded HTTPS request and one response dictionary. It is intentionally not a general HTTP client.

## Quick path

```shelly
http GET "https://example.com/health" -> $response
http POST "https://api.example.com/items" headers {"content-type": "text/plain"} body "hello" timeout 10 maxBytes 1048576 -> $response
```

The command must be one physical line and capture exactly one variable. Methods are `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, and `OPTIONS`.

## Options and response

```text
http METHOD URL [headers DICT] [body STRING] [timeout NUMBER] [maxBytes NUMBER] -> $VARIABLE
```

Each option may appear once. `headers` contains scalar string values; `body` is text and is not JSON-encoded automatically. The default timeout is 30 seconds, maximum 120 seconds. `maxBytes` defaults to 8 MiB and cannot exceed 8 MiB.

A successful response contains:

| Key | Value |
| --- | --- |
| `status` | HTTP status as text |
| `body` | Response bytes decoded as UTF-8 lossily |
| `headers` | Header names mapped to lists of values |
| `url` | Requested URL text |

Repeated response headers are preserved. Status 400 or higher raises a catchable `http` error instead of returning a response.

## Pagination

```shelly
httpPages GET "https://api.github.com/repos/shelly-lang/shelly/issues?per_page=100" headers {"accept": "application/vnd.github+json"} -> $pages
```

`httpPages` accepts GET only and returns complete response dictionaries in request order. It follows only a `Link` header with `rel="next"`, using absolute HTTPS or relative URLs. It follows no redirects and does not infer pagination from JSON.

| Option | Default | Limit |
| --- | --- | --- |
| `maxPages` | 10 | Positive maximum responses |
| `maxTotalBytes` | 32 MiB | Positive combined body limit |
| `maxBytes` | 8 MiB | Positive per-response limit |

Malformed links, cycles, page limits, byte limits, or any failed page fail the command; partial pages are not returned.

## Network and security boundaries

- Only `https://` URLs are accepted; userinfo, passwords, and fragments are rejected.
- DNS is resolved before connecting. Loopback, private, link-local, multicast, unspecified, IPv4 `198.0.0.0/8`, and IPv6 unique-local/link-local addresses are rejected. A mixed DNS answer containing a blocked address rejects the request.
- The client uses no proxy, follows no redirects, and performs one transport dispatch. TLS uses the HTTP library's normal validation; Shelly exposes no custom CA or verification bypass.

This address check is made from the resolver answer at request time; it cannot prevent later DNS changes or an address becoming private after resolution.

## Errors and checking

Transport failures use kinds such as `network`, `timeout`, `policy`, `syntax`, or `limit`. HTTP error catch fields do not include response status, body, headers, or URL. Use [try/catch](errors.md); see [Timeouts](timeouts.md) for deadlines.

`shelly check` validates grammar and expressions without DNS, network, TLS, proxy, or redirect work. Runtime policy and limits still apply when executed.

HTTP v1 does not provide plain HTTP, redirects, proxies, custom TLS, streaming, binary preservation, multipart uploads, repeated request headers, automatic JSON, authentication helpers, cookies, or retries.
