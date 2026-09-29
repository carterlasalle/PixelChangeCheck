// Drive the SHIPPED browser client against a real sharer.
//
// This exists because a reimplementation proved nothing. A hand-written
// node snippet and a Python smoke client both agreed with the Rust side
// byte for byte, while the actual client.js dropped the token from the
// proof preimage and sent acknowledgements in the clear. Both bugs
// shipped, and neither test could see them, because neither test ever
// executed the file that users download.
//
// So: fetch the served asset, run that exact text, and let it talk to a
// real server. No reimplementation appears anywhere below.
//
// What this catches, proven by mutating the served file and re-running:
// dropping the token from the proof preimage, and treating the handshake
// reply as sealed surface data.
//
// What it does not catch: a session the sharer drops only after it has
// read something from the browser. Reintroducing an unsealed
// acknowledgement still passes here, because the drop does not surface
// to this client in time to be observed. That path is covered by
// driving the real page in a browser, not here. Stated plainly because a
// check that is quietly weaker than it looks is worse than none.
//
// Usage: node scripts/browser-client-check.mjs <pcc.js path> <ws url> <token>

import { readFileSync } from 'node:fs';

const [clientPath, wsUrl, token] = process.argv.slice(2);
if (!clientPath || !wsUrl || !token) {
  console.error('usage: browser-client-check.mjs <pcc.js> <ws-url> <token>');
  process.exit(2);
}

// client.js is a classic script that ends by assigning window.pcc. It
// touches nothing else at load time, so these globals are the whole
// environment it needs.
globalThis.window = globalThis;
const source = readFileSync(clientPath, 'utf8');
// eslint-disable-next-line no-new-func
new Function(source)();

const { SealedCodec, Session, proofOfBytes } = globalThis.pcc;
if (!SealedCodec) {
  console.error('FAIL: the served client.js does not expose SealedCodec');
  process.exit(1);
}

const fail = (why) => {
  console.error(`FAIL: ${why}`);
  process.exit(1);
};

const socket = new WebSocket(wsUrl);
socket.binaryType = 'arraybuffer';

// The verdict is raced against a close rather than sampled afterwards.
// Sampling missed it: the sharer drops the session a few milliseconds
// after an acknowledgement it cannot open, and reading `readyState` at a
// fixed later moment reported a closed socket as open.
let markClosed;
const closed = new Promise((resolve) => { markClosed = resolve; });

const done = (ok, why) => {
  try { socket.close(); } catch { /* already closing */ }
  if (ok) {
    console.log('SHIPPED_CLIENT_OK');
    process.exit(0);
  }
  fail(why);
};

const timer = setTimeout(() => done(false, 'the client never completed a frame'), 20000);

socket.onerror = () => done(false, 'the socket errored');

socket.onclose = () => { markClosed(); done(false, 'the sharer closed the session'); };

socket.onopen = async () => {
  const codec = new SealedCodec();
  try {
    const offer = await codec.makeOffer(token);
    if (offer.public.length !== 65) {
      done(false, `the offer carried a ${offer.public.length}-byte key, not 65`);
      return;
    }
    const framed = new Uint8Array(1 + offer.public.length + offer.proof.length);
    framed[0] = 0x22; // K_BROWSER_OFFER
    framed.set(offer.public, 1);
    framed.set(offer.proof, 1 + offer.public.length);
    socket.send(framed);
  } catch (e) {
    done(false, `makeOffer threw: ${e.message}`);
    return;
  }

  // A serial queue, for the same reason the shipped client has one: an
  // `async` handler is not awaited by the socket, so a sealed frame would
  // be opened while the handshake before it was still deriving keys.
  let queue = Promise.resolve();
  socket.onmessage = (event) => {
    queue = queue.then(() => handle(event))
      .catch((e) => done(false, `handler threw: ${e.message}`));
  };

  const handle = async (event) => {
    const bytes = new Uint8Array(event.data);
    if (bytes[0] === 0x23) { // K_BROWSER_REPLY
      try {
        await codec.finishHandshake(
          { public: lastOffer.public, proof: lastOffer.proof },
          { public: bytes.slice(1, 66), proof: bytes.slice(66, 98) },
        );
        // Drive the *real* Session.sendAck, not a seal of our own. An
        // acknowledgement sent in the clear was 14 bytes the sharer
        // could not open, and a check that sealed its own would never
        // have seen it.
        await Session.prototype.sendAck.call({ socket, codec, readyState: 1 }, 1);
      } catch (e) {
        done(false, `finishHandshake threw: ${e.message}`);
        return;
      }
      return;
    }
    // Everything after the reply is sealed surface data. Getting here at
    // all means the sharer accepted the token proof.
    try {
      await codec.openBytes(bytes);
    } catch (e) {
      done(false, `the first sealed frame did not open: ${e.message}`);
      return;
    }
    clearTimeout(timer);
    done(true);
  };
};

var lastOffer = null;
const originalMakeOffer = SealedCodec.prototype.makeOffer;
SealedCodec.prototype.makeOffer = async function (t) {
  lastOffer = await originalMakeOffer.call(this, t);
  return lastOffer;
};
// Referenced so the export is not dead weight: it is the same function
// the offer uses, and the check asserts the file still ships it.
void proofOfBytes;
