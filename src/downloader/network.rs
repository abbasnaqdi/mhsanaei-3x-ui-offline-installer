use anyhow::Result;
use console::style;
use reqwest::Error as ReqwestError;
use std::time::Duration;
use tokio::time::sleep;

/// Represents the classification of a network error.
#[derive(Debug, PartialEq)]
pub enum ErrorKind {
    /// Transient error, e.g. timeout, incomplete body, connection reset. Should retry.
    Transient,
    /// Complete network failure or hard connection error. Should retry a limited number of times, then fail.
    HardConnection,
    /// Unrecoverable error, e.g. 404, 403, bad URL, decode error, or local file IO error.
    Unrecoverable,
}

pub fn classify_error(err: &anyhow::Error) -> ErrorKind {
    if let Some(req_err) = err.downcast_ref::<ReqwestError>() {
        if req_err.is_timeout() || req_err.is_body() {
            return ErrorKind::Transient;
        }
        if req_err.is_connect() {
            // DNS failure or completely unreachable network
            return ErrorKind::HardConnection;
        }
        if let Some(status) = req_err.status() {
            if status.is_client_error() {
                // 4xx errors are usually unrecoverable (e.g., 404 Not Found, 403 Forbidden)
                return ErrorKind::Unrecoverable;
            }
            if status.is_server_error() {
                // 5xx errors (e.g., 502 Bad Gateway) are transient
                return ErrorKind::Transient;
            }
        }
        if req_err.is_decode() || req_err.is_builder() || req_err.is_request() {
            return ErrorKind::Unrecoverable;
        }
        return ErrorKind::Transient;
    }
    
    // If it's not a reqwest error (e.g. disk write failure), we shouldn't endlessly retry.
    ErrorKind::Unrecoverable
}

pub fn get_backoff_duration(attempt: u32) -> Duration {
    // 2s, 4s, 8s, max 15s
    let secs = (2_u64.pow(attempt.min(3))).min(15);
    Duration::from_secs(secs)
}

/// A generic retry wrapper for futures that applies the smart retry logic.
pub async fn with_smart_retry<F, Fut, T>(mut action: F, label: &str) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut transient_attempts = 0;
    let mut hard_connect_attempts = 0;
    const MAX_HARD_CONNECT_RETRIES: u32 = 3;

    loop {
        match action().await {
            Ok(val) => return Ok(val),
            Err(e) => {
                let kind = classify_error(&e);
                match kind {
                    ErrorKind::Unrecoverable => {
                        return Err(anyhow::anyhow!("Unrecoverable error during {}: {}", label, e));
                    }
                    ErrorKind::HardConnection => {
                        hard_connect_attempts += 1;
                        if hard_connect_attempts > MAX_HARD_CONNECT_RETRIES {
                            println!(
                                "  {} {} failed completely (Network unreachable). Stopping.",
                                style("✗").red(),
                                label
                            );
                            return Err(anyhow::anyhow!(
                                "Network is completely disconnected or unreachable: {}",
                                e
                            ));
                        }
                        println!(
                            "  {} Network unreachable ({}). Retrying ({}/{})...",
                            style("ℹ").yellow(),
                            e,
                            hard_connect_attempts,
                            MAX_HARD_CONNECT_RETRIES
                        );
                        sleep(Duration::from_secs(5)).await;
                    }
                    ErrorKind::Transient => {
                        transient_attempts += 1;
                        let backoff = get_backoff_duration(transient_attempts);
                        println!(
                            "  {} Connection interrupted ({}). Retrying in {}s (Attempt {})",
                            style("ℹ").yellow(),
                            e,
                            backoff.as_secs(),
                            transient_attempts
                        );
                        sleep(backoff).await;
                    }
                }
            }
        }
    }
}
