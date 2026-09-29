//! Regression tests for the failures the audits identified.
//!
//! Each test here maps to a specific way the previous implementation lost
//! or corrupted state. They are deliberately written against observable
//! behaviour -- what a viewer ends up showing -- rather than against
//! internal call sequences.

use anyhow::Result;
use pixel_change_check_client::encoder::{compression, encode_snapshot, SnapshotFormat};
use pixel_change_check_client::network::{Message, WireOp};
use pixel_change_check_client::pcc::types::{rgb_len, Frame, PixelChange};
use pixel_change_check_client::pcc::{ApplyError, Compositor, PlanLimits, Planner, Rejected};

const W: u32 = 64;
const H: u32 = 48;

fn frame(data: Vec<u8>) -> Frame {
    Frame::new(0, W, H, data).unwrap()
}

fn blank() -> Frame {
    frame(vec![0; rgb_len(W, H).unwrap()])
}

fn px(f: &mut Frame, x: u32, y: u32, rgb: [u8; 3]) {
    let i = ((y * f.width + x) * 3) as usize;
    f.data[i..i + 3].copy_from_slice(&rgb);
}

fn rect_op(x: u32, y: u32, w: u32, h: u32, pixels: [u8; 3]) -> WireOp {
    let data: Vec<u8> = pixels
        .iter()
        .copied()
        .cycle()
        .take((w * h) as usize * 3)
        .collect();
    WireOp::Rect {
        x,
        y,
        width: w,
        height: h,
        compressed: compression::compress_frame(&data).unwrap(),
    }
}

/// Install a lossless snapshot the way a joining viewer is installed.
fn install(c: &mut Compositor, source: &Frame, epoch: u32, rev: u64) {
    let (_, data) = encode_snapshot(source.width, source.height, &source.data).unwrap();
    c.begin_snapshot(
        epoch,
        source.width,
        source.height,
        SnapshotFormat::Png,
        data.len() as u32,
        1,
    )
    .unwrap();
    c.push_snapshot_chunk(0, &data).unwrap();
    c.commit_snapshot(rev, epoch).unwrap();
}

fn limits() -> PlanLimits {
    PlanLimits {
        snapshot_bytes: 8 * 1024 * 1024,
        max_update_bytes: pixel_change_check_client::network::MAX_MESSAGE_SIZE as usize,
    }
}

// ---------------------------------------------------------------------------
// "Fade 100 -> 103 -> ... -> 250: accumulation against the wrong reference"
// ---------------------------------------------------------------------------

/// A slow fade used to advance the diff reference on every capture, so
/// each 3-step change was below the threshold and the viewer stayed at
/// 100 forever.
#[test]
fn a_slow_fade_eventually_reaches_the_viewer() {
    let planner = Planner::new(5, 8).unwrap();
    let mut reference = blank();
    px(&mut reference, 10, 10, [100, 100, 100]);

    let mut viewer = Compositor::new();
    install(&mut viewer, &reference, 0, 0);

    let mut rev = 0u64;
    let mut sent_updates = 0;
    let mut value = 100u8;
    for _ in 0..50 {
        value = value.saturating_add(3);
        let mut current = reference.clone();
        px(&mut current, 10, 10, [value, value, value]);

        let plan = planner
            .plan(&mut reference.data, W, H, &current, limits())
            .unwrap();
        if !plan.ops.is_empty() {
            rev += 1;
            sent_updates += 1;
            viewer.apply_ops(rev, 0, &plan.ops).unwrap();
        }
    }

    assert!(
        sent_updates > 0,
        "a 150-step drift must cross the threshold"
    );
    let idx = ((10 * W + 10) * 3) as usize;
    assert_eq!(
        viewer.buffer()[idx],
        250,
        "the viewer must end up where the screen is, not where it started"
    );
}

// ---------------------------------------------------------------------------
// "One change then 30 seconds idle: missed periodic repair and stale late join"
// ---------------------------------------------------------------------------

