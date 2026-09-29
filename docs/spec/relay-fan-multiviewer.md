# Relay fan: N viewers over one host relay connection

Design doc for fixing the multi-viewer relay bug (`src/app/share.rs::serve_viewer`
runs once per relay connection, so the first viewer's handshake sets the E2E keys
and every other viewer gets undecryptable ciphertext). Seam per AGENTS.md:
`RelayTransport::into_fan`. Design only; no implementation.

Reading list this doc is based on: `src/relay.rs` (read_frame, handle_client,
RelayTransport, RelaySink/Source), `src/app/share.rs` (serve_viewer,
spawn_relay_loop, capture_loop), `src/network/protocol.rs` (encode/decode,
read_len_prefix, read_framed/read_envelope), `src/network/transport.rs`
(MessageSink/Source/Transport traits, SealedSink/Source, write_encoded),
`src/network/e2e.rs` (host_handshake/viewer_handshake, seal/open, Direction),
`src/app/view.rs` (SealedSession, receive_once), `tests/transport.rs`
(relay tests), AGENTS.md relay invariants.

---

## 0. The invariant this must respect

AGENTS.md (relay section):

> The relay's framing is length-preserving, and the prefix is re-attached on
> read. `read_frame` returns `u32 len || payload` … Any envelope must therefore
> go *inside* the length, with the prefix adjusted -- prepending bytes to the
> queued frame without fixing the prefix makes every peer's `read_framed` read a
> stale length and corrupt the stream. This was found the hard way: an attempt
> to give each relayed frame a peer id regressed two passing relay tests.

Every transform below therefore changes the outer `u32 LE` length *in the same
buffer operation* that inserts or removes the peer id. The relay stays a pure
byte pipe: it never parses `Message` contents, never looks at ciphertext.

---

## 1. Exact wire bytes

### Viewer leg — UNCHANGED (both directions)

The viewer's `RelayTransport` keeps speaking today's framing. Old viewers work
against a new host + relay combination, byte for byte; the fan is invisible to
them.

```
0   u32 LE   wire_len = 5 + body_len          (≤ MAX_MESSAGE_SIZE + 5)
4   u8       PROTOCOL_VERSION                 (5 today)
5   u32 LE   body_len                         (the envelope's own inner length)
9   body     kind byte + fields, or a sealed blob
```

A sealed blob inside the envelope is `[ctr u32 LE][ciphertext][16-byte tag]`
(as produced by `e2e::Direction::seal`), length-prefixed exactly like the
plaintext envelope. The relay never opens it.

### Host leg — NEW (both directions)

Identical to the viewer leg with the peer id inserted *inside the outer length*,
immediately after it:

```
0   u32 LE   wire_len = 9 + body_len          (≤ MAX_MESSAGE_SIZE + 9)
4   u32 LE   peer_id    1 ..= 0xFFFF_FFFE     (relay-assigned, see §2)
8   u8       PROTOCOL_VERSION                 (absent on control frames)
9   u32 LE   body_len                         (absent on control frames)
13  body                                     kind byte + fields, or a sealed blob
```

Endianness: little-endian everywhere, matching the rest of the wire.
`peer_id` is a u32: 4 bytes, addresses up to `MAX_VIEWERS_PER_SESSION = 64`
viewers with room to spare. `0xFFFF_FFFF` is reserved for relay control; the
host never emits it.

