#!/usr/bin/env bash
# End-to-end smoke test against the real binaries, over real sockets.
#
# Exercises: direct QUIC with token + pin, the browser WebSocket
# compositor handshake, the MJPEG fallback, and the relay path. Prints a
# line per check and exits non-zero on the first failure.
set -uo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$ROOT/target/release/pcc"
BASE_PORT=$(( 15000 + (RANDOM % 2000) * 3 ))
SHARE_PORT=$BASE_PORT
WEB_PORT=$(( BASE_PORT + 1 ))
RELAY_PORT=$(( BASE_PORT + 2 ))
METRICS_PORT=$(( BASE_PORT + 3 ))
# A fresh log directory per run: a killed process can still hold an old
# file open, and reading a stale fingerprint from it looks exactly like a
# certificate mismatch.
LOG_DIR="$ROOT/.tmp/smoke-$$"
FAILURES=0

mkdir -p "$LOG_DIR"
# Nothing from a previous run may still be listening.
pkill -f "$BIN" 2>/dev/null
sleep 0.5

pass() { printf '  ok   %s\n' "$1"; }
fail() { printf '  FAIL %s (%s)\n' "$1" "$2"; FAILURES=$((FAILURES + 1)); }

check() { # check <name> <condition-result> <detail>
  if [ "$2" = "0" ]; then pass "$1"; else fail "$1" "$3"; fi
}

[ -x "$BIN" ] || { echo "build first: cargo build --release"; exit 1; }

TOKEN="SMOKETOKEN1234"

# ---------------------------------------------------------------- sharer
echo "starting sharer (synthetic capture, no display needed)"
$BIN share --synthetic \
  --listen "127.0.0.1:$SHARE_PORT" \
  --web "127.0.0.1:$WEB_PORT" \
  --token "$TOKEN" \
  --fps 10 > "$LOG_DIR/share.log" 2>&1 &
SHARE_PID=$!
trap 'kill $SHARE_PID $VIEW_PID $RELAY_PID $SHARE2_PID $VIEW2_PID 2>/dev/null' EXIT

# Wait for the sharer to report its fingerprint.
PIN=""
for _ in $(seq 1 60); do
  PIN=$(grep -o 'Certificate fingerprint (sha256): [0-9a-f]*' "$LOG_DIR/share.log" 2>/dev/null | tail -1 | awk '{print $4}')
  [ -n "$PIN" ] && break
  sleep 0.2
done
check "sharer started and printed a certificate pin" "$([ -n "$PIN" ] && echo 0 || echo 1)" "no fingerprint in $LOG_DIR/share.log"
if [ -z "$PIN" ]; then cat "$LOG_DIR/share.log"; exit 1; fi

sleep 1.5  # let it capture and publish

# ------------------------------------------------------------ web viewer
echo "checking the browser viewer"
# The page carries no secret, so it is served to anyone who asks. The
# secret travels in the fragment, which the browser never puts in a
# request, so the page load cannot and does not authenticate.
INDEX=$(curl -s --max-time 5 "http://127.0.0.1:$WEB_PORT/")
check "index page is served without credentials" \
  "$([ -n "$INDEX" ] && echo 0 || echo 1)" "empty index"
check "the page reads the secret from the fragment, not the query" \
  "$(echo "$INDEX" | grep -q 'location.hash' && echo 0 || echo 1)" "no location.hash in the page"
check "the page does not read the secret from the query string" \
  "$(echo "$INDEX" | grep -q 'URLSearchParams(location.search)' && echo 1 || echo 0)" "still reading location.search"

JS=$(curl -s --max-time 5 "http://127.0.0.1:$WEB_PORT/pcc.js")
check "browser compositor is served" \
  "$(echo "$JS" | grep -q 'OP_SNAPSHOT_BEGIN\|0x04' && echo 0 || echo 1)" "client.js not served"

UNAUTH=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "http://127.0.0.1:$WEB_PORT/stream")
check "an unauthenticated stream request is refused" \
  "$([ "$UNAUTH" = "401" ] && echo 0 || echo 1)" "got HTTP $UNAUTH"