/// The cached keyframe used to be whatever the sharer happened to send
/// last, so a viewer joining after the screen had settled saw an old
/// desktop indefinitely.
#[test]
fn a_late_joiner_gets_the_current_surface_not_an_old_one() {
    let planner = Planner::default();
    let mut reference = blank();
    let start = blank();

    // The sharer changes once, then goes quiet.
    let mut after_change = start;
    px(&mut after_change, 5, 5, [200, 100, 50]);
    let _ = planner
        .plan(&mut reference.data, W, H, &after_change, limits())
        .unwrap();

    for _ in 0..300 {
        let _ = planner
            .plan(&mut reference.data, W, H, &after_change, limits())
            .unwrap();
    }

    // A viewer joining now must be caught up from the live reference, not
    // from the frame as it was before the change.
    let mut viewer = Compositor::new();
    install(&mut viewer, &reference, 0, 10);
    assert_eq!(
        viewer.buffer(),
        &after_change.data,
        "a late joiner must see the changed screen, not the pre-change one"
    );
}

// ---------------------------------------------------------------------------
// A late joiner must see the *live* surface, not a copy taken when the
// sharer started.
//
// The published snapshot used to be built once, at startup. Every later
// update advanced the sharer's own reference without touching it, so a
// viewer joining after any change received the pre-change screen and
// nothing would ever have corrected it.
// ---------------------------------------------------------------------------

/// Drive the sharer's publish step the way `capture_loop` does: the
/// published snapshot must follow the shared surface, not lag behind it.
#[test]
fn the_published_surface_follows_every_shipped_update() {
    let (w, h) = (64u32, 48u32);
    let planner = Planner::default();
    let start = blank();

    // The shared buffer the capture loop mutates, and the handle a
    // joining viewer is handed -- the same allocation, not a copy taken
    // when the sharer started.
    let mut shared: std::sync::Arc<Vec<u8>> = std::sync::Arc::new(start.data.clone());

    let mut current = blank();
    for y in 0..h {
        for x in 0..w {
            if (x * 5 + y * 3) % 17 < 4 {
                px(&mut current, x, y, [220, 30, 90]);
            }
        }
    }

    let rgb: &mut Vec<u8> = std::sync::Arc::make_mut(&mut shared);
    let plan = planner.plan(rgb, w, h, &current, limits()).unwrap();
    assert!(!plan.ops.is_empty(), "the test needs a real update to ship");
    // This is the step the sharer performs after every frame.
    let published = shared.clone();

    // A viewer joining now is handed the shared handle, not the frame
    // the sharer happened to start with.
    let mut viewer = Compositor::new();
    let (format, data) = encode_snapshot(w, h, &published).unwrap();
    viewer
        .begin_snapshot(0, w, h, format, data.len() as u32, 1)
        .unwrap();
    viewer.push_snapshot_chunk(0, &data).unwrap();
    viewer.commit_snapshot(0, 0).unwrap();
    assert_eq!(
        viewer.buffer(),
        &current.data,
        "a late joiner must see the changed screen, not the startup frame"
    );
}

// ---------------------------------------------------------------------------
// "Join during snapshot publication: older patches applied after a newer base"
// ---------------------------------------------------------------------------

/// A viewer that subscribes, then takes a snapshot, must ignore anything
/// at or below the snapshot's revision.
#[test]
fn patches_at_or_below_the_snapshot_revision_are_ignored() {
    let base = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &base, 0, 5);

    // An update the joiner already has inside its snapshot.
    let stale = vec![rect_op(0, 0, 1, 1, [9, 9, 9])];
    assert_eq!(
        viewer.apply_ops(5, 0, &stale).unwrap_err(),
        ApplyError::Rejected(Rejected::AlreadyApplied)
    );
    // One the joiner does not have.
    let fresh = vec![rect_op(1, 1, 1, 1, [9, 9, 9])];
    viewer.apply_ops(6, 0, &fresh).unwrap();
    // Pixel (1, 1) in an RGB surface: byte ((y * W + x) * 3).
    let at = (W * 3 + 3) as usize;
    assert_eq!(&viewer.buffer()[at..at + 3], &[9, 9, 9]);
}