Length cap change required (mechanical, not a protocol version change): today
`read_len_prefix` caps a wire length at `MAX_MESSAGE_SIZE + 5`. A host-leg
frame carries the id (4 bytes) on top of the largest envelope a viewer may
legitimately send (≤ `MAX_MESSAGE_SIZE + 5`, the relay's own viewer-leg cap),
so the relay must read host-leg frames with slack 4:

```rust
// protocol.rs — new; read_len_prefix stays the slack=0 wrapper
pub fn read_len_prefix_with_slack(len_buf: &[u8; 4], slack: usize) -> Result<usize>;
```

The relay's viewer-leg read and every other caller stay at slack 0.

### Relay transforms (the two places the invariant applies)

- **viewer → host**: read viewer-leg `[len][env]`; queue `[len+4][id][env]`,
  where `id` is that viewer's assigned id. Length fixed in the same step.
- **host → viewer**: read host-leg `[len][id][env]`; look up the viewer by id;
  queue `[len-4][env]`. Length fixed in the same step.

The envelope bytes (`version … body`) are forwarded verbatim in both
directions; only the outer 4-byte prefix and the 4-byte id are rewritten.
`Message::decode`/`read_framed` on either endpoint never sees the id.

### Relay control frames (relay → host only)

Sent on the **same channel and same ordering** as data (they must not overtake
data), with `peer_id = 0xFFFF_FFFF` and no version field:

```
0   u32 LE   wire_len = 9
4   u32 LE   0xFFFF_FFFF
8   u8       kind
9   u32 LE   viewer_id

kind 0xEF  ViewerHere — a viewer with this id is registered for this session
              (sent to the host at host registration, one per resident viewer)
kind 0xEE  ViewerLeft — the viewer with this id disconnected (host's
              registration for it is gone)
```

Control frames carry no secrets (ids only) and are fire-and-forget; the host
treats them as advisory.

### Registration envelope — unchanged

`[u32 LE reg_len][bincode RelayRegister{session, role, token}]` + 1-byte
`PROTOCOL_VERSION` ack — identical to today. The relay-leg version question is
covered in §6 (no bump needed; `RelayRegister` may later grow a
`relay_leg_version: u8` field so an *old host* fails with a clear message at
registration instead of having its frames dropped — see §6 flag).

---

## 2. Relay server routing (`handle_client`)

**Role disambiguation**: each connection's `reg.role` (already checked at
registration) decides which framing the handler parses — viewer frames are
viewer-leg, host frames are host-leg. No ambiguity: a host speaks only host-leg
frames on its connection, a viewer only viewer-leg.

**Peer identity**: `Session` gains a per-session monotonic viewer-id counter;
`Peer` gains the id.

```rust
struct Peer {
    gen: u64,
    id: u32,              // NEW: 0 for the host; relay-assigned 1..=0xFFFF_FFFE for viewers
    tx: mpsc::Sender<Vec<u8>>,
}
struct Session {
    host: Option<Peer>,
    viewers: Vec<Peer>,
    next_viewer_id: u32,  // NEW: per-session counter, starts at 1, never reused
    last_seen: Instant,
}
```

Id allocation happens under the session lock at registration, alongside the
existing `MAX_VIEWERS_PER_SESSION` check. A reconnecting viewer is a new
registration and gets a new id (gen semantics unchanged; ids are a session
property, so a **replacement host sees the same ids** its predecessor saw).

**Forwarding (inbound arm), by role**:

- Host frame: parse `[len][id][env]` (cap `MAX_MESSAGE_SIZE + 9`). If
  `id == 0xFFFF_FFFF` → invalid direction (hosts never send control) →
  drop + warn. Else look up `session.viewers.find(id)`; **unicast**: the
  target is exactly that viewer's queue, stripped to `[len-4][env]`. If no
  viewer has that id → drop the frame + warn (stale id after a ViewerLeft, or
  a nonconforming host) — do not tear anything down, do not reply. This
  **replaces** today's host→every-viewer broadcast; in the fan world the host
  has nothing to broadcast (every payload is a per-viewer reply or a sealed
  blob for one viewer).
- Viewer frame: read `[len][env]` as today; queue `[len+4][id][env]` to the
  host's queue (`session.host.tx`), fixing the prefix in the same step.

**Queueing and budgets — unchanged per viewer**: each viewer keeps its own
`mpsc` channel (capacity `MAX_PEER_QUEUE_MESSAGES = 64`) and its own byte
accounting behind `MAX_PEER_QUEUE_BYTES = 8 MiB`; the host's outbound channel
capacity bumps to `MAX_PEER_QUEUE_MESSAGES + MAX_VIEWERS_PER_SESSION` (128)
so a ViewerHere replay burst (≤64 frames, §4) cannot overflow it. One change
of *semantics* to keep: a full target queue still means "that peer is not
draining" and the relay still tears down the *sender's* connection
(`congested → break`), i.e. a stalled viewer currently bounces the host
connection, which reconnects in `spawn_relay_loop`. That is today's behavior
(each viewer was disconnected indirectly via the host loop); the fan keeps it —
see §6 for why per-viewer kill switches are out of scope. Fix the log message
to say what actually happens: "relay dropped host: viewer <id> not draining".

**Unknown peer id at the relay**: drop + warn, as above. Consequence: frames
sent to a viewer that already left cost nothing and hurt nobody; the host
learns of the departure via ViewerLeft.