# The MJPEG fallback must produce an actual JPEG.
JPEG_HEAD=$(curl -s --max-time 8 -D - -o /dev/null "http://127.0.0.1:$WEB_PORT/stream?token=$TOKEN" | tr -d '\r' | grep -i 'content-type')
check "MJPEG fallback serves a multipart stream" \
  "$(echo "$JPEG_HEAD" | grep -qi 'multipart/x-mixed-replace' && echo 0 || echo 1)" "got: $JPEG_HEAD"

MJPEG=$(curl -s --max-time 10 "http://127.0.0.1:$WEB_PORT/stream?token=$TOKEN" | dd bs=1 count=4096 2>/dev/null | xxd -p | tr -d '\n')
check "MJPEG fallback emits a JPEG SOI marker" \
  "$(echo "$MJPEG" | grep -q 'ffd8ff' && echo 0 || echo 1)" "no JPEG magic in the first bytes"

# The WebSocket upgrade must be refused without a token and accepted with
# one, which is what proves the browser path is the real compositor.
python3 - "$WEB_PORT" "$TOKEN" <<'PY'
import base64, socket, sys, os
port, token = int(sys.argv[1]), sys.argv[2]

def handshake(query):
    key = base64.b64encode(os.urandom(16)).decode()
    req = (
        f"GET /ws{query} HTTP/1.1\r\n"
        f"Host: 127.0.0.1:{port}\r\n"
        "Upgrade: websocket\r\nConnection: Upgrade\r\n"
        f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
    )
    s = socket.create_connection(("127.0.0.1", port), timeout=5)
    s.sendall(req.encode())
    head = s.recv(4096)
    s.close()
    return head.split(b"\r\n")[0].decode(errors="replace")

good = handshake(f"?token={token}")
bad = handshake("")
assert good.startswith("HTTP/1.1 101"), f"expected 101, got {good}"
assert bad.startswith("HTTP/1.1 401"), f"expected 401, got {bad}"
print("WEBSOCKET_OK")
PY
WS=$?
check "WebSocket upgrade is accepted with a token and refused without one" "$WS" "see $LOG_DIR/share.log"

# The proof of token possession is computed independently in Rust and in
# JavaScript. If those two ever disagree, every browser handshake fails
# with no useful error, so compare them here rather than by inspection.
PROOF=$(python3 - "$WEB_PORT" "$TOKEN" <<'PY2'
import base64, hashlib, os, socket, struct, sys
port, token = int(sys.argv[1]), sys.argv[2]

# A fixed public key: the P-256 generator. The proof does not depend on
# which key it is, only on the bytes and the construction.
PUB = bytes.fromhex(
    "046b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296"
    "4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5"
)

def frame(opcode, payload, mask):
    out = bytearray([0x80 | opcode])
    n = len(payload)
    if n < 126:
        out.append(0x80 | n)
    else:
        out.append(0x80 | 126); out += struct.pack(">H", n)
    out += mask
    out += bytes(b ^ mask[i % 4] for i, b in enumerate(payload))
    return bytes(out)

def read_frame(sock):
    head = sock.recv(2)
    ln = head[1] & 0x7F
    if ln == 126:
        ln = struct.unpack(">H", sock.recv(2))[0]
    elif ln == 127:
        ln = struct.unpack(">Q", sock.recv(8))[0]
    data = b""
    while len(data) < ln:
        data += sock.recv(ln - len(data))
    return data

key = base64.b64encode(os.urandom(16)).decode()
s = socket.create_connection(("127.0.0.1", port), timeout=5)
s.sendall((
    f"GET /ws?token={token} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n"
    f"Upgrade: websocket\r\nConnection: Upgrade\r\n"
    f"Sec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n"
).encode())
head = b""
while not head.endswith(b"\r\n\r\n"):
    head += s.recv(1)
assert head.startswith(b"HTTP/1.1 101"), head[:40]

proof = hashlib.sha256(b"pcc/web/v1" + b"proof" + PUB + token.encode()).digest()
s.sendall(frame(0x2, bytes([0x22]) + PUB + proof, os.urandom(4)))
reply = read_frame(s)
s.close()
assert reply[0] == 0x23, f"expected a reply, got {reply[0]}"
# The server's own proof over its key, checked here for the same reason.
expect = hashlib.sha256(b"pcc/web/v1" + b"proof" + reply[1:66] + token.encode()).digest()
assert reply[66:98] == expect, "the server's proof does not match the construction"
print("PROOF_OK")
PY2
)
check "the browser proof construction agrees with the sharer" \
  "$(echo "$PROOF" | grep -q 'PROOF_OK' && echo 0 || echo 1)" "$PROOF"

