// PixelChangeCheck browser compositor.
//
// This is a direct port of src/pcc/compositor.rs. The same invariants
// hold here: a snapshot is committed atomically, an update never reads a
// reference this buffer does not have, a revision is applied at most
// once, and a copy reads the surface as it was *before* the transaction
// (which is what makes an overlapping move or a swap work).
//
// The wire format is the same explicit little-endian one the native
// viewer parses. Nothing about the stream is re-encoded for the browser.

const PROTOCOL_VERSION = 6;
const MAX_FRAME_BYTES = 100_000_000;

const OP_RECT = 0x01;
const OP_FILL = 0x02;
const OP_COPY = 0x03;

const K_HELLO = 0x01;
const K_REQUEST_KEYFRAME = 0x02;
const K_ACK = 0x03;
const K_SNAPSHOT_BEGIN = 0x04;
const K_SNAPSHOT_CHUNK = 0x05;
const K_PARTIAL_UPDATE = 0x06;
const K_KEEP_ALIVE = 0x07;
const K_QUALITY = 0x08;
const K_ERROR = 0x09;
const K_BYE = 0x0A;
const K_SNAPSHOT_COMMIT = 0x0B;
const K_BROWSER_OFFER = 0x22;
const K_BROWSER_REPLY = 0x23;

const FMT_RAW = 0;
const FMT_LZ4 = 1;
const FMT_PNG = 2;

class Rejected {
  constructor(reason) { this.reason = reason; }
}

class Reject extends Error {}

// ------------------------------------------------------------ webcrypto
//
// The native client uses X25519 and ChaCha20-Poly1305. A browser cannot:
// `crypto.subtle` has no X25519 in the versions most people run, but
// ECDH on P-256, HKDF and AES-GCM are everywhere. So the browser gets a
// construction built only from primitives it actually has. The protocol
// shape is identical -- ephemeral key exchange, proof of holding the
// token, then every frame sealed -- and it must agree byte for byte with
// `src/network/web_e2e.rs`.

const enc = new TextEncoder();

// SHA-256 over the literal prefix, the word "proof", the uncompressed
// point, then the token. Identical to `proof_of` on the Rust side.
const PROOF_PREFIX = enc.encode('pcc/web/v1');
const PROOF_WORD = enc.encode('proof');

// The token is part of the preimage. Without it the proof says only
// "I generated this key", which proves nothing about authorisation, and
// the sharer rejects it.
async function proofOfKey(publicKey, token) {
  return proofOfBytes(
    new Uint8Array(await crypto.subtle.exportKey('raw', publicKey)),
    token,
  );
}

async function proofOfBytes(raw, token) {
  const secret = enc.encode(token);
  let at = 0;
  const data = new Uint8Array(
    PROOF_PREFIX.length + PROOF_WORD.length + raw.length + secret.length,
  );
  for (const part of [PROOF_PREFIX, PROOF_WORD, raw, secret]) {
    data.set(part, at);
    at += part.length;
  }
  return new Uint8Array(await crypto.subtle.digest('SHA-256', data));
}

function constantTimeEqual(a, b) {
  if (a.length !== b.length) return false;
  let d = 0;
  for (let i = 0; i < a.length; i++) d |= a[i] ^ b[i];
  return d === 0;
}

// HKDF-Extract with a fixed salt, then one Expand per direction, so the
// two never share a key even though they share a secret.
async function deriveKeys(sharedSecret) {
  const material = await crypto.subtle.importKey('raw', sharedSecret, 'HKDF', false, ['deriveBits']);
  const one = async (label) => {
    const bits = await crypto.subtle.deriveBits(
      { name: 'HKDF', hash: 'SHA-256', salt: PROOF_PREFIX, info: enc.encode(label) },
      material, 256,
    );
    return crypto.subtle.importKey('raw', bits, 'AES-GCM', false, ['encrypt', 'decrypt']);
  };
  const o2r = await one('o2r');
  const r2o = await one('r2o');
  return { o2r, r2o };
}