**Host departure** (host handler exit, after the existing gen-guarded
unregister): if `session.host` is now `None` (no replacement host is
registered), **drop every remaining viewer registration** — remove each
`Peer` from `session.viewers` and drop its `tx` Sender. Dropping the last
Sender makes each viewer handler's outbound `rx.recv()` return `None`, so the
viewer's relay connection is closed by its own loop and the viewer's app
reconnect loop re-dials with a fresh registration, fresh id, fresh Hello and
fresh E2E keys. This is what makes recovery possible at all: a host's keys die
with its connection, the relay never inspects contents, and an established
viewer will never re-offer on its own. (If a replacement host *is* registered,
viewers stay — that is the takeover case, §4/§6.)

---

## 3. Host side: the `into_fan` seam

`relay.rs` owns the connection and the framing; `share.rs` owns the app logic.
The seam keeps that boundary.

```rust
// relay.rs — new
pub const RELAY_ID_BYTES: usize = 4;
pub const RELAY_CONTROL_ID: u32 = 0xFFFF_FFFF;
const K_CONTROL_VIEWER_HERE: u8 = 0xEF;
const K_CONTROL_VIEWER_LEFT: u8 = 0xEE;
const FAN_SESSION_QUEUE: usize = 64;   // per-viewer control queue, host side

/// One host relay connection, multiplexed into per-viewer sessions.
/// `into_fan` splits the TLS stream, spawns the reader task (host-leg
/// framing → (peer_id, envelope) on `rx`) and the writer task
/// ((peer_id, envelope) on `tx` → host-leg framing), and hands back the
/// two channel ends. Both tasks terminate when the relay connection dies;
/// `rx` closing is the signal the caller uses to tear sessions down.
pub struct RelayFan {
    pub rx: mpsc::Receiver<(u32, Vec<u8>)>, // peer_id == RELAY_CONTROL_ID ⇒ control frame
    pub tx: mpsc::Sender<(u32, Vec<u8>)>,   // (peer_id, envelope)
}

impl RelayTransport {
    pub fn into_fan(self: Box<Self>) -> RelayFan; // host role only

    // pub for tests: one host-leg read
    pub async fn read_host_frame<R: AsyncRead + Unpin>(
        reader: &mut R,
    ) -> Result<Option<(u32, Vec<u8>)>>; // (id, env); control frames: (RELAY_CONTROL_ID, kind+viewer_id)
}

// protocol.rs — new
pub fn read_len_prefix_with_slack(len_buf: &[u8; 4], slack: usize) -> Result<usize>;
```

`RelayTransport::connect`, `split`, `RelaySink`, `RelaySource` are untouched:
they remain the viewer-leg transport (and the seam is what a future
non-fan host would not call). `serve_viewer` needs **zero changes**; it keeps
its signature and its whole body. What changes is who calls it: instead of
`spawn_relay_loop` calling it once with the raw `RelayTransport`, a fan demux
calls it once *per viewer id* with a logical per-viewer transport.

```rust
// share.rs — new
struct FanSession {                      // one logical transport per viewer
    id: u32,
    tx: mpsc::Sender<(u32, Vec<u8>)>,    // shared writer channel (clone of fan.tx)
    rx: mpsc::Receiver<Vec<u8>>,         // this viewer's envelope queue
}
// MessageTransport for FanSession:
//   send/send_encoded → tx.send((id, bytes)).await  (awaiting = backpressure;
//       Full surfaces as an error, which serve_viewer already treats as a
//       write failure → drops that viewer)
//   recv               → Message::decode(rx.recv().await)
//   split              → (FanSink{id, tx}, FanSource{rx})
// MessageSink for FanSink   : send = encode+send_encoded; send_encoded → tx.send((id, bytes))
// MessageSource for FanSource: recv = decode(rx.recv()); recv_raw = rx.recv()
```

`FanSource::recv_raw` is what makes the E2E layers work unchanged:
`SealedSource::recv` calls `inner.recv_raw()` to get the ciphertext envelope
before opening it, exactly as `RelaySource::recv_raw` does today on the
viewer leg.

**Flow inside `spawn_relay_loop`** (was: one `serve_viewer`):

