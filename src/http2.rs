use crate::tcp::get_tcp_rtt;
use crate::ARGS;
use anyhow::Result;
use bytes::Bytes;
use h2::server::SendResponse;
use h2::{Ping, PingPong, RecvStream};
use http::header::{ACCESS_CONTROL_ALLOW_ORIGIN, CONTENT_TYPE};
use http::Request;
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info};

#[tracing::instrument(fields(uri = %request.uri().path(), ip = %addr.ip()), skip_all)]
pub async fn handle(
    request: &mut Request<RecvStream>,
    respond: &mut SendResponse<Bytes>,
    raw_fd: i32,
    addr: SocketAddr,
    ping_pong: Arc<Mutex<PingPong>>,
) -> Result<()> {
    if request.uri().path() != "/" {
        error!(?request, "not found");
        let response = http::Response::builder()
            .status(http::StatusCode::NOT_FOUND)
            .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
            .body(())?;
        let _ = respond.send_response(response, true)?;
        return Ok(());
    }

    let header_frame = http::Response::builder()
        .status(http::StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(ACCESS_CONTROL_ALLOW_ORIGIN, "*")
        .body(())?;
    let mut send = respond.send_response(header_frame, false)?;

    let tls_rtts = tls_rtt(ping_pong).await?;
    let (tls_rtt, _std_dev, anomalies) = detect_anomalies(&tls_rtts, 0.5);
    let tcp_rtt = get_tcp_rtt(raw_fd);

    // TODO: type convert
    let is_proxy = tls_rtt - tcp_rtt as f64 > ARGS.duration as f64 * 1000.0;

    info!(?is_proxy, ?tcp_rtt, ?tls_rtt, ?anomalies, "result");

    Ok(send.send_data(
        json!({
            "is_proxy": is_proxy,
            "ip": addr.ip().to_string(),
        })
        .to_string()
        .into(),
        true,
    )?)
}

#[tracing::instrument(skip_all)]
async fn tls_rtt(ping_pong: Arc<Mutex<PingPong>>) -> Result<Vec<i32>> {
    let mut ping = ping_pong.lock().await;
    let mut tls_rtts = vec![];
    for index in 0..10 {
        let instant = tokio::time::Instant::now();
        ping.ping(Ping::opaque()).await?;
        let duration = instant.elapsed().as_micros();
        debug!(?duration, %index, "ping");
        tls_rtts.push(duration as i32);
    }
    Ok(tls_rtts)
}

#[tracing::instrument]
fn detect_anomalies(data: &[i32], threshold: f64) -> (f64, f64, Vec<i32>) {
    let sum: i32 = data.iter().sum();
    let mean = sum as f64 / data.len() as f64;

    let variance = calculate_variance(data, mean);
    let std_dev = variance.sqrt();

    let anomalies: Vec<i32> = data
        .iter()
        .filter(|&&x| (x as f64 - mean).abs() > threshold * std_dev)
        .copied()
        .collect();

    let non_anomalies_count = data.len() - anomalies.len();
    let adjusted_mean = if non_anomalies_count > 0 {
        (sum - anomalies.iter().sum::<i32>()) as f64 / non_anomalies_count as f64
    } else {
        mean
    };

    info!(?data, ?adjusted_mean, ?std_dev, ?anomalies);

    (adjusted_mean, std_dev, anomalies)
}

fn calculate_variance(data: &[i32], mean: f64) -> f64 {
    data.iter().map(|&x| (x as f64 - mean).powi(2)).sum::<f64>() / data.len() as f64
}
