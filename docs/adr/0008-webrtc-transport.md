# ADR 0008: the WebRTC transport (manual signalling, chunked data channel)

- **Status:** accepted
- **Supersedes:** the WebRTC half of ADR 0006.

## Context

ADR 0006 deferred WebRTC for two reasons: signalling (no channel for
it) and codec (the lossless surface is not video). Both resolved
without a new service:

- Signalling is **manual and CLI-shaped**: the sharer prints an offer
  blob, the operator pastes the viewer's answer blob back, and ICE
  candidates trickle as pasted lines. No rendezvous server, no account,
  no new process. This is the honest scope for a tool whose sessions
  are already arranged out of band (token + pin over chat).
- There is **no codec decision** because there is no video. The session
  is one reliable ordered data channel labelled `pcc` carrying the same
  `Message` envelopes as every other transport, chunked at 16 KiB with
  a 9-byte (total/index/last) header. A data channel is what the
  lossless surface wants; an encoded video stream would destroy the
  exactness the whole project exists for.

## Decision

`--transport webrtc`. Sharer: `webrtc_host_offer` prints the offer
blob, reads the answer blob plus trickle candidates from stdin (empty
line ends them), prints its own trickle candidates as they arrive.
Viewer: `--offer` joins, prints the answer blob immediately (the
connection cannot form until the sharer applies it — returning the blob
before waiting is load-bearing, not cosmetic), prints its own
candidates, waits for the channel, then runs the standard session.

ICE is non-trickle SDP plus trickle candidates, host-only (no STUN
server configured). Receipt from the loopback probe: on a network that
blackholes UDP to public STUN, configured STUN stalls gathering
forever; host candidates gather immediately and two local peers connect.
A symmetric-NAT pair needs a TURN server the project does not run —
that pair is what the quic/iroh legs are for.

## Constraints

- One channel, one label (`pcc`), reliable + ordered. Unreliable or
  out-of-order delivery would break the revision chain the compositor
  depends on.
- 16 KiB chunking below `Message`: the stack caps data-channel
  messages at 16 KiB (`a=max-message-size:65536` in both SDPs, enforced
  at 16384). The reassembler rejects envelopes over the standard frame
  budget and out-of-range indices before allocating.
- The E2E handshake runs unchanged on top: token proof inside the
  data channel, DTLS fingerprints riding in the SDP underneath. Two
  independent authentications, neither trusting the other.
- No datagram audio: the serve loop gets no connection handle, same as
  iroh and the relay fan.

## Consequences

- New dependency: `webrtc 0.21` (runtime-tokio). Smaller than iroh but
  still substantial; same correct coupling (remove it, the variant
  stops compiling).
- Manual signalling is genuinely awkward for more than two humans and
  one session. If sessions ever need arranging at scale, the answer is
  a signalling channel for the blobs that already exist — not a new
  handshake.
- Verified: loopback probe (two peers, one process, host candidates,
  "hello" each way) plus chunk/reassemble unit tests. The two-process
  CLI run is the remaining receipt before this is called done.
