use std::fs;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Arc;

use rcgen::generate_simple_self_signed;
use rustls::pki_types::CertificateDer;

pub struct TlsConfig {
    pub cert_path: PathBuf,
    pub key_path: PathBuf,
}

impl TlsConfig {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            cert_path: base_dir.join("cert.pem"),
            key_path: base_dir.join("key.pem"),
        }
    }

    pub fn exists(&self) -> bool {
        self.cert_path.exists() && self.key_path.exists()
    }

    pub fn generate(&self, peer_name: &str) -> Result<(), String> {
        if let Some(parent) = self.cert_path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {}", e))?;
        }

        let subject_alt_names = vec![peer_name.to_string(), "localhost".to_string()];
        let certified_key = generate_simple_self_signed(subject_alt_names)
            .map_err(|e| format!("Failed to generate certificate: {}", e))?;

        let cert_pem = certified_key.cert.pem();
        let key_pem = certified_key.signing_key.serialize_pem();

        fs::write(&self.cert_path, cert_pem)
            .map_err(|e| format!("Failed to write certificate: {}", e))?;

        fs::write(&self.key_path, key_pem).map_err(|e| format!("Failed to write key: {}", e))?;

        Ok(())
    }

    pub fn load_server_config(&self) -> Result<Arc<rustls::ServerConfig>, String> {
        let cert_file = fs::File::open(&self.cert_path)
            .map_err(|e| format!("Failed to open certificate: {}", e))?;
        let key_file =
            fs::File::open(&self.key_path).map_err(|e| format!("Failed to open key: {}", e))?;

        let mut cert_reader = BufReader::new(cert_file);
        let mut key_reader = BufReader::new(key_file);

        let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
            .filter_map(|r| r.ok())
            .collect();

        let key = rustls_pemfile::private_key(&mut key_reader)
            .map_err(|e| format!("Failed to parse key: {}", e))?
            .ok_or_else(|| "No private key found".to_string())?;

        let config = rustls::ServerConfig::builder()
            .with_no_client_auth()
            .with_single_cert(certs, key)
            .map_err(|e| format!("Failed to create server config: {}", e))?;

        Ok(Arc::new(config))
    }

    pub fn load_client_config(&self) -> Result<Arc<rustls::ClientConfig>, String> {
        let mut root_store = rustls::RootCertStore::empty();

        if self.cert_path.exists() {
            let cert_file = fs::File::open(&self.cert_path)
                .map_err(|e| format!("Failed to open certificate: {}", e))?;
            let mut cert_reader = BufReader::new(cert_file);

            let certs: Vec<CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
                .filter_map(|r| r.ok())
                .collect();

            for cert in certs {
                root_store
                    .add(cert)
                    .map_err(|e| format!("Failed to add certificate: {}", e))?;
            }
        }

        let config = rustls::ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        Ok(Arc::new(config))
    }

    pub fn load_client_config_insecure() -> Result<Arc<rustls::ClientConfig>, String> {
        let config = rustls::ClientConfig::builder()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(InsecureServerCertVerifier))
            .with_no_client_auth();

        Ok(Arc::new(config))
    }

    pub fn get_certificate_fingerprint(&self) -> Result<String, String> {
        let cert_file = fs::File::open(&self.cert_path)
            .map_err(|e| format!("Failed to open certificate: {}", e))?;
        let mut cert_reader = BufReader::new(cert_file);

        let cert = rustls_pemfile::certs(&mut cert_reader)
            .filter_map(|r| r.ok())
            .next()
            .ok_or_else(|| "No certificate found".to_string())?;

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(cert.as_ref());
        let result = hasher.finalize();

        Ok(hex::encode(result))
    }
}

#[derive(Debug)]
struct InsecureServerCertVerifier;

impl rustls::client::danger::ServerCertVerifier for InsecureServerCertVerifier {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        Ok(rustls::client::danger::HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        vec![
            rustls::SignatureScheme::RSA_PKCS1_SHA256,
            rustls::SignatureScheme::RSA_PKCS1_SHA384,
            rustls::SignatureScheme::RSA_PKCS1_SHA512,
            rustls::SignatureScheme::ECDSA_NISTP256_SHA256,
            rustls::SignatureScheme::ECDSA_NISTP384_SHA384,
            rustls::SignatureScheme::ECDSA_NISTP521_SHA512,
            rustls::SignatureScheme::RSA_PSS_SHA256,
            rustls::SignatureScheme::RSA_PSS_SHA384,
            rustls::SignatureScheme::RSA_PSS_SHA512,
            rustls::SignatureScheme::ED25519,
        ]
    }
}

pub fn ensure_certificates(config: &TlsConfig, peer_name: &str) -> Result<(), String> {
    if !config.exists() {
        config.generate(peer_name)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env::temp_dir;

    #[test]
    fn test_tls_config_creation() {
        let config = TlsConfig::new(temp_dir().join("sena_test_tls"));
        assert!(!config.exists());
    }

    #[test]
    fn test_certificate_generation() {
        let dir = temp_dir().join("sena_test_cert_gen");
        let config = TlsConfig::new(dir.clone());

        config.generate("Test Peer").unwrap();
        assert!(config.exists());

        let fingerprint = config.get_certificate_fingerprint().unwrap();
        assert!(!fingerprint.is_empty());

        let _ = fs::remove_dir_all(dir);
    }
}
