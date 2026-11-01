//! Asking the home router to forward the host server's port (UPnP IGD), and to tell the public
//! address, so a host doesn't have to set up port forwarding by hand. Only talks to the router on
//! the local network; nothing leaves it.

use std::io;
use std::net::{Ipv4Addr, SocketAddr, UdpSocket};
use std::time::{Duration, Instant};

const SSDP: &str = "239.255.255.250:1900";
const SEARCH_FOR: Duration = Duration::from_secs(3);
const HTTP_TIMEOUT: Duration = Duration::from_secs(5);
const SERVICES: [&str; 3] = [
    "urn:schemas-upnp-org:service:WANIPConnection:2",
    "urn:schemas-upnp-org:service:WANIPConnection:1",
    "urn:schemas-upnp-org:service:WANPPPConnection:1",
];
const DESCRIPTION: &str = "F1 Multiplayer host server";

/// A router that answered and offers port mappings.
#[derive(Clone, Debug)]
pub struct Gateway {
    control: String,
    service: String,
    /// This machine's address on the router's network.
    pub local_ip: Ipv4Addr,
}

/// What [`forward`] did.
#[derive(Clone, Debug)]
pub struct Forwarded {
    pub gateway: Gateway,
    pub port: u16,
    /// The router's public address, when it told it.
    pub external_ip: Option<Ipv4Addr>,
}

impl Forwarded {
    /// Whether the router's public address is itself private (a second router or carrier NAT), so
    /// friends still can't reach the port.
    pub fn behind_another_nat(&self) -> bool {
        self.external_ip.is_some_and(|ip| {
            ip.is_private()
                || ip.is_loopback()
                || ip.is_link_local()
                || (ip.octets()[0] == 100 && ip.octets()[1] & 0xc0 == 64)
        })
    }
}

/// Find the router, forward `port` for TCP and UDP to this machine and read the public address.
pub fn forward(port: u16) -> io::Result<Forwarded> {
    let gateway = discover()?;
    for protocol in ["TCP", "UDP"] {
        add_mapping(&gateway, port, protocol)?;
    }
    let external_ip = external_ip(&gateway).ok();
    Ok(Forwarded {
        gateway,
        port,
        external_ip,
    })
}

/// Remove what [`forward`] added; errors are ignored, the router may be gone.
pub fn unforward(f: &Forwarded) {
    for protocol in ["TCP", "UDP"] {
        let _ = soap(
            &f.gateway,
            "DeletePortMapping",
            &format!(
                "<NewRemoteHost></NewRemoteHost><NewExternalPort>{}</NewExternalPort><NewProtocol>{protocol}</NewProtocol>",
                f.port
            ),
        );
    }
}

pub fn discover() -> io::Result<Gateway> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    socket.set_read_timeout(Some(Duration::from_millis(250)))?;
    let search = format!(
        "M-SEARCH * HTTP/1.1\r\nHOST: {SSDP}\r\nMAN: \"ssdp:discover\"\r\nMX: 2\r\nST: urn:schemas-upnp-org:device:InternetGatewayDevice:1\r\n\r\n"
    );
    let deadline = Instant::now() + SEARCH_FOR;
    let mut sent = 0;
    let mut buf = [0u8; 2048];
    let mut tried = Vec::new();
    while Instant::now() < deadline {
        if sent < 3 {
            socket.send_to(search.as_bytes(), SSDP)?;
            sent += 1;
        }
        let Ok((n, from)) = socket.recv_from(&mut buf) else {
            continue;
        };
        let text = String::from_utf8_lossy(&buf[..n]);
        let Some(location) = header(&text, "location") else {
            continue;
        };
        if tried.contains(&location) {
            continue;
        }
        tried.push(location.clone());
        if let Ok(gateway) = describe(&location, from) {
            return Ok(gateway);
        }
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "no router answered with UPnP port mapping (it may be off in the router's settings)",
    ))
}

fn header(response: &str, name: &str) -> Option<String> {
    response
        .lines()
        .filter_map(|l| l.split_once(':'))
        .find(|(k, _)| k.trim().eq_ignore_ascii_case(name))
        .map(|(_, v)| v.trim().to_owned())
}

/// `scheme://host:port` of an absolute URL.
fn origin(url: &str) -> Option<&str> {
    let after = url.find("://")? + 3;
    let end = url[after..].find('/').map_or(url.len(), |i| after + i);
    Some(&url[..end])
}