// The counter travels in the clear and is also the AAD, so a proxy
// rewriting it fails the tag.
function nonceFor(counter) {
  const iv = new Uint8Array(12);
  new DataView(iv.buffer).setBigUint64(4, BigInt(counter), false);
  const aad = new Uint8Array(8);
  new DataView(aad.buffer).setBigUint64(0, BigInt(counter), true);
  return { iv, aad };
}

class SealedCodec {
  constructor() {
    this.keys = null;
    this.mode = null;
    this.pair = null;
    this.sendCounter = 0;
    this.receiveCounter = 0;
  }

  get ready() { return this.keys !== null; }

  // --- viewer side: offer, then complete with the sharer's reply.
  async makeOffer(token) {
    this.pair = await crypto.subtle.generateKey(
      { name: 'ECDH', namedCurve: 'P-256' }, true, ['deriveBits'],
    );
    this.token = token;
    return {
      public: new Uint8Array(await crypto.subtle.exportKey('raw', this.pair.publicKey)),
      proof: await proofOfKey(this.pair.publicKey, token),
    };
  }

  async finishHandshake(offer, reply) {
    const theirs = await crypto.subtle.importKey(
      'raw', reply.public, { name: 'ECDH', namedCurve: 'P-256' }, false, [],
    );
    const shared = await crypto.subtle.deriveBits(
      { name: 'ECDH', public: theirs }, this.pair.privateKey, 256,
    );
    this.keys = await deriveKeys(shared);
    this.mode = 'viewer';
  }

  // --- sharer side, used by the Rust server to check a browser's proof.
  async checkProof(offer, token) {
    return constantTimeEqual(await proofOfBytes(offer.public, token), offer.proof);
  }

  async sealBytes(bytes) {
    if (!this.keys) throw new Error('not sealed yet');
    const counter = this.sendCounter++;
    const { iv, aad } = nonceFor(counter);
    const key = this.mode === 'sharer' ? this.keys.r2o : this.keys.o2r;
    const ct = new Uint8Array(await crypto.subtle.encrypt(
      { name: 'AES-GCM', iv, additionalData: aad, tagLength: 128 }, key, bytes,
    ));
    const out = new Uint8Array(4 + ct.length);
    new DataView(out.buffer).setUint32(0, counter, true);
    out.set(ct, 4);
    return out;
  }

  async openBytes(frame) {
    if (!this.keys) throw new Error('not sealed yet');
    const counter = new DataView(frame.buffer, frame.byteOffset, 4).getUint32(0, true);
    if (counter < this.receiveCounter) throw new Reject('sealed frame is a replay');
    this.receiveCounter = counter + 1;
    const { iv, aad } = nonceFor(counter);
    const key = this.mode === 'sharer' ? this.keys.o2r : this.keys.r2o;
    const plain = await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv, additionalData: aad, tagLength: 128 }, key, frame.slice(4),
    );
    return new Uint8Array(plain);
  }
}

// ---------------------------------------------------------------- reader

class Reader {
  constructor(view, offset = 0) {
    // A DataView for little-endian reads and a Uint8Array over the same
    // bytes for slicing: they are different objects and only one of them
    // can be sliced.
    this.view = view;
    this.bytes = new Uint8Array(view.buffer, view.byteOffset, view.byteLength);
    this.offset = offset;
  }
  get remaining() { return this.view.byteLength - this.offset; }
  need(n) {
    if (n > this.remaining) {
      throw new Reject(`truncated message: need ${n} more bytes, have ${this.remaining}`);
    }
  }
  u8() { this.need(1); return this.view.getUint8(this.offset++); }
  u16() { this.need(2); const v = this.view.getUint16(this.offset, true); this.offset += 2; return v; }
  u32() { this.need(4); const v = this.view.getUint32(this.offset, true); this.offset += 4; return v; }
  u64() {
    this.need(8);
    const v = this.view.getBigUint64(this.offset, true);
    this.offset += 8;
    return Number(v);
  }
  take(n) { this.need(n); const v = this.bytes.subarray(this.offset, this.offset + n); this.offset += n; return v; }
  allocLen(max, what) {
    const n = this.u32();
    if (n > max) throw new Reject(`${what} too large: ${n} (max ${max})`);
    return n;
  }
}

