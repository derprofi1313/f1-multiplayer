//! The host server's own certificate: made once, kept in its data folder, and shared as the pin in
//! the host code (see [`crate::multiplayer::pinned`]).

use std::io;
use std::path::Path;
use std::sync::Arc;

use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};

const CERT_FILE: &str = "host-cert.der";
const KEY_FILE: &str = "host-key.der";

pub struct Identity {
    pub cert: Vec<u8>,
    key: Vec<u8>,
}

impl Identity {
    /// The certificate and key in `dir`, made and saved there the first time.
    pub fn load_or_create(dir: &Path) -> io::Result<Identity> {
        let (cert_path, key_path) = (dir.join(CERT_FILE), dir.join(KEY_FILE));
        if let (Ok(cert), Ok(key)) = (std::fs::read(&cert_path), std::fs::read(&key_path))
            && !cert.is_empty()
            && !key.is_empty()
        {
            return Ok(Identity { cert, key });
        }
        let made = rcgen::generate_simple_self_signed(vec![crate::multiplayer::pinned::SNI.to_owned()])
            .map_err(|e| io::Error::other(format!("couldn't make the host certificate: {e}")))?;
        let identity = Identity {
            cert: made.cert.der().to_vec(),
            key: made.key_pair.serialize_der(),
        };
        std::fs::create_dir_all(dir)?;
        std::fs::write(&key_path, &identity.key)?;
        std::fs::write(&cert_path, &identity.cert)?;
        Ok(identity)
    }

    pub fn pin(&self) -> [u8; 32] {
        crate::multiplayer::pinned::pin_of(&self.cert)
    }

    pub fn server_config(&self) -> io::Result<Arc<rustls::ServerConfig>> {
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .map_err(io::Error::other)?
            .with_no_client_auth()
            .with_single_cert(
                vec![CertificateDer::from(self.cert.clone())],
                PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(self.key.clone())),
            )
            .map_err(io::Error::other)?;
        Ok(Arc::new(config))
    }
}