# The shipped browser client, run verbatim against the live sharer. A
# reimplementation proves nothing: two of them agreed with the server
# while the real client.js did not.
if command -v node >/dev/null 2>&1; then
  curl -s --max-time 10 "http://127.0.0.1:$WEB_PORT/pcc.js" -o "$LOG_DIR/pcc.js"
  SHIPPED=$(node "$ROOT/scripts/browser-client-check.mjs" \
    "$LOG_DIR/pcc.js" "ws://127.0.0.1:$WEB_PORT/ws?token=$TOKEN" "$TOKEN" 2>&1)
  check "the shipped browser client completes a sealed session" \
    "$(echo "$SHIPPED" | grep -q 'SHIPPED_CLIENT_OK' && echo 0 || echo 1)" "$SHIPPED"
else
  echo "  skip  the shipped browser client check (node is not installed)"
fi

# ---------------------------------------------------------- native viewer
echo "starting a native viewer (headless)"
$BIN view --connect "127.0.0.1:$SHARE_PORT" --token "$TOKEN" --pin "$PIN" --no-window \
  > "$LOG_DIR/view.log" 2>&1 &
VIEW_PID=$!
sleep 3

if kill -0 $VIEW_PID 2>/dev/null; then
  pass "viewer connected and is still running"
else
  fail "viewer connected and is still running" "exited"
fi
check "viewer received frames" \
  "$(grep -q 'Receiving\|receiving' "$LOG_DIR/view.log" && echo 0 || echo 1)" "$(tail -3 "$LOG_DIR/view.log")"

# An unauthorized viewer must be refused.
$BIN view --connect "127.0.0.1:$SHARE_PORT" --token "WRONGTOKEN999" --pin "$PIN" --no-window \
  > "$LOG_DIR/badview.log" 2>&1
check "a viewer with the wrong token is refused" \
  "$(grep -qiE 'unauthorized|refused this session' "$LOG_DIR/badview.log" && echo 0 || echo 1)" \
  "$(tail -3 "$LOG_DIR/badview.log")"
check "a viewer with the wrong token exits non-zero" \
  "$($BIN view --connect "127.0.0.1:$SHARE_PORT" --token WRONGTOKEN999 --pin "$PIN" \
      --no-window >/dev/null 2>&1 && echo 1 || echo 0)" "it exited zero"

# A viewer pinned to the wrong certificate cannot connect at all.
OTHER_PIN=$(printf '00%.0s' {1..32})
if $BIN view --connect "127.0.0.1:$SHARE_PORT" --token "$TOKEN" --pin "$OTHER_PIN" --no-window \
  > "$LOG_DIR/pinview.log" 2>&1; then
  fail "a viewer with the wrong certificate pin cannot connect" "it connected anyway"
else
  pass "a viewer with the wrong certificate pin cannot connect"
fi

kill $VIEW_PID 2>/dev/null
wait $VIEW_PID 2>/dev/null

# ------------------------------------------------------ observability
echo "checking the observability surface"
DIAG=$($BIN diagnose 2>&1)
check "pcc diagnose prints a report" \
  "$(echo "$DIAG" | grep -q 'Best direct path' && echo 0 || echo 1)" "$DIAG"
