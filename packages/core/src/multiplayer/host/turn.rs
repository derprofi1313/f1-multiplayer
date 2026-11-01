//! A small TURN server (RFC 8656 over UDP) for a host server, enough for the launcher's
//! [`TurnTransport`](crate::multiplayer::turn::TurnTransport): Allocate, Refresh,
//! CreatePermission, ChannelBind, Send indications and ChannelData, with long-term credentials
//! whose passwords are TURN REST passwords ([`credentials`]).
//!
//! Relays never leave this process. Each allocation gets a virtual relayed address
//! `100.64.x.y:40000` that only means something here: data a client sends to the relayed address
//! of another allocation on this server is handed to that allocation's client, as ChannelData when
//! it bound a channel to the sender and as a Data indication otherwise. So the host only opens one
//! UDP port, the same number as the server's TCP port.

use std::collections::HashMap;
use std::io;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::multiplayer::stun::{self, Builder, Class, Kind, Message, attr, method};

pub const REALM: &str = "swofty-host";
const DEFAULT_LIFETIME: u32 = 600;
const MAX_LIFETIME: u32 = 3600;
const PERMISSION_LIFETIME: Duration = Duration::from_secs(300);
const RELAY_PORT: u16 = 40000;
const MAX_ALLOCATIONS: usize = 256;
const TICK: Duration = Duration::from_millis(200);

/// TURN REST credentials for `user`, valid for `ttl`: `(username, password, expires_at)`.
pub fn credentials(secret: &[u8], user: &str, ttl: Duration) -> (String, String, u64) {
    let expires = now_secs() + ttl.as_secs();
    let username = format!("{expires}:{user}");
    let password = password_for(secret, &username);
    (username, password, expires)
}

