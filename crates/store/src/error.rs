//! Store errors, classified for retry and user prompts (storage-backends §Interface).

use std::fmt;
use std::time::Duration;

use opendal::ErrorKind;

use crate::redact::redact;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorClass {
    NotFound,
    AlreadyExists,
    /// Retry with backoff.
    Transient,
    /// Ask the user to log in again.
    Auth,
    Quota,
    Permanent,
}

#[derive(Debug, Clone)]
pub struct StoreError {
    class: ErrorClass,
    message: String,
    retry_after: Option<Duration>,
}

impl StoreError {
    #[must_use]
    pub fn new(class: ErrorClass, message: &str) -> Self {
        Self {
            class,
            message: redact(message),
            retry_after: None,
        }
    }

    /// The server's `Retry-After` in whole seconds, if it sent one.
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        self.retry_after
    }

    #[cfg(test)]
    pub(crate) fn with_retry_after(mut self, after: Duration) -> Self {
        self.retry_after = Some(after);
        self
    }

    #[must_use]
    pub fn class(&self) -> ErrorClass {
        self.class
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for StoreError {}

impl From<opendal::Error> for StoreError {
    fn from(e: opendal::Error) -> Self {
        let text = e.to_string();
        Self {
            retry_after: retry_after(&text),
            ..Self::new(classify(e.kind(), e.is_temporary(), &text), &text)
        }
    }
}

/// A `"retry-after": "<seconds>"` header in the response debug text. HTTP-dates give `None`.
fn retry_after(text: &str) -> Option<Duration> {
    const KEY: &str = "\"retry-after\": \"";
    let lower = text.to_ascii_lowercase();
    let rest = &lower[lower.find(KEY)? + KEY.len()..];
    let value = &rest[..rest.find('"')?];
    if value.is_empty() || !value.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    value.parse().ok().map(Duration::from_secs)
}

/// S3 error codes that mean the access key or its signature was rejected.
const S3_AUTH_CODES: [&str; 4] = [
    "Unauthorized",
    "AccessDenied",
    "InvalidAccessKeyId",
    "SignatureDoesNotMatch",
];

pub(crate) fn classify(kind: ErrorKind, temporary: bool, text: &str) -> ErrorClass {
    match kind {
        ErrorKind::NotFound => ErrorClass::NotFound,
        ErrorKind::ConditionNotMatch | ErrorKind::AlreadyExists => ErrorClass::AlreadyExists,
        // Drive reports its 403s (quota, rate limit, real denials) as PermissionDenied.
        ErrorKind::PermissionDenied if text.contains("storageQuotaExceeded") => ErrorClass::Quota,
        ErrorKind::PermissionDenied if text.contains("ateLimitExceeded") => ErrorClass::Transient,
        ErrorKind::PermissionDenied => ErrorClass::Auth,
        ErrorKind::RateLimited => ErrorClass::Transient,
        _ if temporary => ErrorClass::Transient,
        _ if text.contains("status: 401") => ErrorClass::Auth,
        _ if text.contains("storageQuotaExceeded") || text.contains("status: 507") => {
            ErrorClass::Quota
        }
        _ if text.contains("status: 429") || text.contains("status: 5") => ErrorClass::Transient,
        // S3 (R2, B2, MinIO…): a rejected key arrives as Unexpected with its HTTP status/code.
        _ if text.contains("status: 403") || S3_AUTH_CODES.iter().any(|c| text.contains(c)) => {
            ErrorClass::Auth
        }
        _ => ErrorClass::Permanent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opendal::ErrorKind;

    #[test]
    fn classifies_by_kind_then_by_http_status() {
        assert_eq!(
            classify(ErrorKind::NotFound, false, ""),
            ErrorClass::NotFound
        );
        assert_eq!(
            classify(ErrorKind::ConditionNotMatch, false, ""),
            ErrorClass::AlreadyExists
        );
        assert_eq!(
            classify(ErrorKind::PermissionDenied, false, ""),
            ErrorClass::Auth
        );
        assert_eq!(
            classify(ErrorKind::RateLimited, false, ""),
            ErrorClass::Transient
        );
        assert_eq!(
            classify(ErrorKind::Unexpected, true, ""),
            ErrorClass::Transient
        );
        // OpenDAL's gdrive service reports a 401 as Unexpected (seen 2026-10-18 with a fake token).
        assert_eq!(
            classify(
                ErrorKind::Unexpected,
                false,
                "response: Parts { status: 401, ..."
            ),
            ErrorClass::Auth
        );
        assert_eq!(
            classify(
                ErrorKind::Unexpected,
                false,
                "status: 403 ... storageQuotaExceeded"
            ),
            ErrorClass::Quota
        );
        assert_eq!(
            classify(ErrorKind::Unexpected, false, "status: 503"),
            ErrorClass::Transient
        );
        assert_eq!(
            classify(ErrorKind::Unexpected, false, "status: 400"),
            ErrorClass::Permanent
        );
    }

    #[test]
    fn retry_after_is_read_from_the_response_headers() {
        let raw = |h: &str| {
            StoreError::from(opendal::Error::new(
                ErrorKind::RateLimited,
                format!("response: Parts {{ status: 429, headers: {{{h}}} }}"),
            ))
        };
        assert_eq!(
            raw(r#""content-type": "text/xml", "Retry-After": "7""#).retry_after(),
            Some(Duration::from_secs(7))
        );
        assert_eq!(raw(r#""content-type": "text/xml""#).retry_after(), None);
        assert_eq!(
            raw(r#""retry-after": "Wed, 21 Oct 2026 07:28:00 GMT""#).retry_after(),
            None
        );
        assert_eq!(
            StoreError::new(ErrorClass::Transient, r#""retry-after": "7""#).retry_after(),
            None
        );
    }

    #[test]
    fn messages_are_always_redacted() {
        let e = StoreError::new(ErrorClass::Permanent, "refresh_token=SECRET");
        assert_eq!(e.message(), "refresh_token=REDACTED");
        assert_eq!(e.to_string(), "refresh_token=REDACTED");
    }

    #[test]
    fn provider_errors_are_redacted_on_conversion() {
        let raw = opendal::Error::new(
            ErrorKind::Unexpected,
            "send failed: refresh_token=SECRET&client_secret=SECRET2",
        );
        let e = StoreError::from(raw);
        assert!(!e.message().contains("SECRET"), "{}", e.message());
        assert!(!e.to_string().contains("SECRET"));
    }

    #[test]
    fn drive_403s_arrive_as_permission_denied_but_keep_their_meaning() {
        let denied = ErrorKind::PermissionDenied;
        assert_eq!(
            classify(denied, false, "reason: storageQuotaExceeded"),
            ErrorClass::Quota
        );
        assert_eq!(
            classify(denied, false, "reason: rateLimitExceeded"),
            ErrorClass::Transient
        );
        assert_eq!(
            classify(denied, false, "reason: userRateLimitExceeded"),
            ErrorClass::Transient
        );
        assert_eq!(
            classify(denied, false, "reason: insufficientPermissions"),
            ErrorClass::Auth
        );
        assert_eq!(classify(denied, false, ""), ErrorClass::Auth);
    }

    #[test]
    fn s3_rejected_keys_are_auth() {
        for text in [
            "response: Parts { status: 403, ...",
            "<Code>Unauthorized</Code>",
            "<Code>AccessDenied</Code>",
            "<Code>InvalidAccessKeyId</Code>",
            "<Code>SignatureDoesNotMatch</Code>",
        ] {
            assert_eq!(
                classify(ErrorKind::Unexpected, false, text),
                ErrorClass::Auth,
                "{text}"
            );
        }
    }
}
