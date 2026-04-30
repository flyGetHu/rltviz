use crate::config::{AppConfig, HttpMethod};
use curl_parser::ParsedRequest;
use std::str::FromStr;

/// Parse a raw cURL command string and populate the given config.
/// Returns an error message on failure.
pub fn import_curl(cmd: &str, config: &mut AppConfig) -> Result<(), String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return Err("cURL 命令为空".to_string());
    }
    let normalized = normalize_curl(trimmed);
    let parsed = ParsedRequest::from_str(&normalized).map_err(|e| format!("解析失败: {}", e))?;
    populate_config(config, &parsed)
}

/// Flags recognized by curl-parser that we pass through.
const KNOWN_FLAGS: &[&str] = &[
    "-X", "--request", "-H", "--header", "-d", "--data", "-L", "--location", "-u", "-k",
    "--insecure", "--url",
];

/// Flags we convert to short form (handled above in the match).
const CONVERT_FLAGS: &[&str] = &[
    "--request",
    "--url",
    "--header",
    "--data",
    "--data-raw",
    "--data-binary",
];

/// Harmless flags to strip (no-argument flags that curl-parser doesn't recognize).
const STRIP_FLAGS: &[&str] = &[
    "--compressed",
    "-s",
    "--silent",
    "-S",
    "--show-error",
    "-v",
    "--verbose",
    "-#",
    "--progress-bar",
    "-g",
    "--globoff",
    "-i",
    "--include",
    "-I",
    "--head",
];