```rust
// per relay connection:
let fan = transport.into_fan();
let mut sessions: HashMap<u32, mpsc::Sender<Vec<u8>>> = HashMap::new();
let (exit_tx, mut exit_rx) = mpsc::channel::<u32>(64); // session-finished notices
tokio::spawn(async move {   // the fan demux — one task owns the map
    tokio::select! {
        frame = fan.rx.recv() => {
            let (id, bytes) = frame? else break;          // relay connection lost
            if id == RELAY_CONTROL_ID {
                handle_relay_control(&bytes, &mut sessions);  // ViewerLeft → remove + drop sender
                continue;
            }
            // Implicit join: first frame from an unknown id spawns the session.
            let send = sessions.entry(id).or_insert_with(|| {
                let (tx, rx) = mpsc::channel::<Vec<u8>>(FAN_SESSION_QUEUE);
                tokio::spawn(async move {
                    serve_viewer(
                        Box::new(FanSession::new(id, fan.tx.clone(), rx)),
                        relay_addr, format!("relay id={id}"),
                        tx /*share broadcast*/, published, token, viewers,
                        next_id, audio, None, metrics,
                    ).await;
                    let _ = exit_tx.send(id).await;       // cleanup notice
                });
                tx
            });
            send.send(bytes).await?;   // backpressure; errors drop the session
        }
        exited = exit_rx.recv() => { let Some(id) else break; sessions.remove(id); }
    }
    // connection lost: drop every session sender; each serve_viewer's
    // source.recv() then errors and tears the session down.
    for (_, tx) in sessions { drop(tx); }
});
```

The one task owning `sessions` means no lock; the `exit_rx` arm makes session
death visible so a dead session's queue cannot fill and block the demux. On
relay connection loss the demux breaks and drops every sender, all
`serve_viewer` tasks exit via their closed sources, and the `viewers` stats
map is cleaned by their normal exit paths — then `spawn_relay_loop`'s existing
reconnect loop dials a fresh connection.

