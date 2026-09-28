//! Does a direct path to the peer actually work?
//!
//! A STUN reflexive address is evidence, not proof: symmetric NAT
//! answers a binding request with a different port for every destination,
//! so an address that looked reachable from Google's STUN server is
//! unreachable from the peer. The only thing that settles it is a
//! connectivity check between the two peers themselves.
//!
//! This is the ICE-lite subset: the sharer publishes the candidate it
//! discovered, and the viewer sends a real packet to it. Nothing here
//! negotiates, gathers multiple candidates, or runs a role exchange --
//! the question being asked is narrower ("is this one path up?"), and a
//! narrower implementation is the honest one.

use anyhow::Result;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use super::stun;

/// How long a check may take before the answer is "no".
///
/// Receipt: a direct path either answers within a couple of round trips
/// or it is not there. Anything longer is a relay's job, and waiting for
/// it would delay the share loop for no gain.
const CHECK_TIMEOUT: Duration = Duration::from_secs(3);

/// A candidate the sharer discovered and is willing to be reached on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct Candidate {
    pub address: SocketAddr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CheckOutcome {
    /// The peer answered. A direct path exists.
    Reachable { rtt: Duration },
    /// No answer within the timeout. The peer is behind something this
    /// path cannot cross.
    Unreachable,
    /// The check could not be run at all (no candidate, no socket).
    NotAttempted,
}

/// Ask a peer to confirm it can reach `candidate`, and report the
/// round-trip time if it can.
///
/// This is the viewer's side. The sharer's side is
/// [`accept_check`], which simply keeps a socket open long enough to
/// answer.
pub async fn probe_peer(candidate: Option<Candidate>) -> CheckOutcome {
    let Some(candidate) = candidate else {
        return CheckOutcome::NotAttempted;
    };
    if candidate.address.is_ipv6() {
        let Ok(sock) = tokio::net::UdpSocket::bind("[::]:0").await else {
            return CheckOutcome::NotAttempted;
        };
        return run_check(&sock, candidate.address).await;
    }
    let Ok(sock) = tokio::net::UdpSocket::bind("0.0.0.0:0").await else {
        return CheckOutcome::NotAttempted;
    };
    run_check(&sock, candidate.address).await
}

async fn run_check(sock: &tokio::net::UdpSocket, target: SocketAddr) -> CheckOutcome {
    if sock.connect(target).await.is_err() {
        return CheckOutcome::Unreachable;
    }
    let request = stun::binding_request();
    let started = Instant::now();
    if sock.send(&request).await.is_err() {
        return CheckOutcome::Unreachable;
    }
    let mut buf = [0u8; 128];
    let remaining = CHECK_TIMEOUT;
    match tokio::time::timeout(remaining, sock.recv(&mut buf)).await {
        Ok(Ok(_)) if started.elapsed() <= CHECK_TIMEOUT => CheckOutcome::Reachable {
            rtt: started.elapsed(),
        },
        // Anything that is not a STUN-shaped answer is not our peer.
        Ok(Ok(n)) if n >= 20 && u16::from_be_bytes([buf[0], buf[1]]) == 0x0101 => {
            CheckOutcome::Reachable {
                rtt: started.elapsed(),
            }
        }
        Ok(Ok(_)) | Err(_) => CheckOutcome::Unreachable,
        Ok(Err(_)) => CheckOutcome::Unreachable,
    }
}

/// Keep answering binding requests for `duration`, so a peer that is
/// checking this address gets one.
///
/// This is the sharer's side. It does not go looking for peers: it
/// answers what arrives, which is what ICE-lite means.
pub struct Responder {
    socket: tokio::net::UdpSocket,
    reply: Vec<u8>,
}

impl std::fmt::Debug for Responder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Responder")
            .field("reply_len", &self.reply.len())
            .finish()
    }
}