/// Normalize a curl command for the parser:
/// - join line continuations
/// - convert long-form flags to short form
/// - quote bare URLs (no scheme, no quotes)
/// - strip unknown flags that would cause parse failures
fn normalize_curl(cmd: &str) -> String {
    let joined = cmd
        .lines()
        .map(|l| l.strip_suffix('\\').unwrap_or(l).trim_end())
        .collect::<Vec<_>>()
        .join(" ");

    let tokens: Vec<&str> = joined.split_whitespace().collect();
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        let t = tokens[i];

        // Strip harmless no-argument flags
        if STRIP_FLAGS.contains(&t) {
            i += 1;
            continue;
        }

        match t {
            "--request" | "-X" => {
                out.push("-X".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--url" => {
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--header" => {
                out.push("-H".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            "--data" | "--data-raw" | "--data-binary" => {
                out.push("-d".to_string());
                if let Some(&val) = tokens.get(i + 1) {
                    out.push(val.to_string());
                    i += 2;
                    continue;
                }
            }
            _ => {
                let quoted = t.starts_with('\'') || t.starts_with('"');
                let is_flag = t.starts_with('-');
                let has_scheme = t.contains("://");
                // Bare domain: not a flag, not quoted, no scheme, looks like a domain
                // Must not contain : (port/header) or stray quotes (split quoted value)
                let is_bare_domain = !is_flag
                    && !quoted
                    && !has_scheme
                    && t.contains('.')
                    && !t.contains(':')
                    && !t.contains('\'')
                    && !t.contains('"');
                if is_bare_domain {
                    out.push(format!("'{}'", t));
                } else if is_flag && !KNOWN_FLAGS.contains(&t) && !CONVERT_FLAGS.contains(&t)
                    && !STRIP_FLAGS.contains(&t)
                {
                    // Unknown flag — skip it silently
                } else {
                    out.push(t.to_string());
                }
            }
        }
        i += 1;
    }
    out.join(" ")
}

fn populate_config(config: &mut AppConfig, parsed: &ParsedRequest) -> Result<(), String> {
    let method = HttpMethod::from_str(parsed.method.as_str())
        .ok_or_else(|| format!("不支持的方法: {}。支持: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS", parsed.method))?;

    config.http.url = parsed.url.to_string();
    config.http.method = method;
    config.http.insecure = parsed.insecure;
    config.http.headers.clear();
    for (name, value) in parsed.headers.iter() {
        let key = name.as_str().to_string();
        let val = value.to_str().unwrap_or("").to_string();
        config.http.headers.push((key, val));
    }
    // Read raw body directly — avoids curl-parser's body() which panics on
    // unsupported Content-Type via unimplemented!()
    let body = parsed.body.last().cloned().unwrap_or_default();
    config.http.body = body;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    fn import_curl(cmd: &str) -> Result<AppConfig, String> {
        let mut config = AppConfig::default();
        super::import_curl(cmd, &mut config)?;
        Ok(config)
    }

    // --- Part 1: normalize_curl unit tests ---

    #[test]
    fn normalize_curl_joins_line_continuations() {
        let input = "curl \\\n  -X POST \\\n  https://example.com";
        assert_eq!(normalize_curl(input), "curl -X POST https://example.com");
    }

    #[test]
    fn normalize_curl_converts_long_request_flag() {
        assert_eq!(
            normalize_curl("curl --request POST https://example.com"),
            "curl -X POST https://example.com"
        );
    }

    #[test]
    fn normalize_curl_converts_long_url_flag() {
        assert_eq!(
            normalize_curl("curl --url https://example.com"),
            "curl https://example.com"
        );
    }

    #[test]
    fn normalize_curl_converts_long_header_flag() {
        assert_eq!(
            normalize_curl(r#"curl --header "Content-Type: application/json" https://example.com"#),
            r#"curl -H "Content-Type: application/json" https://example.com"#
        );
    }

    #[test]
    fn normalize_curl_converts_data_variants() {
        assert_eq!(
            normalize_curl(r#"curl --data-raw '{"key":"value"}' https://example.com"#),
            r#"curl -d '{"key":"value"}' https://example.com"#
        );
        assert_eq!(
            normalize_curl(r#"curl --data-binary '@file.json' https://example.com"#),
            r#"curl -d '@file.json' https://example.com"#
        );
        assert_eq!(
            normalize_curl(r#"curl --data 'key=val' https://example.com"#),
            r#"curl -d 'key=val' https://example.com"#
        );
    }

    #[test]
    fn normalize_curl_short_form_passthrough() {
        let input = "curl -X GET -H 'Accept: */*' -d 'data' https://example.com";
        assert_eq!(normalize_curl(input), input);
    }

    #[test]
    fn normalize_curl_empty_input() {
        assert_eq!(normalize_curl(""), "");
        assert_eq!(normalize_curl("   "), "");
    }

    // --- Part 2: end-to-end integration tests ---

    #[test]
    fn e2e_simple_get() {
        let config = import_curl("curl https://httpbin.org/get").unwrap();
        assert!(config.http.url.contains("httpbin.org/get"));
        assert_eq!(config.http.method, HttpMethod::GET);
        assert!(config.http.body.is_empty());
    }

    #[test]
    fn e2e_post_with_json() {
        let config = import_curl(
            r#"curl -X POST https://httpbin.org/post \
               -H "Content-Type: application/json" \
               -H "Authorization: Bearer token123" \
               -d '{"name":"test","value":42}'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
        assert!(config.http.url.contains("httpbin.org/post"));
        assert!(config.http.body.contains("name"));
        let has_ct = config
            .http
            .headers
            .iter()
            .any(|(k, v)| k == "content-type" && v.contains("application/json"));
        let has_auth = config
            .http
            .headers
            .iter()
            .any(|(k, v)| k == "authorization" && v.contains("Bearer"));
        assert!(has_ct, "should have Content-Type header");
        assert!(has_auth, "should have Authorization header");
    }

    #[test]
    fn e2e_multiline_long_form() {
        let config = import_curl(
            r#"curl 'https://api.example.com/users' \
               --request POST \
               --header 'Content-Type: application/json' \
               --header 'X-Custom: hello' \
               --data '{"user":"admin"}'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
        assert!(config.http.url.contains("api.example.com/users"));
        assert!(config.http.body.contains("admin"));
        let has_custom = config
            .http
            .headers
            .iter()
            .any(|(k, v)| k == "x-custom" && v.contains("hello"));
        assert!(has_custom, "should have X-Custom header");
    }

    #[test]
    fn e2e_all_long_form_flags() {
        let config = import_curl(
            r#"curl --request PUT --url https://api.example.com/item/1 \
               --header "Content-Type: application/json" \
               --data '{"id":1,"name":"updated"}'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::PUT);
        assert!(config.http.url.contains("api.example.com/item/1"));
        assert!(config.http.body.contains("updated"));
    }

    #[test]
    fn e2e_patch_method() {
        let config = import_curl("curl -X PATCH https://api.example.com/resource").unwrap();
        assert_eq!(config.http.method, HttpMethod::PATCH);
    }

    #[test]
    fn e2e_empty_body_post() {
        let config = import_curl("curl -X POST https://httpbin.org/post").unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
        assert!(config.http.body.is_empty());
    }

    #[test]
    fn e2e_url_without_scheme() {
        let config = import_curl("curl 'example.com/api'").unwrap();
        assert!(
            config.http.url.starts_with("http://"),
            "URL should be prefixed with http://, got: {}",
            config.http.url
        );
    }

    #[test]
    fn e2e_bare_url() {
        let config = import_curl("curl example.com/api").unwrap();
        assert!(
            config.http.url.contains("example.com"),
            "should parse bare URL, got: {:?}",
            config.http.url
        );
    }

    #[test]
    fn e2e_browser_data_raw() {
        let config = import_curl(
            r#"curl 'https://httpbin.org/post' \
               -H 'Content-Type: application/x-www-form-urlencoded' \
               --data-raw 'username=testuser&password=secret123'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
        assert!(
            config.http.body.contains("testuser"),
            "body should contain form data, got: {}",
            config.http.body
        );
    }

    // --- Part 3: edge cases ---

    #[test]
    fn e2e_data_binary_flag() {
        let config = import_curl(
            r#"curl --data-binary 'key=value&foo=bar' https://example.com/upload"#,
        )
        .unwrap();
        assert!(
            config.http.body.contains("key"),
            "body should contain data, got: {}",
            config.http.body
        );
    }

    #[test]
    fn e2e_header_with_colon_in_value() {
        let config = import_curl(
            r#"curl https://api.example.com/data \
               -H 'X-Forwarded-For: 192.168.1.1:8080' \
               -H 'If-Modified-Since: Wed, 21 Oct 2015 07:28:00 GMT'"#,
        )
        .unwrap();
        let has_forwarded = config
            .http
            .headers
            .iter()
            .any(|(k, v)| k == "x-forwarded-for" && v.contains("192.168.1.1:8080"));
        let has_modified = config
            .http
            .headers
            .iter()
            .any(|(k, v)| k == "if-modified-since" && v.contains("07:28:00"));
        assert!(has_forwarded, "should have X-Forwarded-For with port");
        assert!(has_modified, "should have If-Modified-Since with time");
    }

    #[test]
    fn normalize_curl_mixed_flags() {
        let result = normalize_curl(
            r#"curl --request POST \
               -H "Content-Type: application/json" \
               --header "Authorization: Bearer tok" \
               -d '{"test":true}' \
               https://api.example.com"#,
        );
        assert!(
            result.contains("-X POST"),
            "should contain -X POST, got: {}",
            result
        );
        assert!(
            !result.contains("--request"),
            "should not contain --request, got: {}",
            result
        );
        assert!(
            !result.contains("--header"),
            "should not contain --header, got: {}",
            result
        );
        assert!(result.contains("-H"), "should contain -H");
        assert!(result.contains("-d"), "should contain -d");
    }

    #[test]
    fn e2e_body_without_content_type() {
        // JSON body without Content-Type should not panic
        let config = import_curl(
            r#"curl -X POST https://example.com -d '{"key":"value"}'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
    }

    #[test]
    fn e2e_insecure_flag() {
        let config = import_curl("curl -k https://self-signed.example.com").unwrap();
        assert!(config.http.insecure, "insecure flag should be true");
    }

    #[test]
    fn e2e_insecure_long_form() {
        let config = import_curl("curl --insecure https://self-signed.example.com").unwrap();
        assert!(config.http.insecure, "insecure flag should be true");
    }

    #[test]
    fn e2e_compressed_flag_stripped() {
        let config = import_curl("curl --compressed https://httpbin.org/get").unwrap();
        assert!(config.http.url.contains("httpbin.org/get"));
    }

    #[test]
    fn e2e_browser_full_copy() {
        // Simulate Chrome "Copy as cURL" output with multiple ignored flags
        let config = import_curl(
            r#"curl 'https://api.example.com/data' \
               -H 'Accept: application/json' \
               -H 'Content-Type: application/json' \
               --compressed \
               -s \
               -d '{"query":"test"}'"#,
        )
        .unwrap();
        assert_eq!(config.http.method, HttpMethod::POST);
        assert!(config.http.body.contains("query"));
    }
}