**Where the per-viewer handshake happens**: inside `serve_viewer`, unmodified.
Each `FanSession` runs the full protocol against one viewer: plaintext
`Hello` (token check) → `e2e::host_handshake` (`E2eOffer` in, `E2eReply` out,
all plaintext envelopes tagged with that viewer's id) → `SealedSink`/
`SealedSource` wrapping the per-viewer `FanSink`/`FanSource`. Each viewer gets
fresh `KeyPair::generate()` keys, so viewer A's ciphertext is unopenable by
viewer B — which is the bug this whole design fixes.

**Control messages (RequestKeyframe/Ack/Bye) with peer identity**: the
identity lives in the *transport* layer — the relay tags each viewer→host
frame with the viewer's id, the demux routes by id to the owning session, and
`serve_viewer`'s existing control loop handles the message. No application
message carries an id; the E2E session keys already bind every sealed frame to
one viewer, and the peer id never appears in `Message` contents.

---

## 4. `spawn_relay_loop` and viewer join/leave protocol

Summary of the lifecycle decisions:

- **Implicit join on first frame**: a frame from an unknown id spawns the
  session (as in §3). This covers the normal case: a viewer registers, sends
  `Hello`, and the relay forwards it tagged; the host demux spawns and the
  handshake proceeds. No join notification is needed for this path, and the
  first frame being anything other than `Hello` is already handled by
  `serve_viewer`'s existing rejection path.
- **`ViewerHere` replay on host registration** (control frame, §1): when any
  host registers (or a replacement takes over), the relay queues one
  `ViewerHere { id }` per currently-registered viewer, in registration order.
  *Required*, because the host learns ids only from viewer traffic and the
  viewer→host direction is silent for an idle viewer: without the replay, a
  host that connects after its viewers would never spawn a session for them.
  The demux treats `ViewerHere` exactly like a first data frame (spawn the
  session for that id).
- **`ViewerLeft` on viewer departure**: when a viewer's handler exits, the
  relay queues `ViewerLeft { id }` to the host. The demux removes the id and
  drops the session's sender; the session's `serve_viewer` exits when its
  source closes (recv error), removing its `ViewerStats` entry. This keeps
  dead sessions from sealing frames into the void forever.
- **Host departure bounces the session's viewers** (§2): if the host handler
  exits and no replacement host is registered, the relay drops all viewer
  registrations, their connections close, and the viewers' own reconnect
  loops rejoin fresh (new id, full plaintext handshake). This is the *only*
  recovery that works without a viewer change, because re-keying an
  established viewer is impossible any other way (§6 flag).

**The takeover gap (flagged)**: when host B registers *before* host A's
connection dies (the `a_reconnecting_host…` test scenario), viewers are not
bounced, and B gets `ViewerHere` ids. That works for viewers that speak after
the takeover (Hello still pending, or active Ackers). A viewer that has
*completed* its handshake and is then silent cannot be re-keyed by B and will
sit until it next sends — stale sessions are bounded by `serve_viewer`'s
10 s `HANDSHAKE_TIMEOUT` for handshake-phase sessions, but an established,
idle viewer can strand. Closing that last gap needs one of two small add-ons,
both explicitly optional: (a) a viewer-side recv watchdog ("no frame for 30 s
→ error → reconnect") in `receive_once`; or (b) an application-level re-key
message (a protocol bump). Not in the minimal design.

---

## 5. Test plan (`tests/transport.rs`, plus unit tests)

All new/changed tests use the **public** seam (`RelayTransport::connect` +
`into_fan`, `read_host_frame`, `read_len_prefix_with_slack`) and drive real
TLS, real relay, real `e2e` (offer/reply/accept/complete, `Direction::seal/
open`) and a real `Compositor` — no mocks, mirroring the file's existing
style. The host side of each test is a test-local demux built on `into_fan`
(the tests already hand-roll the sharer side via `serve_once`/`produce`).
Viewers read sealed frames via `transport.split()` → `source.recv_raw()` →
`Direction::open` → `Message::decode` → `apply`.

**Changed**

1. `the_relay_forwards_real_frames_to_an_authorized_viewer` — rewritten to
   the fan shape: the viewer sends `Hello` first (so the host can learn its
   id), the host replies with the `produce()` script tagged at that id
   (plaintext, as today), and the existing pixel-identity assertions stand.
   Keeps a plaintext regression for the tagging/transform path below the E2E
   layer. (Unchanged plaintext path = relay strips the id and fixes the
   prefix; this is the test that caught the original prefix bug.)
2. `a_reconnecting_host_keeps_its_registration` — reworked: viewer registers
   and sends `Hello`; host 1 learns `id`; host 2 registers (and receives
   `ViewerHere { id }` with the *same* id — ids are a session property); host 2
   sends a tagged `KeepAlive` to that id; the viewer receives it. Directly
   asserts the gen/cleanup fix *and* the ViewerHere replay.

**New**

3. `two_relayed_viewers_each_complete_e2e_and_are_pixel_identical` — the
   regression test for this bug. Viewers A and B each: register, send
   `Hello`, run `e2e::viewer_handshake` with their own `KeyPair`, then read
   `produce(&mut script, 15)` sealed messages and apply them to separate
   `Compositor`s. The test host: demux on `into_fan`, per-id
   `e2e::host_handshake`, seal each script message per viewer's
   `host_to_viewer` direction, send tagged. Assert: both viewers complete the
   handshake (the old code fails here — the second viewer's offer is answered
   by the first viewer's session, or bounced with "Already authorized"), both
   compositors are fresh with dimensions `(W, H)`, and both buffers equal
   `final_reference.data` — A is not holding A's stream while B silently gets
   B's ciphertext, and neither viewer decodes the other's bytes.
4. `a_viewer_only_receives_frames_addressed_to_it` — both viewers handshake;
   the host seals five updates to A's id only; A gets all five, B's
   `source.recv_raw()` times out within ~2 s. Proves the relay's
   broadcast→unicast change and that the fan never leaks frames across ids.
5. `an_unknown_peer_id_is_dropped_without_disturbing_the_session` — after A's
   handshake, the host sends `(0xDEADBEEF, <valid sealed bytes>)`; the relay
   drops it; the next valid frame tagged to A still arrives; the host's
   `fan.rx` stays open. Proves unknown ids are drop-and-log, not fatal.
6. `a_viewer_that_stops_draining_disconnects_the_host` — budget behavior:
   A handshakes then stops reading; the host floods 200 small tagged frames
   at A's id (well under the 8 MiB byte budget, over the 64-message queue
   cap); the host's `fan.rx` must observe EOF within a timeout — the relay
   bounced the host connection, exactly the existing congested semantics.
7. `the_relay_tells_the_host_a_viewer_left` — C registers, sends `Hello`
   (host learns the id), then C's transport is dropped; the host receives a
   control frame `(RELAY_CONTROL_ID, [0xEE][id])` within a timeout, and a
   subsequent frame tagged at the dead id is dropped silently (ties into 5).
8. Unit tests in `relay.rs :: mod tests` and `protocol.rs :: mod tests`:
   - `host_leg_frames_roundtrip_through_the_relay_transform`: build a
     viewer-leg frame, apply the host-bound transform (prepend id, fix
     length), parse with `read_host_frame` (id + envelope match), apply the
     viewer-bound transform (strip id, fix length), assert the original bytes
     come back exactly — the length-preserving invariant as a test.
   - `read_len_prefix_with_slack_accepts_the_peer_id_overhead`:
     `MAX_MESSAGE_SIZE + 9` parses with slack 4 and is rejected with slack 0;
     `MAX_MESSAGE_SIZE + 5` still parses at slack 0 (no regression).

Direct-path tests (`a_direct_viewer…`, token/cert rejection tests) are
untouched. The regression that two old relay tests caught once (prefix not
fixed) is re-caught by test 8's roundtrip plus test 1.

---

## 6. Risks, flags, and what NOT to build

**Where current code makes the design impossible (flagged)**

- **`read_len_prefix`'s ceiling** (`MAX_MESSAGE_SIZE + 5`) rejects legal
  host-leg frames because of the 4-byte id. This is a mechanical change
  (`read_len_prefix_with_slack(_, 4)` used only on the host leg and in
  `read_host_frame`), **not** a protocol version bump. If the relay's
  viewer-leg read were *not* touched — it isn't — old viewers stay compatible.
- **`serve_viewer`'s Hello-first protocol makes re-keying impossible without
  viewer cooperation.** A host's E2E keys die with its relay connection, the
  relay never inspects contents, and an established viewer never initiates.
  Hence the two relay behaviors this design adds: bounce-the-viewers on host
  departure (§2) and the takeover gap (§4, optional viewer watchdog). No
  viewer *wire* change and no `PROTOCOL_VERSION` bump are required.
- **`PROTOCOL_VERSION` stays 5.** The fan frames exist only on the
  relay↔host leg; viewers never see an id and keep working. Bumping would
  force old viewers off for no benefit. The remaining exposure: an *old host*
  against a new relay parses as garbage ids (its frames are dropped, its
  inbound has a bogus id → "Protocol version mismatch" → it reconnects, which
  is loud-ish but not clean). If clean rejection matters, add
  `relay_leg_version: u8` to `RelayRegister` (this *does* change the
  registration envelope and requires a relay+host upgrade together; it is the
  only candidate that touches the registration format, so it is optional and
  listed as such).
- **Messages at the size ceiling plus 20 bytes of sealing overhead** (`ctr` +
  tag) exceed the wire cap `MAX_MESSAGE_SIZE + 5` by a hair on the viewer leg
  *today*, pre-existing and unchanged. Encoders produce 1 MiB chunks — far
  below the 16 MiB cap — so it is a latent corner, not part of this fix.

**Semantic changes to be aware of**

- The relay no longer broadcasts host frames; it unicasts by id. All
  host→viewer traffic is per-viewer already (replies, per-viewer sealed
  blobs), so nothing in the app protocol needs broadcast; a future
  "broadcast to all" would add a reserved id, not a redesign.
- A host relay reconnect now bounces the session's viewers (they reconnect and
  re-handshake, ~RTT + their retry delay) instead of seamlessly resuming.
  This is the price of per-viewer keys; the old seamless resume is impossible
  without relay content inspection.
- A stalled viewer still ends up disconnecting the *host* (relay congestion
  rule), which reconnects; per-viewer kill switches at the relay are
  explicitly out of scope.

**Risks**

- Head-of-line blocking on the one host TCP stream (shared writer) — identical
  to today's single-connection model; per-viewer relay queues absorb burst
  variance. The fan's awaited per-session sends convert TLS stall into session
  backpressure, which the relay already bounds per viewer (64 msgs / 8 MiB).
- Malicious/misbehaving relay: could inject `ViewerHere`/mis-tagged frames.
  Worst case is session churn or DoS of the *host* (bounded by
  `MAX_VIEWERS_PER_SESSION` and the 10 s handshake timeout); confidentiality
  is untouched because keys never cross the relay and a mis-routed sealed
  frame fails AEAD.
- Zombie prevention relies on the demux owning the session map; the `exit_rx`
  arm and ViewerLeft keep dead sessions from filling queues (which would
  otherwise stall the whole fan).

**Explicitly NOT building**

- No changes to the viewer leg, `Message` framing, `PROTOCOL_VERSION`,
  registration semantics (unless the optional `relay_leg_version`), or any
  viewer code.
- No relay content inspection, no buffering or replaying *frames* (only the
  supremely cheap id bookkeeping; `ViewerHere` carries ids, not bytes).
- No per-viewer congestion kill switch at the relay; no per-viewer byte
  budgets host-side beyond the channel capacities; no control-frame
  acknowledgements.
- No audio over the relay (still datagram-only, still direct-QUIC-only).
- No re-key mid-session, no host→viewer application-level ACKs, no web/browser
  changes (the web path is host-side and unaffected).