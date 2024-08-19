use anyhow::Result;
use clap::Parser;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::ServerConfig;
use std::io::BufReader;
use std::sync::{Arc, LazyLock};
use tokio::net::TcpListener;
use tracing::{debug, info, trace};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

mod args;
mod http2;
mod tcp;
mod tls;

pub static ARGS: LazyLock<args::Args> = LazyLock::new(args::Args::parse);

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::Registry::default()
        .with(
            EnvFilter::builder()
                .with_default_directive(ARGS.log_level.parse().unwrap())
                .from_env_lossy(),
        )
        .with(tracing_subscriber::fmt::layer().pretty())
        .init();

    let listener = TcpListener::bind(&ARGS.addr).await?;
    info!("listening on {}", ARGS.addr);
    debug!("listening on {}", ARGS.addr);
    trace!("listening on {}", ARGS.addr);
    println!("listening on {}", ARGS.addr);

    let server_config = load_tls().expect(
        r#"
load tls certificate fail,
run:
openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 -keyout server.key -out server.crt

generate one
        "#,
    );
    let tls_acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(server_config));

    loop {
        if let Ok((socket, addr)) = listener.accept().await {
            info!(name = "Accept TCP connection from", addr = %addr.ip());

            let tls_acceptor = tls_acceptor.clone();
            tokio::spawn(tcp::handle(socket, tls_acceptor, addr));
        }
    }
}

fn load_tls() -> Result<ServerConfig> {
    info!("load key and cert");
    let cert = load_certs(&ARGS.cert)?;
    let key = load_private_key(&ARGS.key)?;
    let mut config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert, key)?;
    config.alpn_protocols = vec![b"h2".to_vec()];
    Ok(config)
}

fn load_certs(filename: &str) -> Result<Vec<CertificateDer<'static>>> {
    let certfile = std::fs::File::open(filename)?;
    let mut reader = BufReader::new(certfile);
    Ok(rustls_pemfile::certs(&mut reader)
        .map(|result| result.expect("parse cert error"))
        .collect())
}

fn load_private_key(filename: &str) -> Result<PrivateKeyDer<'static>> {
    let keyfile = std::fs::File::open(filename).expect("cannot open private key file");
    let mut reader = BufReader::new(keyfile);

    Ok(
        match rustls_pemfile::read_one(&mut reader).expect("cannot parse private key .pem file") {
            Some(rustls_pemfile::Item::Pkcs1Key(key)) => key.into(),
            Some(rustls_pemfile::Item::Pkcs8Key(key)) => key.into(),
            Some(rustls_pemfile::Item::Sec1Key(key)) => key.into(),
            _ => {
                panic!(
                    "no keys found in {:?} (encrypted keys not supported)",
                    filename
                )
            }
        },
    )
}
