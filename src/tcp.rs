use libc::{self, __u16, __u32, __u64, __u8};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;
use tracing::{error, info};

#[tracing::instrument(skip_all)]
pub async fn handle(conn: TcpStream, acceptor: TlsAcceptor, addr: SocketAddr) {
    let socket = match acceptor.accept(conn).await {
        Ok(v) => v,
        Err(err) => {
            error!(?err, "Failed to perform TLS handshake");
            return;
        }
    };

    info!(name = "Accept TLS connection", addr = %addr.ip());

    if let Err(err) = super::tls::handle(socket, addr).await {
        error!(error = ?err, "error while handle tls connection");
    }
}

#[repr(C)]
#[derive(Default)]
pub struct TcpInfo {
    pub tcpi_state: __u8,
    pub tcpi_ca_state: __u8,
    pub tcpi_retransmits: __u8,
    pub tcpi_probes: __u8,
    pub tcpi_backoff: __u8,
    pub tcpi_options: __u8,
    pub tcpi_snd_wscale_rcv_wscale: __u8,
    pub tcpi_delivery_rate_app_limited_fastopen_client_fail: __u8,

    pub tcpi_rto: __u32,
    pub tcpi_ato: __u32,
    pub tcpi_snd_mss: __u32,
    pub tcpi_rcv_mss: __u32,

    pub tcpi_unacked: __u32,
    pub tcpi_sacked: __u32,
    pub tcpi_lost: __u32,
    pub tcpi_retrans: __u32,
    pub tcpi_fackets: __u32,

    pub tcpi_last_data_sent: __u32,
    pub tcpi_last_ack_sent: __u32,
    pub tcpi_last_data_recv: __u32,
    pub tcpi_last_ack_recv: __u32,

    pub tcpi_pmtu: __u32,
    pub tcpi_rcv_ssthresh: __u32,
    pub tcpi_rtt: __u32,
    pub tcpi_rttvar: __u32,
    pub tcpi_snd_ssthresh: __u32,
    pub tcpi_snd_cwnd: __u32,
    pub tcpi_advmss: __u32,
    pub tcpi_reordering: __u32,

    pub tcpi_rcv_rtt: __u32,
    pub tcpi_rcv_space: __u32,

    pub tcpi_total_retrans: __u32,

    pub tcpi_pacing_rate: __u64,
    pub tcpi_max_pacing_rate: __u64,
    pub tcpi_bytes_acked: __u64,
    pub tcpi_bytes_received: __u64,
    pub tcpi_segs_out: __u32,
    pub tcpi_segs_in: __u32,

    pub tcpi_notsent_bytes: __u32,
    pub tcpi_min_rtt: __u32,
    pub tcpi_data_segs_in: __u32,
    pub tcpi_data_segs_out: __u32,

    pub tcpi_delivery_rate: __u64,

    pub tcpi_busy_time: __u64,
    pub tcpi_rwnd_limited: __u64,
    pub tcpi_sndbuf_limited: __u64,

    pub tcpi_delivered: __u32,
    pub tcpi_delivered_ce: __u32,

    pub tcpi_bytes_sent: __u64,
    pub tcpi_bytes_retrans: __u64,
    pub tcpi_dsack_dups: __u32,
    pub tcpi_reord_seen: __u32,

    pub tcpi_rcv_ooopack: __u32,

    pub tcpi_snd_wnd: __u32,
    pub tcpi_rcv_wnd: __u32,

    pub tcpi_rehash: __u32,

    pub tcpi_total_rto: __u16,
    pub tcpi_total_rto_recoveries: __u16,
    pub tcpi_total_rto_time: __u32,
}

pub fn get_tcp_rtt(raw_fd: i32) -> u32 {
    let mut tcp_info = TcpInfo::default();
    let mut tcp_info_len = std::mem::size_of::<TcpInfo>() as u32;

    let _ret = unsafe {
        libc::getsockopt(
            raw_fd,
            libc::IPPROTO_TCP,
            libc::TCP_INFO,
            &mut tcp_info as *mut _ as *mut libc::c_void,
            &mut tcp_info_len,
        )
    };

    tcp_info.tcpi_rtt
}
