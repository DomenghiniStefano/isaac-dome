//! The only place in the repo that talks to the network. A sequential, identifiable
//! client: a named User-Agent, a single retry, a pause after every response, so the
//! wiki sees a polite reader rather than a crawler.

use std::thread::sleep;
use std::time::Duration;

use ureq::Agent;

const USER_AGENT: &str = concat!(
    "IsaacDome-snapshot/",
    env!("CARGO_PKG_VERSION"),
    " (+https://github.com/DomenghiniStefano/isaac-dome)"
);
const TIMEOUT: Duration = Duration::from_secs(30);
/// Wait time before retrying after a transport error or a 429/5xx status.
const RETRY_AFTER: Duration = Duration::from_secs(5);
/// Pause after every response, before returning it to the caller.
const PAUSE: Duration = Duration::from_millis(250);

/// Why an attempt didn't yield a body: distinguishes what's worth retrying
/// (network, overloaded server) from a final response (404, 400, …).
enum Failure {
    Retryable(String),
    Final(String),
}

fn agent() -> Agent {
    let config = Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        // We evaluate the status ourselves: a 429 gets retried, a 404 doesn't.
        .http_status_as_error(false)
        .user_agent(USER_AGENT)
        .build();
    Agent::new_with_config(config)
}

fn attempt(agent: &Agent, url: &str) -> Result<String, Failure> {
    let mut response = agent
        .get(url)
        .call()
        .map_err(|e| Failure::Retryable(format!("transport error: {e}")))?;
    let status = response.status().as_u16();
    if status == 429 || status >= 500 {
        return Err(Failure::Retryable(format!("HTTP {status}")));
    }
    if !response.status().is_success() {
        return Err(Failure::Final(format!("HTTP {status}")));
    }
    response
        .body_mut()
        .read_to_string()
        .map_err(|e| Failure::Retryable(format!("truncated body: {e}")))
}

/// Downloads `url` as text. A transport error or a 429/5xx status earns a
/// retry after `RETRY_AFTER`; every response is followed by `PAUSE`.
pub fn get(url: &str) -> Result<String, String> {
    let agent = agent();
    let result = match attempt(&agent, url) {
        Ok(body) => Ok(body),
        Err(Failure::Final(reason)) => Err(reason),
        Err(Failure::Retryable(reason)) => {
            eprintln!("  {reason}; retrying in {} s", RETRY_AFTER.as_secs());
            sleep(RETRY_AFTER);
            attempt(&agent, url).map_err(|e| match e {
                Failure::Retryable(r) | Failure::Final(r) => r,
            })
        }
    };
    sleep(PAUSE);
    result.map_err(|reason| format!("{reason} — {url}"))
}