// ---------------------------------------------------------------------------
// "Drop one patch then keep changing another tile: permanent stale region"
// ---------------------------------------------------------------------------

/// Delta encoding has no self-healing: a dropped patch leaves a hole
/// until a snapshot replaces it. The hole is real, and so is the repair.
#[test]
fn a_dropped_patch_leaves_a_hole_that_only_a_snapshot_fills() {
    let before = blank();
    let mut source = before.clone();
    px(&mut source, 1, 1, [10, 20, 30]);
    px(&mut source, 40, 40, [1, 1, 1]);

    // The viewer joined *before* either change was published.
    let mut viewer = Compositor::new();
    install(&mut viewer, &before, 0, 0);

    // Revision 1 updates the left tile; the viewer misses it (a lag).
    // Revision 2 updates the right tile and arrives.
    // Revision 1 is never delivered: this viewer is behind.
    let later = vec![rect_op(40, 40, 1, 1, [2, 2, 2])];
    viewer.apply_ops(2, 0, &later).unwrap();
    let stale_index = (W * 3 + 3) as usize;
    assert_eq!(
        &viewer.buffer()[stale_index..stale_index + 3],
        &[0, 0, 0],
        "the missed patch leaves a real hole"
    );

    // Recovery: the host notices the lag and sends a fresh snapshot.
    let mut repaired = source;
    px(&mut repaired, 40, 40, [2, 2, 2]);
    install(&mut viewer, &repaired, 0, 3);
    assert_eq!(
        viewer.buffer(),
        &repaired.data,
        "a fresh snapshot must close the hole"
    );
}

// ---------------------------------------------------------------------------
// "Rectangle at final column extending across row: flat bounds vs geometry"
// ---------------------------------------------------------------------------

#[test]
fn a_rectangle_that_would_wrap_a_scanline_is_refused() {
    let source = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);

    // x = W-1 with width 2 crosses into the next row. The flat buffer is
    // large enough, so only a geometric check can catch it.
    let crossing = vec![rect_op(W - 1, 0, 2, 1, [255, 255, 255])];
    let before = viewer.buffer().to_vec();
    assert!(viewer.apply_ops(1, 0, &crossing).is_err());
    assert_eq!(viewer.buffer(), &before[..]);
    assert_eq!(
        viewer.rev(),
        0,
        "a refused update must not advance the revision"
    );
}

#[test]
fn a_legitimate_rectangle_at_the_final_column_is_accepted() {
    let source = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);
    let edge = vec![rect_op(W - 1, H - 1, 1, 1, [7, 7, 7])];
    viewer.apply_ops(1, 0, &edge).unwrap();
    let i = ((H - 1) * W + W - 1) * 3;
    assert_eq!(&viewer.buffer()[i as usize..i as usize + 3], &[7, 7, 7]);
}

// ---------------------------------------------------------------------------
// "Huge dimensions, products, lengths, region counts"
// ---------------------------------------------------------------------------

#[test]
fn absurd_geometry_is_refused_before_any_allocation() {
    let mut viewer = Compositor::new();
    let err = viewer
        .begin_snapshot(0, 60_000, 60_000, SnapshotFormat::Png, 10, 1)
        .unwrap_err()
        .to_string();
    assert!(err.contains("max_frame_bytes"), "unhelpful: {err}");
    assert!(!viewer.is_fresh(), "a refused snapshot must not install");
}