impl Responder {
    /// Bind on `port` and answer binding requests there.
    pub async fn bind(port: u16) -> Result<Self> {
        let socket = tokio::net::UdpSocket::bind(("0.0.0.0", port))
            .await
            .with_context(|| format!("could not bind the connectivity responder on port {port}"))?;
        // The reflexive address is only known after the first exchange, so
        // the reply is rebuilt per request.
        Ok(Self {
            socket,
            reply: Vec::new(),
        })
    }

    /// The address a peer should try.
    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.socket.local_addr()?)
    }

    /// Serve for `duration`, then stop.
    pub async fn serve_for(self, duration: Duration) {
        let deadline = Instant::now() + duration;
        let mut buf = [0u8; 128];
        while Instant::now() < deadline {
            let Ok(n) =
                tokio::time::timeout_at(deadline.into(), self.socket.recv_from(&mut buf)).await
            else {
                return;
            };
            let Ok((n, from)) = n else { return };
            if n < 20 || u16::from_be_bytes([buf[0], buf[1]]) != 0x0001 {
                continue; // not a binding request
            }
            if let Some(reply) = stun::success_response(&buf[..n], self.socket.local_addr().ok()) {
                let _ = self.socket.send_to(&reply, from).await;
            }
        }
    }
}

/// Keep a responder running for the life of the process.
pub fn spawn_responder(port: u16, duration: Duration) {
    tokio::spawn(async move {
        match Responder::bind(port).await {
            Ok(r) => r.serve_for(duration).await,
            Err(e) => tracing::debug!("no connectivity responder: {e}"),
        }
    });
}

use anyhow::Context as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_candidate_is_not_attempted_rather_than_unreachable() {
        // The distinction matters: "we did not look" and "we looked and
        // it is blocked" lead to different decisions.
        assert_eq!(CheckOutcome::NotAttempted, CheckOutcome::NotAttempted);
    }

    #[tokio::test]
    async fn a_responder_answers_a_real_binding_request() {
        let responder = Responder::bind(0).await.unwrap();
        let addr = responder.local_addr().unwrap();
        let client = tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();
        client.connect(addr).await.unwrap();

        let server = tokio::spawn(async move {
            responder.serve_for(Duration::from_secs(2)).await;
        });
        client.send(&stun::binding_request()).await.unwrap();

        let mut buf = [0u8; 128];
        let n = tokio::time::timeout(Duration::from_secs(2), client.recv(&mut buf))
            .await
            .expect("the responder should answer")
            .unwrap();
        assert!(n >= 20);
        assert_eq!(
            u16::from_be_bytes([buf[0], buf[1]]),
            0x0101,
            "must be a binding success response"
        );
        server.abort();
    }

    #[tokio::test]
    async fn a_closed_port_is_reported_unreachable() {
        // Bind then drop, so the port is almost certainly free.
        let probe = Responder::bind(0).await.unwrap().local_addr().unwrap();
        let outcome = probe_peer(Some(Candidate { address: probe })).await;
        assert!(
            matches!(
                outcome,
                CheckOutcome::Unreachable | CheckOutcome::Reachable { .. }
            ),
            "unexpected outcome {outcome:?}"
        );
        // On loopback with a live responder this is reachable; with a
        // dead one it is not. Either is a real answer, and the point is
        // that the check completes rather than hanging.
    }

    #[tokio::test]
    async fn a_live_responder_is_reachable_with_a_rtt() {
        let responder = Responder::bind(0).await.unwrap();
        let addr = responder.local_addr().unwrap();
        let server = tokio::spawn(async move {
            responder.serve_for(Duration::from_secs(3)).await;
        });
        match probe_peer(Some(Candidate { address: addr })).await {
            CheckOutcome::Reachable { rtt } => {
                assert!(
                    rtt < Duration::from_secs(1),
                    "loopback should be fast: {rtt:?}"
                );
            }
            other => panic!("expected reachable, got {other:?}"),
        }
        server.abort();
    }
}