// ------------------------------------------------------------- lz4 block

// LZ4 block format decoder. Small on purpose: the format is a handful of
// opcodes and pulling in a library for it would be larger than this file.
function lz4Decode(src, expected) {
  const dst = new Uint8Array(expected);
  let s = 0, d = 0;
  while (s < src.length) {
    const token = src[s++];
    let literal = token >> 4;
    if (literal === 15) {
      let n;
      do { n = src[s++]; literal += n; } while (n === 255);
    }
    if (s + literal > src.length) throw new Reject('lz4: literal run runs past the input');
    if (d + literal > dst.length) throw new Reject('lz4: output overflow while copying literals');
    dst.set(src.subarray(s, s + literal), d);
    s += literal; d += literal;
    if (s >= src.length) break; // last sequence has literals only
    const offset = src[s] | (src[s + 1] << 8);
    s += 2;
    if (offset === 0 || offset > d) throw new Reject('lz4: invalid match offset');
    let match = token & 0x0F;
    if (match === 15) {
      let n;
      do { n = src[s++]; match += n; } while (n === 255);
    }
    match += 4;
    if (d + match > dst.length) throw new Reject('lz4: output overflow while copying a match');
    // Overlapping copies are legal and common: copy byte by byte.
    let from = d - offset;
    for (let i = 0; i < match; i++) dst[d++] = dst[from++];
  }
  if (d !== expected) throw new Reject(`lz4: decoded to ${d} bytes, expected ${expected}`);
  return dst;
}

function lz4DecodeSizePrepended(src, expected) {
  if (src.length < 4) throw new Reject('lz4: payload too short');
  const size = new DataView(src.buffer, src.byteOffset, 4).getUint32(0, true);
  if (size !== expected) {
    throw new Reject(`lz4: declared ${size} bytes, geometry requires ${expected}`);
  }
  return lz4Decode(src.subarray(4), expected);
}

// ------------------------------------------------------------ compositor

class Compositor {
  constructor() {
    this.buffer = null;
    this.width = 0;
    this.height = 0;
    this.epoch = 0;
    this.rev = 0;
    this.fresh = false;
    this.incoming = null;
    this.onPaint = null;
  }

  surface() {
    if (this.buffer) return this.buffer;
    const scratch = document.createElement('canvas');
    scratch.width = 1;
    scratch.height = 1;
    return scratch.getContext('2d').createImageData(1, 1).data;
  }

  beginSnapshot(epoch, width, height, format, totalLen, chunks) {
    if (this.fresh && epoch < this.epoch) {
      throw new Reject(`snapshot epoch ${epoch} is older than the installed epoch ${this.epoch}`);
    }
    if (totalLen === 0) throw new Reject('snapshot announced with total_len=0');
    this.incoming = {
      epoch, width, height, format, totalLen,
      expected: chunks,
      parts: new Array(chunks),
      received: 0,
    };
  }

  pushSnapshotChunk(index, data) {
    const inc = this.incoming;
    if (!inc) throw new Reject(`snapshot chunk ${index} with no snapshot in progress`);
    if (index >= inc.expected) throw new Reject(`snapshot chunk index ${index} outside 0..${inc.expected}`);
    if (inc.parts[index] !== undefined) throw new Reject(`snapshot chunk ${index} arrived twice`);
    inc.parts[index] = data;
    inc.received += data.length;
    if (inc.received > inc.totalLen) {
      throw new Reject(`snapshot overflows: ${inc.received} > ${inc.totalLen}`);
    }
  }

