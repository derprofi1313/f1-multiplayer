use std::time::Duration;

use serde_json::{Value, json};

use super::{Bundle, PART_LIMIT, Part};

const MIN_PART: usize = 1 << 20;
const MAX_WAITS: usize = 5;
const MAX_WAIT: f64 = 60.;
const SERVER_RETRIES: usize = 2;
const CONTENT_MAX: usize = 2000;

/// The log server Send Logs to Developers uploads to, which posts each part to
/// the developers' Discord channel.
#[derive(Debug, Clone)]
pub struct LogServer {
    url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stage {
    Collecting,
    Packing,
    Sending { part: usize, total: usize, bytes: usize },
    Waiting { secs: u64 },
    Sent { parts: usize },
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Collecting => f.write_str("Collecting the logs…"),
            Self::Packing => f.write_str("Packing the logs…"),
            Self::Sending { total: 1, bytes, .. } => write!(f, "Sending {}…", super::size(*bytes as u64)),
            Self::Sending { part, total, bytes } => {
                write!(f, "Sending part {part}/{total} ({})…", super::size(*bytes as u64))
            }
            Self::Waiting { secs } => write!(f, "The log server asked to wait; retrying in {secs} s…"),
            Self::Sent { .. } => f.write_str("Sent — thanks!"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SendError {
    Unavailable,
    Unreachable(String),
    Status { code: u16, message: Option<String> },
    TooBig,
    RateLimited,
    Other(String),
}

impl std::fmt::Display for SendError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => f.write_str(super::UNAVAILABLE),
            Self::Unreachable(e) => write!(f, "Couldn't reach the log server: {e}"),
            Self::Status {
                code,
                message: Some(m),
            } => write!(f, "The log server answered HTTP {code}: {m}"),
            Self::Status { code, message: None } => write!(f, "The log server answered HTTP {code}"),
            Self::TooBig => f.write_str("The logs are too big to send (HTTP 413), even split into parts"),
            Self::RateLimited => {
                f.write_str("Too many logs were sent from your network recently (HTTP 429); try again later")
            }
            Self::Other(e) => f.write_str(e),
        }
    }
}

impl std::error::Error for SendError {}

impl LogServer {
    pub fn new(raw: Option<&str>) -> Option<Self> {
        let url = raw?.trim();
        (url.starts_with("https://") || url.starts_with("http://")).then(|| Self { url: url.to_owned() })
    }

    pub fn send(&self, bundle: &Bundle, progress: &dyn Fn(Stage)) -> Result<usize, SendError> {
        self.send_with(bundle, PART_LIMIT, Duration::from_secs(300), progress)
    }