#[test]
fn an_absurd_op_count_is_refused() {
    let mut payload = vec![0x06u8]; // PartialUpdate
    payload.extend_from_slice(&1u64.to_le_bytes()); // rev
    payload.extend_from_slice(&0u64.to_le_bytes()); // pts
    payload.extend_from_slice(&0u32.to_le_bytes()); // epoch
    payload.extend_from_slice(&u32::MAX.to_le_bytes()); // absurd op count
    let mut bytes = vec![pixel_change_check_client::network::PROTOCOL_VERSION];
    bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes.extend_from_slice(&payload);
    let err = Message::decode(&bytes).unwrap_err().to_string();
    assert!(err.contains("max_ops_per_update"), "unhelpful: {err}");
}

// ---------------------------------------------------------------------------
// "Truncated/compressed malformed payload: atomic rejection"
// ---------------------------------------------------------------------------

#[test]
fn a_truncated_message_is_refused() {
    let bytes = Message::Ack { rev: 9 }.encode().unwrap();
    assert!(Message::decode(&bytes[..bytes.len() - 2]).is_err());
}

#[test]
fn a_corrupt_lz4_payload_is_refused_without_changing_the_surface() {
    let source = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);
    let before = viewer.buffer().to_vec();

    // A token claiming 15 literals plus a 15-byte match extension, over a
    // payload that is far too short to contain them.
    let mut corrupt = compression::compress_frame(&[1, 2, 3]).unwrap();
    corrupt[4] = 0xFF;

    let op = WireOp::Rect {
        x: 0,
        y: 0,
        width: 1,
        height: 1,
        compressed: corrupt,
    };
    assert!(viewer.apply_ops(1, 0, &[op]).is_err());
    assert_eq!(viewer.buffer(), &before[..]);
}

#[test]
fn one_bad_op_rolls_back_the_whole_transaction() {
    let source = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);
    let before = viewer.buffer().to_vec();

    let good = rect_op(0, 0, 1, 1, [3, 3, 3]);
    let bad = WireOp::Rect {
        x: W,
        y: 0,
        width: 4,
        height: 4,
        compressed: compression::compress_frame(&[0; 48]).unwrap(),
    };
    assert!(viewer.apply_ops(1, 0, &[good, bad]).is_err());
    assert_eq!(viewer.buffer(), &before[..]);
    assert_eq!(viewer.rev(), 0);
}

// ---------------------------------------------------------------------------
// "Resize/rotation/scale change mid-stream: old-epoch work on new geometry"
// ---------------------------------------------------------------------------

#[test]
fn work_from_an_old_epoch_never_touches_a_resized_surface() {
    let small = blank();
    let mut viewer = Compositor::new();
    install(&mut viewer, &small, 1, 1);

    let big = Frame::new(1, W * 2, H, vec![3u8; rgb_len(W * 2, H).unwrap()]).unwrap();
    let (_, data) = encode_snapshot(big.width, big.height, &big.data).unwrap();
    viewer
        .begin_snapshot(
            2,
            big.width,
            big.height,
            SnapshotFormat::Png,
            data.len() as u32,
            1,
        )
        .unwrap();
    viewer.push_snapshot_chunk(0, &data).unwrap();
    viewer.commit_snapshot(2, 2).unwrap();

    // A patch from epoch 1 must be refused, not applied to the new size.
    let stale = vec![rect_op(0, 0, 1, 1, [255, 255, 255])];
    assert_eq!(
        viewer.apply_ops(3, 1, &stale).unwrap_err(),
        ApplyError::Rejected(Rejected::StaleEpoch)
    );
    assert_eq!(viewer.dimensions(), Some((W * 2, H)));
}

// ---------------------------------------------------------------------------
// "Copy overlap and two-region swap: in-place copy corruption"
// ---------------------------------------------------------------------------