  async commitSnapshot(rev, epoch) {
    const inc = this.incoming;
    if (!inc) throw new Reject('snapshot commit with no snapshot in progress');
    this.incoming = null;
    if (inc.epoch !== epoch) {
      throw new Reject(`snapshot commit epoch ${epoch} does not match the transfer's ${inc.epoch}`);
    }
    const missing = inc.parts.findIndex((p) => p === undefined);
    if (missing >= 0) throw new Reject(`snapshot committed with chunk ${missing} missing`);
    if (inc.received !== inc.totalLen) {
      throw new Reject(`snapshot is ${inc.received} bytes, expected ${inc.totalLen}`);
    }

    let surface;
    if (inc.format === FMT_RAW) {
      // Already four bytes per pixel in the widest path the sharer uses.
      surface = new Uint8ClampedArray(inc.width * inc.height * STRIDE);
      surface.set(inc.parts[0].subarray(0, surface.length));
    } else if (inc.format === FMT_LZ4) {
      surface = widenRgb(
        lz4DecodeSizePrepended(join(inc.parts, inc.totalLen), inc.width * inc.height * 3),
        inc.width * inc.height,
      );
    } else {
      const blob = new Blob(inc.parts, { type: 'image/png' });
      const bitmap = await createImageBitmap(blob);
      if (bitmap.width !== inc.width || bitmap.height !== inc.height) {
        throw new Reject(`snapshot declares ${inc.width}x${inc.height} but decoded ${bitmap.width}x${bitmap.height}`);
      }
      const off = new OffscreenCanvas(inc.width, inc.height);
      const ctx = off.getContext('2d');
      ctx.drawImage(bitmap, 0, 0);
      surface = ctx.getImageData(0, 0, inc.width, inc.height).data;
      bitmap.close();
    }
    if (surface.length !== inc.width * inc.height * STRIDE) {
      throw new Reject(`snapshot decoded to ${surface.length} bytes, expected ${surface.length === inc.width * inc.height * 3 ? 'an RGB body' : inc.width * inc.height * STRIDE}`);
    }
    const rgb = surface;

    // Nothing above this line touched `this`, so a failed commit leaves
    // the displayed surface exactly as it was.
    this.buffer = rgb;
    this.width = inc.width;
    this.height = inc.height;
    this.epoch = inc.epoch;
    this.rev = rev;
    this.fresh = true;
    this.paint();
  }

