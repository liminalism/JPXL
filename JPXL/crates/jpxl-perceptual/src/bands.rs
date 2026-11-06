//! Fixed-height row bands and one-shot hand-off of mutable work to executor
//! items.
//!
//! An executor closure is `Fn`, so it cannot own the `&mut` slices its items
//! write. Each item's slices are parked in a `Mutex<Option<_>>` and taken
//! exactly once by the item that owns them — the same pattern the encoder's
//! source preparation uses. Bands are cut at a fixed row count so that the
//! partition, and therefore every partial sum, is independent of the host.

use std::sync::Mutex;

/// Rows per band. Fixed: the band partition must not depend on the worker
/// count, or reduction order — and with it the score — would.
pub(crate) const BAND_ROWS: usize = 32;

/// Number of bands needed to cover `rows` rows.
pub(crate) const fn band_count(rows: usize) -> usize {
    rows.div_ceil(BAND_ROWS)
}

/// Shapes `buf` to `len` zeros, exactly as `clear()` then `resize(len, 0.0)`.
///
/// The difference is who pays for fresh memory. When `buf` must grow,
/// `resize` zero-fills the new allocation on the calling thread — for a
/// frame-sized scratch plane that is tens of megabytes of serial page
/// faulting on the main thread, which profiling put at ~8% of a 6 MP
/// encode's wall time. `vec!`'s zeroed allocation hands out the kernel's
/// lazily zeroed pages instead, so they are first touched by the band
/// workers that write them, in parallel. Either way the buffer holds `len`
/// zeros on return, so callers see identical contents.
pub(crate) fn zeroed_in_place(buf: &mut Vec<f32>, len: usize) {
    if buf.capacity() < len {
        *buf = vec![0.0; len];
    } else {
        buf.clear();
        buf.resize(len, 0.0);
    }
}

/// Byte-free view of one band of a read-only plane.
pub(crate) fn band_of(plane: &[f32], band: usize, band_len: usize) -> &[f32] {
    let start = band.saturating_mul(band_len).min(plane.len());
    let end = start.saturating_add(band_len).min(plane.len());
    plane.get(start..end).unwrap_or(&[])
}

/// Work items parked for one-shot pickup by executor closures.
pub(crate) struct Handoff<T> {
    items: Vec<Mutex<Option<T>>>,
}

impl<T> Handoff<T> {
    pub(crate) fn new(items: Vec<T>) -> Self {
        Self {
            items: items.into_iter().map(|t| Mutex::new(Some(t))).collect(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.items.len()
    }

    /// Takes item `index`; `None` if it does not exist or was already taken.
    pub(crate) fn take(&self, index: usize) -> Option<T> {
        self.items.get(index)?.lock().ok()?.take()
    }
}

/// Row bands over several equally sized mutable planes: band `i` of every
/// plane is handed to item `i` together.
pub(crate) fn mutable_bands(planes: Vec<&mut [f32]>, band_len: usize) -> Handoff<Vec<&mut [f32]>> {
    let band_len = band_len.max(1);
    let mut iters: Vec<_> = planes
        .into_iter()
        .map(|plane| plane.chunks_mut(band_len))
        .collect();
    let mut bands = Vec::new();
    loop {
        let mut group = Vec::with_capacity(iters.len());
        for iter in &mut iters {
            if let Some(chunk) = iter.next() {
                group.push(chunk);
            }
        }
        if group.is_empty() {
            break;
        }
        bands.push(group);
    }
    Handoff::new(bands)
}