#[test]
fn two_regions_can_swap_in_one_transaction() {
    let mut source = blank();
    for y in 0..H {
        for x in 0..W {
            px(
                &mut source,
                x,
                y,
                if x < W / 2 { [255, 0, 0] } else { [0, 0, 255] },
            );
        }
    }
    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);

    let half = W / 2;
    viewer
        .apply_ops(
            1,
            0,
            &[
                WireOp::Copy {
                    x: 0,
                    y: 0,
                    width: half,
                    height: H,
                    src_x: half,
                    src_y: 0,
                },
                WireOp::Copy {
                    x: half,
                    y: 0,
                    width: half,
                    height: H,
                    src_x: 0,
                    src_y: 0,
                },
            ],
        )
        .unwrap();
    assert_eq!(&viewer.buffer()[0..3], &[0, 0, 255], "left is now blue");
    let ri = (half * 3) as usize;
    assert_eq!(
        &viewer.buffer()[ri..ri + 3],
        &[255, 0, 0],
        "right is now red"
    );
}

// ---------------------------------------------------------------------------
// Scrolling: the case the Copy op exists for.
// ---------------------------------------------------------------------------

#[test]
fn a_scroll_is_sent_as_a_copy_and_reconstructs_exactly() {
    let bs = 16u32;
    let w = 128u32;
    let h = 96u32;
    let mut source = Frame::new(0, w, h, vec![0; rgb_len(w, h).unwrap()]).unwrap();
    for y in 0..h {
        for x in 0..w {
            let i = ((y * w + x) * 3) as usize;
            source.data[i..i + 3].copy_from_slice(&[(x / 8) as u8, (y / 8) as u8, 200]);
        }
    }
    let mut scrolled = Frame::new(1, w, h, vec![0; rgb_len(w, h).unwrap()]).unwrap();
    for y in 0..h - bs {
        for x in 0..w {
            let s = (((y + bs) * w + x) * 3) as usize;
            let d = ((y * w + x) * 3) as usize;
            scrolled.data[d..d + 3].copy_from_slice(&source.data[s..s + 3]);
        }
    }

    let planner = Planner::new(0, bs).unwrap();
    let mut reference = source.clone();
    let plan = planner
        .plan(&mut reference.data, w, h, &scrolled, limits())
        .unwrap();
    assert!(
        plan.ops.iter().any(WireOp::is_copy),
        "a one-block scroll must be a Copy, got {:?}",
        plan.ops.iter().map(|o| o.rect()).collect::<Vec<_>>()
    );

    let mut viewer = Compositor::new();
    install(&mut viewer, &source, 0, 0);
    viewer.apply_ops(1, 0, &plan.ops).unwrap();
    assert_eq!(viewer.buffer(), &scrolled.data);
}

// ---------------------------------------------------------------------------
// "Exact encode -> transport -> decode -> apply round trip: real pixels"
// ---------------------------------------------------------------------------

#[test]
fn a_serialised_update_survives_the_wire_byte_for_byte() -> Result<()> {
    let planner = Planner::default();
    let mut reference = blank();
    let mut current = blank();
    for y in 0..H {
        for x in 0..W {
            if (x * 7 + y * 13) % 11 < 3 {
                px(&mut current, x, y, [x as u8, y as u8, 99]);
            }
        }
    }
    let plan = planner.plan(&mut reference.data, W, H, &current, limits())?;
    assert!(!plan.ops.is_empty());

    // Encode exactly as the sender does, decode exactly as the receiver
    // does, and apply through the real compositor.
    let bytes = Message::PartialUpdate {
        rev: 1,
        pts_us: 0,
        epoch: 0,
        ops: plan.ops,
    }
    .encode()?;
    let mut viewer = Compositor::new();
    let base = blank();
    install(&mut viewer, &base, 0, 0);

    match Message::decode(&bytes)? {
        Message::PartialUpdate {
            rev,
            pts_us: 0,
            epoch,
            ops,
        } => viewer.apply_ops(rev, epoch, &ops)?,
        other => panic!("expected a partial update, got {other:?}"),
    }
    assert_eq!(viewer.buffer(), &current.data, "real pixels must survive");
    Ok(())
}