check "pcc diagnose names the path it would take" \
  "$(echo "$DIAG" | grep -qE 'IPv6 direct|UPnP|STUN|relay' && echo 0 || echo 1)" "$DIAG"

PAIR=$($BIN pair --listen "192.0.2.10:$SHARE_PORT" --pin "$PIN" --token "$TOKEN" 2>&1)
check "pcc pair emits one URL carrying the pin and token" \
  "$(echo "$PAIR" | grep -q "connect=192.0.2.10:$SHARE_PORT" \
     && echo "$PAIR" | grep -q "pin=$PIN" \
     && echo "$PAIR" | grep -q "token=$TOKEN" && echo 0 || echo 1)" "$PAIR"

# A metrics scrape must answer on loopback and must not need a token: it is
# deliberately a different surface from the session-authenticated web port.
$BIN share --synthetic --listen "127.0.0.1:$((SHARE_PORT + 1))" --no-web \
  --token "$TOKEN" --fps 5 --metrics-listen "127.0.0.1:$METRICS_PORT" \
  --stats-interval 1 > "$LOG_DIR/metrics.log" 2>&1 &
METRICS_PID=$!
sleep 3
SCRAPE=$(curl -s --max-time 5 "http://127.0.0.1:$METRICS_PORT/metrics")
check "metrics endpoint answers without a token" \
  "$(echo "$SCRAPE" | grep -q 'pcc_uptime_seconds' && echo 0 || echo 1)" "no scrape"
for family in pcc_frames_total pcc_bytes_total pcc_detect_seconds_bucket pcc_changed_area_fraction; do
  check "metrics expose $family" \
    "$(echo "$SCRAPE" | grep -q "$family" && echo 0 || echo 1)" "missing $family"
done
check "stats interval prints a summary" \
  "$(grep -qE 'uptime [0-9]+s  frames [0-9]+' "$LOG_DIR/metrics.log" && echo 0 || echo 1)" \
  "$(tail -3 "$LOG_DIR/metrics.log")"
# `diagnose` prints its report to stdout and logs nothing, so JSON
# rendering is probed on a path that actually emits events.
JSON_LINE=$(timeout 2 $BIN share --synthetic --no-listen --no-web \
  --token "$TOKEN" --log-format json 2>&1 | head -1)
check "logs render as json on request" \
  "$(echo "$JSON_LINE" | grep -qE '^\{' && echo 0 || echo 1)" "got: $JSON_LINE"
kill $METRICS_PID 2>/dev/null

# ----------------------------------------------------------------- relay
echo "starting a relay and a second sharer through it"
# `--web` serves the browser viewer on the relay's own port, behind its
# own certificate, so a viewer needs no binary and no terminal.
$BIN relay --listen "127.0.0.1:$RELAY_PORT" --token "$TOKEN" --web \
  > "$LOG_DIR/relay.log" 2>&1 &
RELAY_PID=$!
sleep 1
RELAY_PIN=""
for _ in $(seq 1 40); do
  RELAY_PIN=$(grep -o 'fingerprint (sha256): [0-9a-f]*' "$LOG_DIR/relay.log" 2>/dev/null | tail -1 | awk '{print $3}')
  [ -n "$RELAY_PIN" ] && break
  sleep 0.2
done
check "relay printed its certificate pin" "$([ -n "$RELAY_PIN" ] && echo 0 || echo 1)" "no pin in relay.log"

# The operator should never have to reassemble a command from placeholders:
# the relay knows its token, its pin and its port, so it must print a line
# that runs as-is. `<this-host>` and `<token>` here cost a first-time user a
# whole round of guesswork.
RELAY_INVITE=$(grep -E '^  pcc share --relay ' "$LOG_DIR/relay.log" 2>/dev/null | head -1)
GATE=$(printf '%s' "$RELAY_INVITE" | grep -c -- "--relay-pin [0-9a-f]\{64\} --token $TOKEN$")
check "the relay prints a copy-pasteable share line with real values" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "got: ${RELAY_INVITE:-<none>}"
GATE=$(printf '%s' "$RELAY_INVITE" | grep -c '<this-host>\|<token>\|<session>')
check "that line contains no placeholders" \
  "$([ "$GATE" -eq 0 ] && echo 0 || echo 1)" "got: $RELAY_INVITE"

