use chrono::{DateTime,Local};

pub fn fetch_headers(contents_len: usize) -> [String; 10] {
    let now: DateTime<Local> = Local::now();
    return [
        String::from("content-type: text/html; charset=utf-8"),
        format!("content-length: {}", contents_len),
        format!("date: {}", now.to_rfc2822()),
        String::from("cross-origin-embedder-policy: require-corp"),
        String::from("cross-origin-opener-policy: cross-origin"),
        String::from("cross-origin-resource-policy: same-origin"),
        String::from("x-content-type-options: nosniff"),
        String::from("x-frame-options: deny"),
        String::from("referer-policy: no-referer"),
        String::from("content-security-policy: connect-src 'self'; font-src 'self'; frame-src 'self'; img-src 'self'; manifest-src 'self'; media-src 'self'; object-src 'self'; script-src 'self'; style-src 'unsafe-inline'; worker-src 'self'"),
    ];
}