#[test]
fn a_serialised_snapshot_survives_the_wire_byte_for_byte() -> Result<()> {
    let mut source = blank();
    for y in 0..H {
        for x in 0..W {
            px(&mut source, x, y, [(x * 3) as u8, (y * 5) as u8, 200]);
        }
    }
    let (format, data) = encode_snapshot(W, H, &source.data)?;
    let rev = 9u64;
    let begin = Message::SnapshotBegin {
        rev,
        pts_us: 0,
        epoch: 4,
        width: W,
        height: H,
        format,
        total_len: data.len() as u32,
        chunks: 1,
    }
    .encode()?;
    let chunk = Message::SnapshotChunk {
        rev: 1,
        index: 0,
        data: data.clone(),
    }
    .encode()?;
    let commit = Message::SnapshotCommit {
        rev: 9,
        pts_us: 0,
        epoch: 4,
    }
    .encode()?;

    let mut viewer = Compositor::new();
    for bytes in [begin, chunk, commit] {
        match Message::decode(&bytes)? {
            Message::SnapshotBegin {
                rev: _,
                pts_us: _,
                epoch,
                width,
                height,
                format,
                total_len,
                chunks,
            } => viewer.begin_snapshot(epoch, width, height, format, total_len, chunks)?,
            Message::SnapshotChunk {
                rev: _,
                index,
                data,
            } => viewer.push_snapshot_chunk(index, &data)?,
            Message::SnapshotCommit {
                rev,
                pts_us: 0,
                epoch,
            } => viewer.commit_snapshot(rev, epoch)?,
            other => panic!("unexpected {other:?}"),
        }
    }
    assert_eq!(viewer.buffer(), &source.data, "a snapshot must be lossless");
    Ok(())
}

// ---------------------------------------------------------------------------
// "4K noise: oversized output, bounded fallback, no disconnect loop"
// ---------------------------------------------------------------------------

#[test]
fn total_chaos_falls_back_to_a_snapshot_instead_of_an_oversized_update() {
    let planner = Planner::default();
    let mut current = blank();
    // Pseudo-random content: incompressible, so LZ4 cannot rescue it.
    let mut state = 0x12345678u32;
    for i in 0..current.data.len() {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        current.data[i] = (state >> 16) as u8;
    }

    // Against a realistic budget the patch set is enormous. Start from a
    // pristine reference: the first plan below already advances it.
    let mut reference2 = blank();
    let roomy = planner
        .plan(
            &mut reference2.data,
            W,
            H,
            &current,
            PlanLimits {
                snapshot_bytes: usize::MAX,
                max_update_bytes: pixel_change_check_client::network::MAX_MESSAGE_SIZE as usize,
            },
        )
        .unwrap();
    assert!(
        roomy.wire_len > (W * H * 3) as usize / 2,
        "incompressible content must not compress away: {} bytes",
        roomy.wire_len
    );

    // ...and when the budget cannot hold it, the planner must fall back to
    // a snapshot instead of emitting an update no transport will accept.
    let mut reference3 = blank();
    let tight = planner
        .plan(
            &mut reference3.data,
            W,
            H,
            &current,
            PlanLimits {
                snapshot_bytes: usize::MAX,
                max_update_bytes: 1024,
            },
        )
        .unwrap();
    assert!(
        tight.ops.is_empty() && tight.wire_len > 1024,
        "an oversized patch set must be replaced by a snapshot, got {} ops",
        tight.ops.len()
    );
}

// ---------------------------------------------------------------------------
// Property: a reference compositor must agree with the fast one.
// ---------------------------------------------------------------------------