fn password_for(secret: &[u8], username: &str) -> String {
    crate::mpauth::base64url(&stun::hmac_sha1(secret, username.as_bytes()))
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

struct Allocation {
    relayed: SocketAddr,
    username: String,
    transaction: [u8; 12],
    expires: Instant,
    permissions: HashMap<IpAddr, Instant>,
    /// Channel → peer, and the reverse.
    channels: HashMap<u16, SocketAddr>,
    by_peer: HashMap<SocketAddr, u16>,
}

impl Allocation {
    fn permits(&self, ip: IpAddr, now: Instant) -> bool {
        self.permissions.get(&ip).is_some_and(|t| now < *t)
    }
}

#[derive(Default)]
struct State {
    /// By the client's address (its 5-tuple on this one UDP socket).
    allocations: HashMap<SocketAddr, Allocation>,
    /// Relayed address → client address.
    relayed: HashMap<SocketAddr, SocketAddr>,
    next: u32,
}

struct Shared {
    socket: UdpSocket,
    secret: Vec<u8>,
    nonce: String,
    state: Mutex<State>,
    stop: AtomicBool,
}

pub struct TurnServer {
    shared: Arc<Shared>,
    local: SocketAddr,
    thread: Option<JoinHandle<()>>,
}

impl TurnServer {
    /// Serve on `addr` with passwords minted from `secret`.
    pub fn start(addr: SocketAddr, secret: Vec<u8>) -> io::Result<TurnServer> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(TICK))?;
        let local = socket.local_addr()?;
        let mut raw = [0u8; 16];
        crate::mpauth::getrandom(&mut raw);
        let shared = Arc::new(Shared {
            socket,
            secret,
            nonce: crate::mpauth::base64url(&raw),
            state: Mutex::default(),
            stop: AtomicBool::new(false),
        });
        let thread = {
            let shared = shared.clone();
            std::thread::Builder::new()
                .name("host-turn".into())
                .spawn(move || shared.run())?
        };
        Ok(TurnServer {
            shared,
            local,
            thread: Some(thread),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.local
    }

    /// How many allocations are live.
    pub fn allocations(&self) -> usize {
        lock(&self.shared.state).allocations.len()
    }
}

impl Drop for TurnServer {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

fn lock(m: &Mutex<State>) -> std::sync::MutexGuard<'_, State> {
    m.lock().unwrap_or_else(|e| e.into_inner())
}

fn error_value(code: u16, reason: &str) -> Vec<u8> {
    let mut v = vec![0, 0, (code / 100) as u8, (code % 100) as u8];
    v.extend_from_slice(reason.as_bytes());
    v
}

/// Who sent an authenticated request: its username and long-term key.
struct Authed {
    username: String,
    key: [u8; 16],
}

impl Shared {
    fn run(&self) {
        let mut buf = vec![0u8; 4096];
        let mut next_sweep = Instant::now();
        while !self.stop.load(Ordering::Relaxed) {
            match self.socket.recv_from(&mut buf) {
                Ok((n, from)) => self.handle(&buf[..n], from),
                Err(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
                // Windows reports an ICMP port unreachable from an earlier send as a reset.
                Err(e) if e.kind() == io::ErrorKind::ConnectionReset => {}
                Err(_) => std::thread::sleep(Duration::from_millis(10)),
            }
            let now = Instant::now();
            if now >= next_sweep {
                self.sweep(now);
                next_sweep = now + Duration::from_secs(1);
            }
        }
    }

    fn sweep(&self, now: Instant) {
        let mut s = lock(&self.state);
        let gone: Vec<SocketAddr> = s
            .allocations
            .iter()
            .filter(|(_, a)| now >= a.expires)
            .map(|(c, _)| *c)
            .collect();
        for client in gone {
            if let Some(a) = s.allocations.remove(&client) {
                s.relayed.remove(&a.relayed);
            }
        }
        for a in s.allocations.values_mut() {
            a.permissions.retain(|_, t| now < *t);
        }
    }

    fn send(&self, bytes: &[u8], to: SocketAddr) {
        let _ = self.socket.send_to(bytes, to);
    }

    fn handle(&self, buf: &[u8], from: SocketAddr) {
        match stun::kind(buf) {
            Kind::ChannelData => {
                let Some((channel, data)) = stun::parse_channel_data(buf) else {
                    return;
                };
                let route = {
                    let s = lock(&self.state);
                    s.allocations
                        .get(&from)
                        .and_then(|a| Some((a.relayed, *a.channels.get(&channel)?)))
                };
                if let Some((relayed, peer)) = route {
                    self.relay(relayed, peer, data);
                }
            }
            Kind::Stun => {
                let Some(msg) = Message::parse(buf).filter(Message::fingerprint_ok) else {
                    return;
                };
                match (msg.class, msg.method) {
                    (Class::Request, method::BINDING) => {
                        let reply = Builder::new(method::BINDING, Class::Success, msg.transaction)
                            .address(attr::XOR_MAPPED_ADDRESS, from)
                            .fingerprint()
                            .finish();
                        self.send(&reply, from);
                    }
                    (Class::Request, _) => self.request(&msg, from),
                    (Class::Indication, method::SEND) => {
                        let (Some(peer), Some(data)) =
                            (msg.address(attr::XOR_PEER_ADDRESS), msg.get(attr::DATA))
                        else {
                            return;
                        };
                        let relayed = lock(&self.state).allocations.get(&from).map(|a| a.relayed);
                        if let Some(relayed) = relayed {
                            self.relay(relayed, peer, data);
                        }
                    }
                    _ => {}
                }
            }
            Kind::Other => {}
        }
    }

    /// Hand `data` from the allocation relayed at `sender` to the allocation relayed at `peer`.
    fn relay(&self, sender: SocketAddr, peer: SocketAddr, data: &[u8]) {
        let now = Instant::now();
        let packet = {
            let s = lock(&self.state);
            let Some(sending) = s.relayed.get(&sender).and_then(|c| s.allocations.get(c)) else {
                return;
            };
            // The sender needs a permission for the peer, and the peer one for the sender.
            if !sending.permits(peer.ip(), now) {
                return;
            }
            let Some((client, target)) = s
                .relayed
                .get(&peer)
                .and_then(|c| Some((*c, s.allocations.get(c)?)))
            else {
                return;
            };
            if !target.permits(sender.ip(), now) {
                return;
            }
            let bytes = match target.by_peer.get(&sender) {
                Some(&channel) => stun::channel_data(channel, data, false),
                None => {
                    let mut tid = [0u8; 12];
                    crate::mpauth::getrandom(&mut tid);
                    Builder::new(method::DATA, Class::Indication, tid)
                        .address(attr::XOR_PEER_ADDRESS, sender)
                        .attr(attr::DATA, data)
                        .fingerprint()
                        .finish()
                }
            };
            (client, bytes)
        };
        self.send(&packet.1, packet.0);
    }

    fn reply_error(&self, msg: &Message, from: SocketAddr, code: u16, reason: &str, key: Option<&[u8; 16]>) {
        let mut b = Builder::new(msg.method, Class::Error, msg.transaction)
            .attr(attr::ERROR_CODE, &error_value(code, reason));
        if matches!(code, 401 | 438) {
            b = b
                .attr(attr::REALM, REALM.as_bytes())
                .attr(attr::NONCE, self.nonce.as_bytes());
        }
        if let Some(key) = key {
            b = b.integrity(key);
        }
        self.send(&b.fingerprint().finish(), from);
    }

    /// Check the long-term credentials on `msg`; answers the challenge itself when they fail.
    fn authenticate(&self, msg: &Message, from: SocketAddr) -> Option<Authed> {
        let (Some(username), true) = (msg.text(attr::USERNAME), msg.has(attr::MESSAGE_INTEGRITY)) else {
            self.reply_error(msg, from, 401, "Unauthorized", None);
            return None;
        };
        let fresh = username
            .split_once(':')
            .and_then(|(t, _)| t.parse::<u64>().ok())
            .is_some_and(|expires| expires > now_secs());
        let password = password_for(&self.secret, &username);
        let key = stun::long_term_key(&username, REALM, &password);
        if !fresh || !msg.check_integrity(&key) {
            self.reply_error(msg, from, 401, "Unauthorized", None);
            return None;
        }
        if msg.text(attr::NONCE).as_deref() != Some(self.nonce.as_str()) {
            self.reply_error(msg, from, 438, "Stale Nonce", None);
            return None;
        }
        Some(Authed { username, key })
    }

    fn success(
        &self,
        msg: &Message,
        from: SocketAddr,
        key: &[u8; 16],
        body: impl FnOnce(Builder) -> Builder,
    ) {
        let b = body(Builder::new(msg.method, Class::Success, msg.transaction));
        self.send(&b.integrity(key).fingerprint().finish(), from);
    }

    fn request(&self, msg: &Message, from: SocketAddr) {
        if !matches!(
            msg.method,
            method::ALLOCATE | method::REFRESH | method::CREATE_PERMISSION | method::CHANNEL_BIND
        ) {
            self.reply_error(msg, from, 400, "Bad Request", None);
            return;
        }
        let Some(who) = self.authenticate(msg, from) else {
            return;
        };
        if msg.method == method::ALLOCATE {
            self.allocate(msg, from, &who);
            return;
        }
        let now = Instant::now();
        let mut s = lock(&self.state);
        let owned = s
            .allocations
            .get(&from)
            .is_some_and(|a| a.username == who.username);
        if !owned {
            drop(s);
            self.reply_error(msg, from, 437, "Allocation Mismatch", Some(&who.key));
            return;
        }
        match msg.method {
            method::REFRESH => {
                let asked = msg.u32(attr::LIFETIME).unwrap_or(DEFAULT_LIFETIME);
                let lifetime = asked.min(MAX_LIFETIME);
                if lifetime == 0 {
                    if let Some(a) = s.allocations.remove(&from) {
                        s.relayed.remove(&a.relayed);
                    }
                } else if let Some(a) = s.allocations.get_mut(&from) {
                    a.expires = now + Duration::from_secs(u64::from(lifetime));
                }
                drop(s);
                self.success(msg, from, &who.key, |b| b.u32(attr::LIFETIME, lifetime));
            }
            method::CREATE_PERMISSION => {
                let Some(peer) = msg.address(attr::XOR_PEER_ADDRESS) else {
                    drop(s);
                    self.reply_error(msg, from, 400, "Bad Request", Some(&who.key));
                    return;
                };
                if let Some(a) = s.allocations.get_mut(&from) {
                    a.permissions.insert(peer.ip(), now + PERMISSION_LIFETIME);
                }
                drop(s);
                self.success(msg, from, &who.key, |b| b);
            }
            _ => {
                let channel = msg.u32(attr::CHANNEL_NUMBER).map(|v| (v >> 16) as u16);
                let (Some(channel), Some(peer)) = (channel, msg.address(attr::XOR_PEER_ADDRESS)) else {
                    drop(s);
                    self.reply_error(msg, from, 400, "Bad Request", Some(&who.key));
                    return;
                };
                let Some(a) = s.allocations.get_mut(&from) else {
                    return;
                };
                let clash = !(0x4000..=0x4FFF).contains(&channel)
                    || a.channels.get(&channel).is_some_and(|p| *p != peer)
                    || a.by_peer.get(&peer).is_some_and(|c| *c != channel);
                if clash {
                    drop(s);
                    self.reply_error(msg, from, 400, "Bad Request", Some(&who.key));
                    return;
                }
                a.channels.insert(channel, peer);
                a.by_peer.insert(peer, channel);
                a.permissions.insert(peer.ip(), now + PERMISSION_LIFETIME);
                drop(s);
                self.success(msg, from, &who.key, |b| b);
            }
        }
    }

    fn allocate(&self, msg: &Message, from: SocketAddr, who: &Authed) {
        let mut s = lock(&self.state);
        if let Some(a) = s.allocations.get(&from) {
            let (same, relayed) = (a.transaction == msg.transaction, a.relayed);
            let left = a.expires.saturating_duration_since(Instant::now()).as_secs() as u32;
            drop(s);
            if same {
                self.success(msg, from, &who.key, |b| {
                    b.address(attr::XOR_RELAYED_ADDRESS, relayed)
                        .address(attr::XOR_MAPPED_ADDRESS, from)
                        .u32(attr::LIFETIME, left)
                });
            } else {
                self.reply_error(msg, from, 437, "Allocation Mismatch", Some(&who.key));
            }
            return;
        }
        if msg.get(attr::REQUESTED_TRANSPORT).and_then(|v| v.first()) != Some(&17) {
            drop(s);
            self.reply_error(msg, from, 442, "Unsupported Transport Protocol", Some(&who.key));
            return;
        }
        if s.allocations.len() >= MAX_ALLOCATIONS {
            drop(s);
            self.reply_error(msg, from, 486, "Allocation Quota Reached", Some(&who.key));
            return;
        }
        let relayed = loop {
            s.next = s.next.wrapping_add(1).max(1) % (1 << 22);
            let n = s.next;
            let ip = Ipv4Addr::new(100, 64 + (n >> 16) as u8, (n >> 8) as u8, n as u8);
            let addr = SocketAddr::new(IpAddr::V4(ip), RELAY_PORT);
            if !s.relayed.contains_key(&addr) {
                break addr;
            }
        };
        let lifetime = msg
            .u32(attr::LIFETIME)
            .unwrap_or(DEFAULT_LIFETIME)
            .clamp(1, MAX_LIFETIME);
        s.allocations.insert(
            from,
            Allocation {
                relayed,
                username: who.username.clone(),
                transaction: msg.transaction,
                expires: Instant::now() + Duration::from_secs(u64::from(lifetime)),
                permissions: HashMap::new(),
                channels: HashMap::new(),
                by_peer: HashMap::new(),
            },
        );
        s.relayed.insert(relayed, from);
        drop(s);
        self.success(msg, from, &who.key, |b| {
            b.address(attr::XOR_RELAYED_ADDRESS, relayed)
                .address(attr::XOR_MAPPED_ADDRESS, from)
                .u32(attr::LIFETIME, lifetime)
        });
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::multiplayer::transport::Transport;
    use crate::multiplayer::turn::TurnTransport;

    fn client(server: &TurnServer, secret: &[u8], user: &str) -> TurnTransport {
        let (username, password, _) = credentials(secret, user, Duration::from_secs(600));
        let url = format!("turn:127.0.0.1:{}?transport=udp", server.local_addr().port());
        TurnTransport::connect(&[url], &username, &password).unwrap()
    }

    fn expect(t: &TurnTransport, from: u8, payload: &[u8]) {
        let mut buf = [0u8; 2048];
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if let Some((m, n)) = t.recv(&mut buf, Duration::from_millis(200)).unwrap() {
                assert_eq!((m, &buf[..n]), (from, payload));
                return;
            }
        }
        panic!("nothing from member {from}");
    }

    #[test]
    fn two_clients_exchange_datagrams_through_virtual_relays() {
        let secret = b"secret".to_vec();
        let server = TurnServer::start("127.0.0.1:0".parse().unwrap(), secret.clone()).unwrap();
        let (a, b) = (client(&server, &secret, "a"), client(&server, &secret, "b"));
        assert_eq!(server.allocations(), 2);
        assert_ne!(a.local_addr(), b.local_addr());
        a.set_peers(&HashMap::from([(2, b.local_addr())])).unwrap();
        b.set_peers(&HashMap::from([(1, a.local_addr())])).unwrap();
        for i in 0..20u8 {
            a.send(2, &[i; 100]).unwrap();
            expect(&b, 1, &[i; 100]);
            b.send(1, &[i; 1300]).unwrap();
            expect(&a, 2, &[i; 1300]);
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn wrong_passwords_are_refused() {
        let server = TurnServer::start("127.0.0.1:0".parse().unwrap(), b"right".to_vec()).unwrap();
        let (username, _, _) = credentials(b"right", "x", Duration::from_secs(600));
        let url = format!("turn:127.0.0.1:{}?transport=udp", server.local_addr().port());
        let err = TurnTransport::connect(&[url], &username, "wrong").err().unwrap();
        assert!(
            err.to_string().contains("401") || err.kind() == io::ErrorKind::PermissionDenied,
            "{err}"
        );
        assert_eq!(server.allocations(), 0);
    }
}