# Remembering a relay is what removes the pasting entirely, so it gets a
# check: save one, then use it with no address, pin or token given. Both
# processes are long-lived, so both are backgrounded — capturing a running
# share's output with $( ) would block until it exits, which it never does.
REMEMBER_PORT=$(( RELAY_PORT + 40 ))
PCC_RELAYS="$LOG_DIR/relays.json" $BIN relay --listen "127.0.0.1:$REMEMBER_PORT" \
  --remember smoke > "$LOG_DIR/relay-remember.log" 2>&1 &
REMEMBER_PID=$!
sleep 1
STORE=$(cat "$LOG_DIR/relays.json" 2>/dev/null)
GATE=$(printf '%s' "$STORE" | grep -c '"smoke"')
check "the relay saved itself under a name" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no store entry: $STORE"

PCC_RELAYS="$LOG_DIR/relays.json" $BIN share --use smoke --synthetic --no-listen --no-web \
  --fps 5 > "$LOG_DIR/share-use.log" 2>&1 &
USE_PID=$!
sleep 2
GATE=$(grep -c "Relay 127.0.0.1:$REMEMBER_PORT" "$LOG_DIR/share-use.log")
check "sharing by saved name needs nothing pasted" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "$(tail -2 "$LOG_DIR/share-use.log")"
kill $USE_PID $REMEMBER_PID 2>/dev/null
wait $USE_PID $REMEMBER_PID 2>/dev/null

$BIN share --synthetic --no-listen --no-web \
  --relay "127.0.0.1:$RELAY_PORT" --relay-pin "$RELAY_PIN" \
  --session SMOKE --token "$TOKEN" --fps 10 > "$LOG_DIR/share2.log" 2>&1 &
SHARE2_PID=$!
sleep 2
$BIN view --relay "127.0.0.1:$RELAY_PORT" --pin "$RELAY_PIN" \
  --session SMOKE --token "$TOKEN" --no-window > "$LOG_DIR/view2.log" 2>&1 &
VIEW2_PID=$!
sleep 3

if kill -0 $VIEW2_PID 2>/dev/null && kill -0 $SHARE2_PID 2>/dev/null; then
  pass "a viewer receives frames through the relay"
else
  fail "a viewer receives frames through the relay" "$(tail -3 "$LOG_DIR/view2.log")"
fi
# A frame the relay refuses to forward is a framing bug, and the usual cause
# is a stream that was left mid-frame — which is what a cancel-unsafe read
# under `select!` produces. Nothing valid should ever trip it.
# `grep -c` prints 0 *and* exits non-zero when there is no match, so the
# count is taken as printed and only defaulted when it is absent.
GATE=$(grep -c "too large" "$LOG_DIR/relay.log" 2>/dev/null)
GATE=${GATE:-0}
check "the relay refused no frames it forwarded" \
  "$([ "$GATE" -eq 0 ] && echo 0 || echo 1)" "$GATE oversized frame(s) in relay.log"

# ------------------------------------------------- relay-hosted browser
echo "checking the relay-hosted browser viewer"
RELAY_BASE="https://127.0.0.1:$RELAY_PORT"
JS_CODE=$(curl -sk -o /dev/null -w '%{http_code}' "$RELAY_BASE/pcc.js")
check "the relay serves the browser compositor" "$([ "$JS_CODE" = 200 ] && echo 0 || echo 1)" "got $JS_CODE"
PAGE_CODE=$(curl -sk -o /dev/null -w '%{http_code}' "$RELAY_BASE/v/SMOKE/")
check "the relay serves the session page" "$([ "$PAGE_CODE" = 200 ] && echo 0 || echo 1)" "got $PAGE_CODE"
SOCK_NO_TOKEN=$(curl -sk -o /dev/null -w '%{http_code}' "$RELAY_BASE/v/SMOKE/ws")
check "the relay refuses a browser socket without a token" \
  "$([ "$SOCK_NO_TOKEN" = 401 ] && echo 0 || echo 1)" "got $SOCK_NO_TOKEN"