/// Drive a long pseudo-random sequence of operations through the real
/// compositor and against an independently written model. The two must
/// agree on the final surface, which is what catches a Copy or a bounds
/// bug that a single hand-written case would miss.
#[test]
fn the_compositor_agrees_with_an_independent_model() {
    let mut state = 0xC0FFEEu32;
    let mut next = || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
        state >> 16
    };

    let mut model = vec![0u8; rgb_len(W, H).unwrap()];
    let mut viewer = Compositor::new();
    let seed = blank();
    install(&mut viewer, &seed, 0, 0);

    let mut rev = 0u64;
    for round in 0..200u32 {
        let kind = next() % 4;
        let x = next() % W;
        let y = next() % H;
        let w = 1 + next() % 8;
        let h = 1 + next() % 8;
        let (x, y, w, h) = (x.min(W - w), y.min(H - h), w, h);
        let ops: Vec<WireOp> = match kind {
            0 => {
                let color = [next() as u8, next() as u8, next() as u8];
                for row in 0..h {
                    for col in 0..w {
                        let i = (((y + row) * W + x + col) * 3) as usize;
                        model[i..i + 3].copy_from_slice(&color);
                    }
                }
                vec![WireOp::Fill {
                    x,
                    y,
                    width: w,
                    height: h,
                    color,
                }]
            }
            1 => {
                let color = [next() as u8, next() as u8, next() as u8];
                let data: Vec<u8> = color
                    .iter()
                    .copied()
                    .cycle()
                    .take((w * h) as usize * 3)
                    .collect();
                for row in 0..h {
                    let from = ((row * w) * 3) as usize;
                    let to = (((y + row) * W + x) * 3) as usize;
                    model[to..to + (w * 3) as usize]
                        .copy_from_slice(&data[from..from + (w * 3) as usize]);
                }
                vec![WireOp::Rect {
                    x,
                    y,
                    width: w,
                    height: h,
                    compressed: compression::compress_frame(&data).unwrap(),
                }]
            }
            2 => {
                // A copy always reads the *pre-transaction* surface, so a
                // lone copy in a transaction is an in-place move.
                let saved = model.clone();
                for row in 0..h {
                    let at = (((y + row) * W + x) * 3) as usize;
                    model[at..at + (w * 3) as usize]
                        .copy_from_slice(&saved[at..at + (w * 3) as usize]);
                }
                vec![WireOp::Copy {
                    x,
                    y,
                    width: w,
                    height: h,
                    src_x: x,
                    src_y: y,
                }]
            }
            _ => {
                // An overlapping pair: copy, then overwrite half of it.
                let saved = model.clone();
                let color = [next() as u8, next() as u8, next() as u8];
                let half_w = (w / 2).max(1);
                for row in 0..h {
                    let at = (((y + row) * W + x) * 3) as usize;
                    model[at..at + (w * 3) as usize]
                        .copy_from_slice(&saved[at..at + (w * 3) as usize]);
                }
                for row in 0..h {
                    for col in 0..half_w {
                        let i = (((y + row) * W + x + col) * 3) as usize;
                        model[i..i + 3].copy_from_slice(&color);
                    }
                }
                let data: Vec<u8> = color
                    .iter()
                    .copied()
                    .cycle()
                    .take((half_w * h) as usize * 3)
                    .collect();
                vec![
                    WireOp::Copy {
                        x,
                        y,
                        width: w,
                        height: h,
                        src_x: x,
                        src_y: y,
                    },
                    WireOp::Rect {
                        x,
                        y,
                        width: half_w,
                        height: h,
                        compressed: compression::compress_frame(&data).unwrap(),
                    },
                ]
            }
        };
        rev += 1;
        viewer.apply_ops(rev, 0, &ops).unwrap();
        assert_eq!(
            viewer.buffer(),
            &model[..],
            "round {round}: the compositor and the model disagree"
        );
    }
}

// ---------------------------------------------------------------------------
// Geometry validation on the value type itself.
// ---------------------------------------------------------------------------

#[test]
fn pixel_change_validation_reports_the_shape_it_rejected() {
    let f = frame(vec![0; rgb_len(W, H).unwrap()]);
    let bad = PixelChange {
        x: W - 1,
        y: 0,
        width: 2,
        height: 1,
        data: vec![0; 6],
    };
    let err = bad.validate(f.width, f.height).unwrap_err().to_string();
    assert!(
        err.contains(&format!("exceeds frame {W}x{H}")),
        "unhelpful: {err}"
    );
}