fn describe(location: &str, from: SocketAddr) -> io::Result<Gateway> {
    let xml = http(location, None, None)?;
    let (service, control) = SERVICES
        .iter()
        .find_map(|s| {
            let at = xml.find(&format!("<serviceType>{s}</serviceType>"))?;
            let rest = &xml[at..];
            let start = rest.find("<controlURL>")? + "<controlURL>".len();
            let end = rest[start..].find("</controlURL>")? + start;
            Some(((*s).to_owned(), rest[start..end].trim().to_owned()))
        })
        .ok_or_else(|| io::Error::new(io::ErrorKind::Unsupported, "the router offers no port mapping"))?;
    let control = if control.starts_with("http") {
        control
    } else {
        let base = origin(location).unwrap_or("");
        format!("{base}/{}", control.trim_start_matches('/'))
    };
    let probe = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0))?;
    probe.connect(from)?;
    let local_ip = match probe.local_addr()? {
        SocketAddr::V4(a) => *a.ip(),
        SocketAddr::V6(_) => return Err(io::Error::new(io::ErrorKind::Unsupported, "IPv6 router")),
    };
    Ok(Gateway {
        control,
        service,
        local_ip,
    })
}

fn add_mapping(g: &Gateway, port: u16, protocol: &str) -> io::Result<()> {
    let args = |lease: u32| {
        format!(
            "<NewRemoteHost></NewRemoteHost><NewExternalPort>{port}</NewExternalPort><NewProtocol>{protocol}</NewProtocol>\
             <NewInternalPort>{port}</NewInternalPort><NewInternalClient>{}</NewInternalClient><NewEnabled>1</NewEnabled>\
             <NewPortMappingDescription>{DESCRIPTION}</NewPortMappingDescription><NewLeaseDuration>{lease}</NewLeaseDuration>",
            g.local_ip
        )
    };
    // A day's lease, renewed while the server runs, so a forward doesn't outlive it for long; some
    // routers only take permanent ones.
    soap(g, "AddPortMapping", &args(24 * 3600)).or_else(|_| soap(g, "AddPortMapping", &args(0)))?;
    Ok(())
}

pub fn external_ip(g: &Gateway) -> io::Result<Ipv4Addr> {
    let answer = soap(g, "GetExternalIPAddress", "")?;
    let start = answer
        .find("<NewExternalIPAddress>")
        .map(|i| i + "<NewExternalIPAddress>".len());
    let ip = start
        .and_then(|s| Some(&answer[s..s + answer[s..].find('<')?]))
        .and_then(|t| t.trim().parse().ok());
    ip.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "the router didn't say its public address",
        )
    })
}

fn soap(g: &Gateway, action: &str, args: &str) -> io::Result<String> {
    let body = format!(
        "<?xml version=\"1.0\"?>\r\n<s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\"><s:Body><u:{action} xmlns:u=\"{}\">{args}</u:{action}></s:Body></s:Envelope>",
        g.service
    );
    http(
        &g.control,
        Some(&format!("\"{}#{action}\"", g.service)),
        Some(body),
    )
}

fn http(url: &str, soap_action: Option<&str>, body: Option<String>) -> io::Result<String> {
    let response = match body {
        Some(body) => ureq::post(url)
            .header("Content-Type", "text/xml; charset=\"utf-8\"")
            .header("SOAPAction", soap_action.unwrap_or(""))
            .config()
            .timeout_global(Some(HTTP_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .send(body),
        None => ureq::get(url)
            .config()
            .timeout_global(Some(HTTP_TIMEOUT))
            .http_status_as_error(false)
            .build()
            .call(),
    };
    let mut response = response.map_err(io::Error::other)?;
    let status = response.status().as_u16();
    let text = response.body_mut().read_to_string().map_err(io::Error::other)?;
    if (200..300).contains(&status) {
        Ok(text)
    } else {
        Err(io::Error::other(format!("the router answered HTTP {status}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_ssdp_headers_and_origins() {
        let answer = "HTTP/1.1 200 OK\r\nCACHE-CONTROL: max-age=120\r\nLocation: http://192.168.178.1:49000/igddesc.xml\r\n\r\n";
        assert_eq!(
            header(answer, "LOCATION").as_deref(),
            Some("http://192.168.178.1:49000/igddesc.xml")
        );
        assert_eq!(
            origin("http://192.168.178.1:49000/igddesc.xml"),
            Some("http://192.168.178.1:49000")
        );
    }

    #[test]
    fn spots_a_second_nat() {
        let gateway = Gateway {
            control: String::new(),
            service: String::new(),
            local_ip: Ipv4Addr::new(192, 168, 1, 2),
        };
        let at = |ip: Ipv4Addr| Forwarded {
            gateway: gateway.clone(),
            port: 1,
            external_ip: Some(ip),
        };
        assert!(at(Ipv4Addr::new(100, 72, 1, 1)).behind_another_nat());
        assert!(at(Ipv4Addr::new(10, 0, 0, 1)).behind_another_nat());
        assert!(!at(Ipv4Addr::new(84, 12, 3, 4)).behind_another_nat());
    }
}
