//! Talking to a self-hosted lobby server (see [`super::host`]) over TLS pinned to its certificate.
//!
//! A host server makes its own self-signed certificate, so the webpki roots can't vouch for it.
//! Instead the host shares a **host code**, `<host>:<port>#<pin>`, where the pin is the unpadded
//! base64url SHA-256 of the server's certificate. A connection is accepted only when the server
//! presents exactly that certificate and proves it holds its key; the name and validity dates in
//! the certificate are not checked. Everything the launcher sends a host server (the session
//! token, the room key it gets back) goes over this channel and nowhere else.

use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::Arc;
use std::time::Duration;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::crypto::{CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, SignatureScheme};
use sha2::{Digest, Sha256};

/// The SNI name a client sends a host server; the server ignores it and the client doesn't check
/// the certificate's names.
pub const SNI: &str = "swofty-host";
/// The port a host server listens on for HTTPS when none is given.
pub const DEFAULT_PORT: u16 = 47800;

/// A self-hosted server: its `https://host:port` base and the certificate pin.
#[derive(Clone, PartialEq, Eq)]
pub struct Endpoint {
    pub base: String,
    pub pin: [u8; 32],
}

impl std::fmt::Debug for Endpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.code())
    }
}

impl Endpoint {
    /// Read a host code: `host[:port]#pin`, optionally with `https://` in front.
    pub fn parse(code: &str) -> Result<Endpoint, String> {
        let code = code.trim();
        let rest = code
            .strip_prefix("https://")
            .or_else(|| code.strip_prefix("swofty://"))
            .unwrap_or(code);
        let (authority, pin) = rest
            .split_once('#')
            .ok_or_else(|| format!("'{code}' is not a host code (like 203.0.113.7:47800#Abc…)"))?;
        let authority = authority.trim_end_matches('/');
        if authority.is_empty() || authority.contains(['/', '?', '@', ' ']) {
            return Err(format!("'{authority}' is not a host address"));
        }
        let pin = super::rooms::decode_key(pin.trim())
            .ok_or_else(|| "the host code's certificate pin is damaged; copy it again".to_owned())?;
        let has_port = match authority.strip_prefix('[') {
            Some(v6) => v6.contains("]:"),
            None => authority.contains(':'),
        };
        let authority = if has_port {
            authority.to_owned()
        } else {
            format!("{authority}:{DEFAULT_PORT}")
        };
        Ok(Endpoint {
            base: format!("https://{authority}"),
            pin,
        })
    }

    /// The host code to share: `host:port#pin`.
    pub fn code(&self) -> String {
        format!(
            "{}#{}",
            self.base.trim_start_matches("https://"),
            crate::mpauth::base64url(&self.pin)
        )
    }

    /// `host:port` of the base URL.
    pub fn authority(&self) -> &str {
        self.base.trim_start_matches("https://")
    }
}

/// The pin of a DER certificate.
pub fn pin_of(cert_der: &[u8]) -> [u8; 32] {
    Sha256::digest(cert_der).into()
}

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

#[derive(Debug)]
struct Pinned {
    pin: [u8; 32],
    algorithms: WebPkiSupportedAlgorithms,
}

impl ServerCertVerifier for Pinned {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, rustls::Error> {
        let got = pin_of(end_entity.as_ref());
        let same = got.iter().zip(&self.pin).fold(0u8, |acc, (a, b)| acc | (a ^ b)) == 0;
        if same {
            Ok(ServerCertVerified::assertion())
        } else {
            Err(rustls::Error::General(
                "the server's certificate is not the one in the host code".into(),
            ))
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.algorithms)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.algorithms)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algorithms.supported_schemes()
    }
}

/// A rustls client config that accepts only the certificate with `pin`.
pub fn client_config(pin: [u8; 32]) -> Result<Arc<rustls::ClientConfig>, String> {
    let provider = provider();
    let verifier = Pinned {
        pin,
        algorithms: provider.signature_verification_algorithms,
    };
    let config = rustls::ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| e.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    Ok(Arc::new(config))
}

/// Wrap `tcp` in a TLS session pinned to `pin`.
pub fn tls(
    tcp: TcpStream,
    pin: [u8; 32],
) -> Result<rustls::StreamOwned<rustls::ClientConnection, TcpStream>, String> {
    let config = client_config(pin)?;
    let name = ServerName::try_from(SNI).map_err(|e| e.to_string())?;
    let conn = rustls::ClientConnection::new(config, name).map_err(|e| e.to_string())?;
    Ok(rustls::StreamOwned::new(conn, tcp))
}