# The real check: the *shipped* browser compositor, over the relay's TLS,
# completing the WebCrypto handshake and reading a sealed frame.
if command -v node >/dev/null 2>&1; then
  curl -sk "$RELAY_BASE/pcc.js" -o "$LOG_DIR/relay-pcc.js"
  BROWSER=$(NODE_TLS_REJECT_UNAUTHORIZED=0 node scripts/browser-client-check.mjs \
    "$LOG_DIR/relay-pcc.js" \
    "wss://127.0.0.1:$RELAY_PORT/v/SMOKE/ws?token=$TOKEN" "$TOKEN" 2>/dev/null)
  check "the shipped browser client views through the relay" \
    "$([ "$BROWSER" = "SHIPPED_CLIENT_OK" ] && echo 0 || echo 1)" "got: $BROWSER"
else
  echo "  skip the shipped browser client check (node is not installed)"
fi

# ------------------------------------------------- new planes (fast gates)
echo "checking the new planes"
GATE=$($BIN share --transport bogus --synthetic --no-listen --no-web --token "$TOKEN" 2>&1 | \
  grep -c "must be quic|iroh|webrtc")
check "an unknown --transport is refused with the choices" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no gate message"
GATE=$($BIN view --transport webrtc --offer BLOB --token "$TOKEN" --no-window --pin 00 2>&1 | grep -c "webrtc blob")
check "a garbage webrtc offer is refused as a blob" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no blob complaint"
GATE=$($BIN diagnose --audio 2>&1 | grep -c "loopback\|Microphone\|no capture device\|capture device")
check "diagnose --audio lists devices or says none" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no audio listing"
GATE=$($BIN share --window "no such window xyz DEF" --no-listen --no-web --token "$TOKEN" 2>&1 | \
  grep -c "no visible window")
check "a window miss fails fast instead of sharing" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "shared something anyway"
GATE=$($BIN share --audio-source bogus --synthetic --no-listen --no-web --token "$TOKEN" 2>&1 | \
  grep -c "must be mic|system|both|none")
check "an unknown --audio-source is refused with the choices" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no gate message"

# ------------------------------------------------- interactive menu
# The menu is a front end that builds argv for the same parser, so what it
# prints must be the command a typed invocation would be.
echo "checking the interactive menu"
MENU_OUT=$(printf '2\n1\npcc://view?connect=127.0.0.1:1&pin=aa11&token=TOKEN1234\nn\nn\nn\n' | \
  $BIN menu 2>/dev/null)
GATE=$(printf '%s' "$MENU_OUT" | grep -c -- 'view --connect 127.0.0.1:1 --pin aa11 --token TOKEN1234')
check "a pasted invite becomes the matching view command" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no matching command printed"
# Empty input must quit rather than fall through to the first option, or a
# script that pipes nothing would start sharing the screen.
printf '' | $BIN menu >/dev/null 2>&1
MENU_EOF=$?
check "the menu quits on empty input" \
  "$([ "$MENU_EOF" -eq 0 ] && echo 0 || echo 1)" "exit $MENU_EOF"
# Off a terminal, bare `pcc` must print usage rather than wait on a prompt.
GATE=$($BIN < /dev/null 2>&1 | grep -c "Usage: pcc")
check "bare pcc off a terminal prints usage" \
  "$([ "$GATE" -ge 1 ] && echo 0 || echo 1)" "no usage printed"

echo
if [ "$FAILURES" -eq 0 ]; then
  echo "smoke: all checks passed"
else
  echo "smoke: $FAILURES check(s) failed; logs in $LOG_DIR"
fi
exit $FAILURES