    /// Upload `bundle` in parts of at most `limit` bytes. A 413 on the first
    /// part halves the limit and starts again, down to 1 MB.
    pub fn send_with(
        &self,
        bundle: &Bundle,
        mut limit: usize,
        timeout: Duration,
        progress: &dyn Fn(Stage),
    ) -> Result<usize, SendError> {
        'pack: loop {
            progress(Stage::Packing);
            let parts = bundle.parts(limit).map_err(|e| SendError::Other(e.to_string()))?;
            for part in &parts {
                progress(Stage::Sending {
                    part: part.index,
                    total: part.total,
                    bytes: part.data.len(),
                });
                match self.post(bundle, part, timeout, progress) {
                    Ok(()) => {}
                    Err(SendError::TooBig) if part.index == 1 && limit / 2 >= MIN_PART => {
                        limit /= 2;
                        continue 'pack;
                    }
                    Err(e) => return Err(e),
                }
            }
            progress(Stage::Sent { parts: parts.len() });
            return Ok(parts.len());
        }
    }

    fn post(
        &self,
        bundle: &Bundle,
        part: &Part,
        timeout: Duration,
        progress: &dyn Fn(Stage),
    ) -> Result<(), SendError> {
        let (content_type, body) = multipart(&payload(bundle, part), &part.file_name, &part.data);
        let (mut waits, mut retries) = (0, 0);
        loop {
            let response = ureq::post(&self.url)
                .header(
                    "User-Agent",
                    concat!("swofty-launcher/", env!("CARGO_PKG_VERSION")),
                )
                .header("Content-Type", &content_type)
                .config()
                .timeout_global(Some(timeout))
                .timeout_connect(Some(Duration::from_secs(15)))
                .http_status_as_error(false)
                .max_redirects(0)
                .build()
                .send(&body[..])
                .map_err(|e| SendError::Unreachable(e.to_string()))?;
            let code = response.status().as_u16();
            if (200..300).contains(&code) {
                return Ok(());
            }
            let header = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.trim().parse::<f64>().ok());
            let mut reply = response.into_body();
            let text = reply
                .with_config()
                .limit(64 << 10)
                .read_to_string()
                .unwrap_or_default();
            let reply: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
            if code == 413 || reply["code"].as_u64() == Some(40005) {
                return Err(SendError::TooBig);
            }
            if code == 429 {
                let secs = reply["retry_after"].as_f64().or(header).unwrap_or(1.).max(0.);
                if waits >= MAX_WAITS || secs > MAX_WAIT {
                    return Err(SendError::RateLimited);
                }
                waits += 1;
                progress(Stage::Waiting {
                    secs: secs.ceil() as u64,
                });
                std::thread::sleep(Duration::from_secs_f64(secs));
                progress(Stage::Sending {
                    part: part.index,
                    total: part.total,
                    bytes: part.data.len(),
                });
                continue;
            }
            if code >= 500 && retries < SERVER_RETRIES {
                retries += 1;
                std::thread::sleep(Duration::from_secs(2));
                continue;
            }
            return Err(SendError::Status {
                code,
                message: reply["message"]
                    .as_str()
                    .filter(|m| !m.is_empty())
                    .map(str::to_owned),
            });
        }
    }
}

pub fn content(bundle: &Bundle, part: &Part) -> String {
    let name: String = bundle.name.chars().filter(|&c| c != '`').collect();
    let mut s = format!("**Logs from `{name}`**");
    if part.total > 1 {
        s.push_str(&format!(" · part {}/{}", part.index, part.total));
    }
    s.push_str(&format!(
        "\nLauncher {} · {} · F1 25 {}",
        bundle.launcher, bundle.os, bundle.game
    ));
    if !bundle.message.is_empty() && part.index == 1 {
        s.push_str("\n>>> ");
        let room = CONTENT_MAX.saturating_sub(s.chars().count());
        let message: String = bundle.message.chars().take(room).collect();
        s.push_str(&message);
    }
    s.chars().take(CONTENT_MAX).collect()
}

pub fn payload(bundle: &Bundle, part: &Part) -> String {
    json!({
        "content": content(bundle, part),
        "allowed_mentions": { "parse": [] },
        "attachments": [{ "id": 0, "filename": part.file_name }],
    })
    .to_string()
}

/// A `multipart/form-data` body with `payload_json` and the zip as `files[0]`.
pub fn multipart(payload: &str, file_name: &str, data: &[u8]) -> (String, Vec<u8>) {
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());
    let mut n = 0u32;
    let boundary = loop {
        let b = format!("swofty-logs-{seed:x}-{}-{n}", std::process::id());
        if !contains(data, b.as_bytes()) && !payload.contains(&b) {
            break b;
        }
        n += 1;
    };
    let file_name = file_name.replace(['"', '\r', '\n'], "_");
    let mut body = Vec::with_capacity(data.len() + payload.len() + 512);
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"payload_json\"\r\n\
             Content-Type: application/json\r\n\r\n{payload}\r\n\
             --{boundary}\r\nContent-Disposition: form-data; name=\"files[0]\"; filename=\"{file_name}\"\r\n\
             Content-Type: application/zip\r\n\r\n"
        )
        .as_bytes(),
    );
    body.extend_from_slice(data);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

fn contains(hay: &[u8], needle: &[u8]) -> bool {
    hay.windows(needle.len()).any(|w| w == needle)
}