pub(crate) fn dial(authority: &str, timeout: Duration) -> io::Result<TcpStream> {
    let mut addrs: Vec<SocketAddr> = authority.to_socket_addrs()?.collect();
    addrs.sort_by_key(SocketAddr::is_ipv6);
    let mut last = io::Error::new(io::ErrorKind::NotFound, format!("{authority} has no address"));
    for addr in addrs {
        match TcpStream::connect_timeout(&addr, timeout) {
            Ok(s) => return Ok(s),
            Err(e) => last = e,
        }
    }
    Err(last)
}

/// One HTTP/1.1 request to `endpoint`: returns the status and the body.
pub fn request(
    endpoint: &Endpoint,
    method: &str,
    path: &str,
    token: Option<&str>,
    body: Option<&str>,
    timeout: Duration,
) -> io::Result<(u16, String)> {
    let tcp = dial(endpoint.authority(), timeout)?;
    tcp.set_read_timeout(Some(timeout))?;
    tcp.set_write_timeout(Some(timeout))?;
    let mut stream = tls(tcp, endpoint.pin).map_err(io::Error::other)?;
    let mut head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {}\r\nUser-Agent: swofty-launcher\r\nConnection: close\r\n",
        endpoint.authority()
    );
    if let Some(token) = token {
        head.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    let body = body.unwrap_or("");
    if method != "GET" {
        head.push_str(&format!(
            "Content-Type: application/json\r\nContent-Length: {}\r\n",
            body.len()
        ));
    }
    head.push_str("\r\n");
    stream.write_all(head.as_bytes())?;
    if method != "GET" {
        stream.write_all(body.as_bytes())?;
    }
    stream.flush()?;
    let mut raw = Vec::new();
    let mut buf = [0u8; 8192];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => raw.extend_from_slice(&buf[..n]),
            // The server closes without close_notify after `Connection: close`.
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof && !raw.is_empty() => break,
            Err(e) => return Err(e),
        }
        if let Some(done) = complete(&raw) {
            raw.truncate(done);
            break;
        }
    }
    parse_response(&raw)
}

/// The length of a whole response with a `Content-Length` in `raw`, once it has arrived.
fn complete(raw: &[u8]) -> Option<usize> {
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n")? + 4;
    let head = String::from_utf8_lossy(&raw[..end]);
    let length: usize = head
        .lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.trim().parse().ok())?;
    (raw.len() >= end + length).then_some(end + length)
}

fn parse_response(raw: &[u8]) -> io::Result<(u16, String)> {
    let bad = || io::Error::new(io::ErrorKind::InvalidData, "the host server sent a broken answer");
    let end = raw.windows(4).position(|w| w == b"\r\n\r\n").ok_or_else(bad)? + 4;
    let head = String::from_utf8_lossy(&raw[..end]);
    let status: u16 = head
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .and_then(|s| s.parse().ok())
        .ok_or_else(bad)?;
    Ok((status, String::from_utf8_lossy(&raw[end..]).into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_and_writes_host_codes() {
        let pin = [7u8; 32];
        let code = format!("203.0.113.7:47800#{}", crate::mpauth::base64url(&pin));
        let e = Endpoint::parse(&code).unwrap();
        assert_eq!(e.base, "https://203.0.113.7:47800");
        assert_eq!(e.pin, pin);
        assert_eq!(e.code(), code);
        let bare = Endpoint::parse(&format!("my.host.example#{}", crate::mpauth::base64url(&pin))).unwrap();
        assert_eq!(bare.authority(), "my.host.example:47800");
        let v6 = Endpoint::parse(&format!(
            "https://[2001:db8::1]:9000#{}",
            crate::mpauth::base64url(&pin)
        ))
        .unwrap();
        assert_eq!(v6.authority(), "[2001:db8::1]:9000");
        assert!(Endpoint::parse("203.0.113.7:47800").is_err());
        assert!(Endpoint::parse("203.0.113.7:47800#short").is_err());
    }

    #[test]
    fn finds_the_end_of_a_response() {
        let raw = b"HTTP/1.1 201 Created\r\nContent-Length: 2\r\n\r\n{}extra";
        assert_eq!(complete(raw), Some(raw.len() - 5));
        assert_eq!(complete(&raw[..40]), None);
        assert_eq!(parse_response(raw).unwrap().0, 201);
    }
}