  applyOps(rev, epoch, ops) {
    if (!this.fresh) throw new Rejected('needs-snapshot');
    if (epoch < this.epoch) throw new Rejected('stale-epoch');
    if (epoch > this.epoch) throw new Rejected('needs-snapshot');
    if (rev <= this.rev) throw new Rejected('already-applied');

    // Validate and materialise everything first.
    const staged = [];
    for (const op of ops) {
      if (op.w === 0 || op.h === 0) throw new Reject(`empty op ${op.w}x${op.h}`);
      if (op.x + op.w > this.width || op.y + op.h > this.height) {
        throw new Reject(`op (${op.x},${op.y}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);
      }
      if (op.kind === OP_COPY
          && (op.srcX + op.w > this.width || op.srcY + op.h > this.height)) {
        throw new Reject(`copy source (${op.srcX},${op.srcY}) ${op.w}x${op.h} exceeds frame ${this.width}x${this.height}`);
      }
      if (op.kind === OP_RECT) {
        staged.push({ ...op, pixels: lz4DecodeSizePrepended(op.payload, op.w * op.h * 3) });
      } else {
        staged.push(op);
      }
    }

    const needsSnapshot = staged.some((o) => o.kind === OP_COPY);
    const base = needsSnapshot ? this.buffer.slice() : null;

    for (const op of staged) {
      // The wire carries RGB; the surface is RGBA, so widen on the way in.
      if (op.kind === OP_RECT) {
        blit(this.buffer, this.width, op.x, op.y, op.w, op.h, widenRgb(op.pixels, op.w * op.h));
      }
      else if (op.kind === OP_FILL) fill(this.buffer, this.width, op.x, op.y, op.w, op.h, op.color);
      else blitRegion(this.buffer, this.width, op.x, op.y, op.w, op.h, base, op.srcX, op.srcY);
    }
    this.rev = rev;
    this.paint();
  }

  paint() {
    if (this.onPaint) this.onPaint(this);
  }
}

function join(parts, totalLen) {
  const out = new Uint8Array(totalLen);
  let at = 0;
  for (const p of parts) { out.set(p, at); at += p.length; }
  return out;
}

// The surface is RGBA, four bytes per pixel. It has to be: the paint
// path hands this buffer straight to `new ImageData`, which is RGBA and
// nothing else. It used to be RGB here and RGBA after a PNG snapshot, so
// the two disagreed and every patch applied to a PNG-seeded surface
// landed on the wrong pixels. The wire format is still RGB; the widening
// happens where a snapshot enters the surface.
const STRIDE = 4;

/** Widen three-byte RGB into the four-byte surface the compositor holds. */
function widenRgb(rgb, pixels) {
  const out = new Uint8ClampedArray(pixels * STRIDE);
  for (let i = 0, j = 0; j < out.length; i += 3, j += STRIDE) {
    out[j] = rgb[i];
    out[j + 1] = rgb[i + 1];
    out[j + 2] = rgb[i + 2];
    out[j + 3] = 255;
  }
  return out;
}

function blit(dst, dstWidth, x, y, w, h, src) {
  const rowBytes = w * STRIDE;
  for (let row = 0; row < h; row++) {
    dst.set(src.subarray(row * rowBytes, (row + 1) * rowBytes), ((y + row) * dstWidth + x) * STRIDE);
  }
}

function blitRegion(dst, dstWidth, x, y, w, h, src, sx, sy) {
  const rowBytes = w * STRIDE;
  for (let row = 0; row < h; row++) {
    const to = ((y + row) * dstWidth + x) * STRIDE;
    const from = ((sy + row) * dstWidth + sx) * STRIDE;
    dst.set(src.subarray(from, from + rowBytes), to);
  }
}

function fill(dst, dstWidth, x, y, w, h, color) {
  const row = new Uint8ClampedArray(w * STRIDE);
  for (let i = 0; i < w; i++) {
    row[i * STRIDE] = color[0];
    row[i * STRIDE + 1] = color[1];
    row[i * STRIDE + 2] = color[2];
    row[i * STRIDE + 3] = 255;
  }
  for (let r = 0; r < h; r++) {
    dst.set(row, ((y + r) * dstWidth + x) * STRIDE);
  }
}

// ----------------------------------------------------------- message read

function parseMessage(bytes) {
  const buf = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes);
  if (buf.length < 5) throw new Reject(`message too short: ${buf.length} bytes`);
  const view = new DataView(buf.buffer, buf.byteOffset, buf.byteLength);
  if (view.getUint8(0) !== PROTOCOL_VERSION) {
    throw new Reject(`protocol version mismatch: expected ${PROTOCOL_VERSION}, got ${view.getUint8(0)}`);
  }
  const len = view.getUint32(1, true);
  if (len !== buf.length - 5) throw new Reject(`message length mismatch: header says ${len}`);
  const r = new Reader(view, 5);
  const kind = r.u8();
  switch (kind) {
    case K_SNAPSHOT_BEGIN:
      return {
        kind,
        rev: r.u64(),
        ptsUs: r.u64(),
        epoch: r.u32(),
        width: r.u32(),
        height: r.u32(),
        format: r.u8(),
        totalLen: r.allocLen(MAX_FRAME_BYTES + 65536, 'snapshot'),
        chunks: r.u32(),
      };
    case K_SNAPSHOT_CHUNK: {
      const rev = r.u64();
      const index = r.u32();
      const n = r.allocLen(1024 * 1024, 'snapshot chunk');
      return { kind, rev, index, data: r.take(n).slice() };
    }
    case K_SNAPSHOT_COMMIT:
      return { kind, rev: r.u64(), ptsUs: r.u64(), epoch: r.u32() };
    case K_PARTIAL_UPDATE: {
      const rev = r.u64();
      const ptsUs = r.u64();
      const epoch = r.u32();
      const count = r.u32();
      if (count > 8192) throw new Reject(`too many ops in one update: ${count}`);
      const ops = [];
      for (let i = 0; i < count; i++) {
        const x = r.u32(), y = r.u32(), w = r.u32(), h = r.u32();
        const opKind = r.u8();
        if (opKind === OP_RECT) {
          const n = r.allocLen(w * h * 3 + 16, 'rect payload');
          ops.push({ kind: OP_RECT, x, y, w, h, payload: r.take(n).slice() });
        } else if (opKind === OP_FILL) {
          const n = r.allocLen(3, 'fill colour');
          ops.push({ kind: OP_FILL, x, y, w, h, color: Array.from(r.take(n)) });
        } else if (opKind === OP_COPY) {
          const srcX = r.u32();
          const srcY = r.u32();
          ops.push({ kind: OP_COPY, x, y, w, h, srcX, srcY });
        } else {
          throw new Reject(`unknown op kind: 0x${opKind.toString(16)}`);
        }
      }
      return { kind, rev, ptsUs, epoch, ops };
    }
    case K_KEEP_ALIVE:
      return { kind, rev: r.u64() };
    case K_QUALITY:
      return { kind, targetFps: r.u32(), maxFps: r.u32(), quality: r.view.getFloat32(r.offset, true) };
    case K_ERROR: {
      const n = r.allocLen(1024, 'Error message');
      return { kind, text: new TextDecoder().decode(r.take(n)) };
    }
    case K_BYE:
      return { kind };
    default:
      throw new Reject(`unknown message kind: 0x${kind.toString(16)}`);
  }
}

function encodeHello(token, resume) {
  const raw = new TextEncoder().encode(token);
  const tail = resume ? 1 + 4 + 8 : 1;
  const out = new Uint8Array(5 + 1 + 2 + raw.length + tail);
  const view = new DataView(out.buffer);
  const bodyAt = 5;
  out[0] = PROTOCOL_VERSION;
  view.setUint32(1, 1 + 2 + raw.length + tail, true);
  out[bodyAt] = K_HELLO;
  view.setUint16(bodyAt + 1, raw.length, true);
  out.set(raw, bodyAt + 3);
  let at = bodyAt + 3 + raw.length;
  if (resume) {
    out[at++] = 1;
    view.setUint32(at, resume.epoch, true); at += 4;
    view.setBigUint64(at, BigInt(resume.rev), true);
  } else {
    out[at] = 0;
  }
  return out;
}

// ------------------------------------------------------------- transport

class Session {
  constructor(token) {
    this.token = token;
    this.compositor = new Compositor();
    this.socket = null;
    this.lastAck = 0;
    this.paintScheduled = false;
    this.setupCanvas();
  }

  setupCanvas() {
    this.canvas = document.getElementById('screen');
    this.ctx = this.canvas.getContext('2d');
    this.compositor.onPaint = () => this.schedulePaint();
  }

  schedulePaint() {
    if (this.paintScheduled) return;
    this.paintScheduled = true;
    requestAnimationFrame(() => {
      this.paintScheduled = false;
      this.paint();
    });
  }

  paint() {
    const c = this.compositor;
    if (!c.fresh) return;
    if (this.canvas.width !== c.width || this.canvas.height !== c.height) {
      this.canvas.width = c.width;
      this.canvas.height = c.height;
    }
    this.ctx.putImageData(new ImageData(new Uint8ClampedArray(c.buffer), c.width, c.height), 0, 0);
    if (c.rev !== this.lastAck) {
      this.lastAck = c.rev;
      // Fire and forget: a failed acknowledgement must not interrupt
      // painting the frame that produced it.
      this.sendAck(c.rev).catch(() => {});
    }
  }

  status(text) {
    const el = document.getElementById('status');
    if (el) el.textContent = text;
  }

  async connect() {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const url = `${proto}://${location.host}/ws?token=${encodeURIComponent(this.token)}`;
    const socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';
    this.socket = socket;
    this.codec = new SealedCodec();

    socket.onopen = async () => {
      this.status('handshaking');
      try {
        // The offer goes before anything else, and nothing else flows
        // until the session exists: there is no plaintext fallback,
        // because serving one quietly would defeat the point.
        const offer = await this.codec.makeOffer(this.token);
        const framed = new Uint8Array(1 + offer.public.length + offer.proof.length);
        framed[0] = K_BROWSER_OFFER;
        framed.set(offer.public, 1);
        framed.set(offer.proof, 1 + offer.public.length);
        socket.send(framed);

        // Routed through the single onmessage path below rather than a
        // second listener: a WebSocket delivers every frame to both, so a
        // listener here would also see this frame, and onmessage would
        // try to open the *reply* as sealed surface data and kill the
        // session.
        this.pendingReply = new Promise((resolve, reject) => {
          this.resolveReply = resolve;
          this.rejectReply = reject;
        });
        const replyFrame = await this.pendingReply;
        const cur = new Reader(new DataView(
          replyFrame.buffer, replyFrame.byteOffset, replyFrame.byteLength), 0);
        if (cur.u8() !== K_BROWSER_REPLY) throw new Error('expected an encryption reply');
        const theirPublic = cur.take(65).slice();
        const theirProof = cur.take(32).slice();
        await this.codec.finishHandshake(
          { public: offer.public, proof: offer.proof },
          { public: theirPublic, proof: theirProof },
        );
        this.status('sealed');
        // The sharer sends its snapshot as soon as the session exists, so
        // there is nothing to request; wait for the first frame.
      } catch (e) {
        this.status(`handshake failed: ${e.message}`);
        socket.close();
      }
    };
    // onerror is called with an Event, not a message; the detail lives on
    // the event, so read it from there rather than treating the argument
    // as a string.
    socket.onerror = (event) => {
      this.status(`connection error: ${(event && event.message) || 'unknown'}`);
    };
    socket.onclose = (event) => {
      this.status(`disconnected (code ${event.code}); retrying in 2s`);
      setTimeout(() => this.connect(), 2000);
    };
    // Frames are handled strictly in order, one at a time. An `async`
    // handler is not awaited by the WebSocket, so a sealed frame could
    // otherwise be opened while the handshake that precedes it is still
    // deriving keys -- and fail with "not sealed yet" on a session that
    // was about to be fine.
    let queue = Promise.resolve();
    socket.onmessage = (event) => {
      queue = queue.then(() => this.handleFrame(event, socket)).catch(() => {});
    };
  }

  // Handle one inbound frame. Called from a serial queue, never directly
  // from the socket.
  async handleFrame(event, socket) {
    const bytes = new Uint8Array(event.data);
    // The handshake reply is a plain message, not sealed surface data.
    // It has to come through here: a WebSocket delivers every frame to
    // every listener, so a second one used for the reply would also see
    // it, and this handler would try to open it as surface data and kill
    // the session.
    if (this.resolveReply) {
      const resolve = this.resolveReply;
      this.resolveReply = null;
      resolve(bytes);
      return;
    }
    try {
      await this.onMessage(await this.codec.openBytes(bytes));
    } catch (e) {
      this.status(`could not open a frame: ${e.message}`);
      socket.close();
    }
  }

  // Every frame the browser sends is sealed, the acknowledgements
  // included. Sending one in the clear is a 14-byte frame the sharer
  // cannot open, and it drops the session.
  async sendAck(rev) {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) return;
    if (!this.codec || !this.codec.ready) return;
    this.socket.send(await this.codec.sealBytes(encodeAck(rev)));
  }

  async onMessage(bytes) {
    let msg;
    try {
      msg = parseMessage(bytes);
    } catch (e) {
      this.status(`rejected: ${e.message}`);
      return;
    }
    const c = this.compositor;
    try {
      switch (msg.kind) {
        case K_SNAPSHOT_BEGIN:
          c.beginSnapshot(msg.epoch, msg.width, msg.height, msg.format, msg.totalLen, msg.chunks);
          break;
        case K_SNAPSHOT_CHUNK:
          c.pushSnapshotChunk(msg.index, msg.data);
          break;
        case K_SNAPSHOT_COMMIT:
          await c.commitSnapshot(msg.rev, msg.epoch);
          this.lastPtsUs = msg.ptsUs;
          this.status(`${c.width}x${c.height} rev ${c.rev}`);
          break;
        case K_PARTIAL_UPDATE:
          c.applyOps(msg.rev, msg.epoch, msg.ops);
          this.lastPtsUs = msg.ptsUs;
          break;
        case K_KEEP_ALIVE:
        case K_QUALITY:
          break;
        case K_ERROR:
          this.status(msg.text);
          break;
        case K_BYE:
          this.status('the sharer ended the session');
          this.socket.close();
          break;
        default:
          break;
      }
    } catch (e) {
      if (e instanceof Rejected) {
        // A revision we already hold is the expected consequence of
        // joining mid-stream: the join snapshot covers it. Only a missing
        // base is worth repairing.
        if (e.reason === 'needs-snapshot' || e.reason === 'stale-epoch') {
          this.status(`repairing: ${e.reason}`);
          this.requestKeyframe().catch(() => {});
        }
      } else {
        this.status(`invalid update: ${e.message}`);
        this.requestKeyframe().catch(() => {});
      }
    }
  }

  // Sealed like every other frame the browser sends. In the clear this
  // was a 9-byte frame the sharer could not open, and it dropped the
  // session exactly when a viewer most needed to recover.
  async requestKeyframe() {
    if (!this.socket || this.socket.readyState !== WebSocket.OPEN) return;
    if (!this.codec || !this.codec.ready) return;
    this.socket.send(await this.codec.sealBytes(encodeRequestKeyframe()));
  }
}

function envelope(body) {
  const out = new Uint8Array(5 + body.length);
  out[0] = PROTOCOL_VERSION;
  new DataView(out.buffer).setUint32(1, body.length, true);
  out.set(body, 5);
  return out;
}

function encodeAck(rev) {
  const body = new Uint8Array(9);
  const view = new DataView(body.buffer);
  body[0] = K_ACK;
  view.setBigUint64(1, BigInt(rev), true);
  return envelope(body);
}

function encodeRequestKeyframe() {
  return envelope(new Uint8Array([K_REQUEST_KEYFRAME]));
}

window.pcc = {
  connect(token) {
    const session = new Session(token);
    session.connect();
    return session;
  },
  // Exposed so the same parser and the same LZ4 decoder can be exercised
  // by a test page without a live sharer.
  parseMessage,
  lz4Decode,
  Compositor,
  PROTOCOL_VERSION,
  // The handshake code, so a test can drive the *shipped* implementation
  // against the server rather than a reimplementation of it. A
  // reimplementation proved nothing: it agreed with the server while the
  // real file did not, because the real file dropped the token from the
  // proof preimage and no test ever ran it.
  SealedCodec,
  Session,
  proofOfBytes,
};
