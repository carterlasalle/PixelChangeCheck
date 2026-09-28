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

const PROTOCOL_VERSION = 3;
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

const FMT_RAW = 0;
const FMT_LZ4 = 1;
const FMT_PNG = 2;

class Rejected {
  constructor(reason) { this.reason = reason; }
}

class Reject extends Error {}

// ---------------------------------------------------------------- reader

class Reader {
  constructor(view, offset = 0) {
    this.view = view;
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
  take(n) { this.need(n); const v = this.view.subarray(this.offset, this.offset + n); this.offset += n; return v; }
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

    let rgb;
    if (inc.format === FMT_RAW) {
      rgb = new Uint8ClampedArray(inc.totalLen);
      rgb.set(inc.parts[0]);
    } else if (inc.format === FMT_LZ4) {
      rgb = lz4DecodeSizePrepended(join(inc.parts, inc.totalLen), inc.width * inc.height * 3);
    } else {
      const blob = new Blob(inc.parts, { type: 'image/png' });
      const bitmap = await createImageBitmap(blob);
      if (bitmap.width !== inc.width || bitmap.height !== inc.height) {
        throw new Reject(`snapshot declares ${inc.width}x${inc.height} but decoded ${bitmap.width}x${bitmap.height}`);
      }
      const off = new OffscreenCanvas(inc.width, inc.height);
      const ctx = off.getContext('2d');
      ctx.drawImage(bitmap, 0, 0);
      rgb = ctx.getImageData(0, 0, inc.width, inc.height).data;
      bitmap.close();
      if (rgb.length > MAX_FRAME_BYTES) throw new Reject('decoded snapshot exceeds the frame budget');
    }

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
      if (op.kind === OP_RECT) blit(this.buffer, this.width, op.x, op.y, op.w, op.h, op.pixels);
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

function blit(dst, dstWidth, x, y, w, h, src) {
  const rowBytes = w * 3;
  for (let row = 0; row < h; row++) {
    dst.set(src.subarray(row * rowBytes, (row + 1) * rowBytes), ((y + row) * dstWidth + x) * 3);
  }
}

function blitRegion(dst, dstWidth, x, y, w, h, src, sx, sy) {
  const rowBytes = w * 3;
  for (let row = 0; row < h; row++) {
    const to = ((y + row) * dstWidth + x) * 3;
    const from = ((sy + row) * dstWidth + sx) * 3;
    dst.set(src.subarray(from, from + rowBytes), to);
  }
}

function fill(dst, dstWidth, x, y, w, h, color) {
  const row = new Uint8ClampedArray(w * 3);
  for (let i = 0; i < w; i++) { row[i * 3] = color[0]; row[i * 3 + 1] = color[1]; row[i * 3 + 2] = color[2]; }
  for (let r = 0; r < h; r++) {
    dst.set(row, ((y + r) * dstWidth + x) * 3);
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
      return { kind, rev: r.u64(), epoch: r.u32() };
    case K_PARTIAL_UPDATE: {
      const rev = r.u64();
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
      return { kind, rev, epoch, ops };
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

function encodeHello(token) {
  const raw = new TextEncoder().encode(token);
  const out = new Uint8Array(5 + 1 + 2 + raw.length);
  const view = new DataView(out.buffer);
  const bodyAt = 5;
  out[0] = PROTOCOL_VERSION;
  view.setUint32(1, 1 + 2 + raw.length, true);
  out[bodyAt] = K_HELLO;
  view.setUint16(bodyAt + 1, raw.length, true);
  out.set(raw, bodyAt + 3);
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
      this.sendAck(c.rev);
    }
  }

  status(text) {
    const el = document.getElementById('status');
    if (el) el.textContent = text;
  }

  connect() {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws';
    const url = `${proto}://${location.host}/ws?token=${encodeURIComponent(this.token)}`;
    const socket = new WebSocket(url);
    socket.binaryType = 'arraybuffer';
    this.socket = socket;

    socket.onopen = () => {
      this.status('connected');
      socket.send(encodeHello(this.token));
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
    socket.onmessage = (event) => this.onMessage(new Uint8Array(event.data));
  }

  sendAck(rev) {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(encodeAck(rev));
    }
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
          this.status(`${c.width}x${c.height} rev ${c.rev}`);
          break;
        case K_PARTIAL_UPDATE:
          c.applyOps(msg.rev, msg.epoch, msg.ops);
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
        // The state machine tells us exactly what is missing; ask for it
        // rather than guessing.
        this.status(`repairing: ${e.reason}`);
        this.requestKeyframe();
      } else {
        this.status(`invalid update: ${e.message}`);
        this.requestKeyframe();
      }
    }
  }

  requestKeyframe() {
    if (this.socket && this.socket.readyState === WebSocket.OPEN) {
      this.socket.send(encodeRequestKeyframe());
    }
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
};
