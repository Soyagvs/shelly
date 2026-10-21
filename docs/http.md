# HTTP requests

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>

[Documentation index](index.md) · [Error handling](errors.md) · [Timeouts](timeouts.md)

Shelly's HTTP v1 command makes one HTTPS request and captures one response dictionary. It is deliberately small: use it for bounded, public HTTPS calls—not as a general HTTP client.

## Quick path

The complete command must be on one physical line:

```shelly
http GET "https://example.com/health" -> $response
```

`http` always requires exactly one capture destination after `->`. The supported methods are `GET`, `POST`, `PUT`, `PATCH`, `DELETE`, `HEAD`, and `OPTIONS`.

Optional clauses may appear after the URL, once each:

```shelly
http POST "https://api.example.com/items" headers {"content-type": "text/plain"} body "hello" timeout 10 maxBytes 1048576 -> $response
```

## Exact grammar

```text
http METHOD URL [headers DICT] [body STRING] [timeout NUMBER] [maxBytes NUMBER] -> $VARIABLE
```

The grammar is one-line only. `URL` is an expression token accepted by Shelly's command parser; `body` is a string expression, `headers` is a dictionary expression, and `timeout`/`maxBytes` are numeric expressions. Each option is optional, but each may occur at most once. The capture is exactly one variable.

Valid:

```shelly
http GET https://example.com/ -> $r
http GET https://example.com/search?q=shelly timeout 2 maxBytes 4096 -> $r
http POST https://example.com/ headers {"x-mode": "test"} body "payload" -> $r
```

Invalid:

```shelly
http TRACE https://example.com/ -> $r       # unsupported method
http GET http://example.com/ -> $r          # HTTPS is required
http GET https://example.com/ -> $a $b       # exactly one capture
http GET https://example.com/ timeout 1 timeout 2 -> $r # repeated option
```

A multiline form is not an alternative spelling; it is not HTTP v1 syntax.

## Response dictionary

A successful request stores a dictionary with:

| Key | Value |
| --- | --- |
| `status` | HTTP status code as text, such as `"200"` |
| `body` | Response bytes decoded as UTF-8 lossily |
| `headers` | Dictionary of response header names to lists of text values |
| `url` | The requested URL text |

Repeated response headers are preserved rather than overwritten:

```text
headers["set-cookie"] == ["a=1", "b=2"]
```

Header names and values are returned as provided by the HTTP library. Request headers are supplied as a dictionary of scalar string values; repeated request-header entries cannot be represented by this v1 grammar.

## Request preparation

`headers` is passed to the request as supplied. `body` is text and is sent as the request body; Shelly does not infer a content type or serialize a dictionary/JSON value for you. Set `content-type` explicitly when the server needs one.

The default timeout is 30 seconds. `timeout` must be finite, positive, and no greater than 120 seconds. `maxBytes` defaults to 8 MiB, must be positive, and cannot exceed 8 MiB. The response body is rejected when it exceeds the selected limit.

## HTTPS and network limits

- Only `https://` URLs are accepted. Userinfo, passwords, and URL fragments are rejected.
- DNS is resolved before connecting. The selected address must be public; loopback, private, link-local, multicast, unspecified, IPv4 198.0.0.0/8, and IPv6 unique-local/link-local ranges are rejected. This check happens before the request is sent.
- The client uses no proxy, follows no redirects, and makes one transport dispatch. Redirect responses are returned as ordinary responses; they are not followed.
- TLS uses the HTTP client's normal HTTPS/TLS validation. Shelly v1 exposes no custom CA, certificate-verification bypass, proxy, redirect, or TLS configuration.

These restrictions are intentional SSRF and resource limits, not suggestions to work around in scripts.

## Errors and redaction

A response with status 400 or higher raises a catchable error with `kind == "http"` and `message == "HTTP request returned a non-success status"`. Its response status, body, headers, and URL are not copied into catch fields. This prevents a failed response from leaking response data through structured error handling.

Transport failures use `kind` values such as `network`, `timeout`, `policy`, `syntax`, or `limit`, with a `message`. DNS failure and blocked destinations are policy errors. A timeout or body-limit failure is not converted into a successful response.

```shelly
try -> $error
    http GET https://example.com/health timeout 5 -> $response
catch
    if $error.kind == "http"
        shellout "the server returned an error"
    end
end
```

See [Error handling](errors.md) for catch rules and [Timeouts](timeouts.md) for the relationship with command and block deadlines.

## Check mode

`shelly check` validates the HTTP grammar, methods, options, expressions, and capture shape without executing the request. It does not perform DNS, network, TLS, proxy, or redirect work. Runtime policy and limit checks still apply when the script runs.

## Non-goals in v1

HTTP v1 does not provide HTTP (non-TLS) URLs, redirects, proxies, custom TLS settings, streaming uploads/downloads, binary response preservation, multipart/form uploads, repeated request headers, automatic JSON encoding/decoding, authentication helpers, cookies, retries, or pagination.

<p align="center">
  <img src="https://capsule-render.vercel.app/api?type=rect&height=2&color=7c3aed" width="100%" alt="purple divider" />
</p>
