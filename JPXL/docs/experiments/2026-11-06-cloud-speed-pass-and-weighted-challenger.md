# Cloud speed pass (decoder and encoder) and the effort 2-6 Weighted challenger

Date: 2026-11-06. Commits 145499e..a7cf31c on `claude/practical-hawking-acflqj`, measured
against f045e14 (the branch base, "HEAD" below).

## Question

On a 4-core cloud host with no access to the operator's images and no libjxl oracle, which
decoder and encoder hotspots can be removed without changing a decoded sample or an encoded
byte, and is there an exact (lossless) density gain that does not touch the default effort?

## Preregistered gate

Set before any change was made, from `AGENTS.md` §1 phase two and the Contract A determinism
decision:

- **Speed changes:** pass only if decoded pixels are identical to HEAD's on every corpus
  stream (lossy and lossless, PPM and PNG output), and encoded bytes are identical to HEAD's
  for every lossy encode (fast, balanced) and every effort-1 lossless encode, at the default
  thread count, at `--threads 1`, and with `JPXL_DISABLE_AVX2=1` where a SIMD path changed.
  A vectorized or threaded path must keep its scalar reference callable and a lock-step test
  against it. Timing claims are indicative only: the host is shared and unpinned
  (`@jpegxl-rs.observation.measuring-host-cannot-resolve-small-effects-2026-08-31`), so only
  effects far above noise are reported.
- **Density change:** the effort-1 default and efforts 7-9 must stay byte-identical; at the
  changed levels no Kodak image may get larger, and every stream must decode to the source
  pixels. *Disclosure:* this gate was written after the first density experiment (Weighted
  added to effort 2's cheap sweep) failed, see Raw results §3; the shipped design was chosen
  in response to that failure.

## Method

Host: cloud container, 4 vCPU Intel Xeon @ 2.10 GHz, 15 GiB, Linux 6.18, shared with the
session's own builds (load average 1.8-3.3 during timing). Toolchain 1.97.1 (pinned).
`release` profile (thin LTO). perf 6.8 (`record -F 999/1999 -g`, and `--call-graph dwarf`
for the page-fault attribution).

Binaries: `jpxl-head` sha256 `4cb907aebfb01f9a…` (f045e14), `jpxl-new` sha256
`094c5f4832e8982c…` (a7cf31c).

Corpus (not committed; under `.agent/scratch/corpus/`, which does not survive the container):

- Kodak PhotoCD set, 24 PNGs 768x512, fetched from the Git LFS mirror
  `lamco-admin/lamcogaussianimage` `test-data/kodak-dataset/` via
  `media.githubusercontent.com`; kodim01's sha256 equals the LFS pointer's oid. sha256 (first
  16 hex): kodim01 a56e27cbf5f843c0, 02 4f4b74a79237e311, 03 e25ca1ff2f0c0cb5,
  04 e3b946107c5d3441, 05 10349e963c5c813d, 06 363510303b715d4c, 07 b77d3f006f42414b,
  08 ba23983c76b4832e, 09 6a4361c2fc194feb, 10 9dfb70f5867c29ff, 11 7936814b58b5387f,
  12 d78c37c2f04f2376, 13 bc34a3ce58dea09d, 14 55a94550ff18f324, 15 7538cbb80cb91036,
  16 a89c7268ccd4718b, 17 37afcc89fbdcb76d, 18 1a9258c365988961, 19 b7450b264b1b0a41,
  20 3b46c71e3b92a563, 21 ac958597c82073f6, 22 1cee58eb1f2d9c7e, 23 e3111a2fd4da24af,
  24 1071c68372cc5a01.
- Derived with Pillow 13.0.0: each Kodak image as baseline JPEG q90 4:2:0 (kodim05.jpg
  81057b583bc2676d); three 3072x2048 mosaics of 16 shuffled Kodak frames (portrait frames
  rotated, `random.Random(1234)`), as PNG and JPEG q90 (mosaic0.png f0094fb671acb5a6,
  mosaic0.jpg cf12be06f952a7b4).
- Synthetic: `gradnoise.png` 2048x1536 RGB ramp + N(0, 6) noise (numpy seed 7,
  97d54a0cb8f32459); `ui.png` 1920x1080 striped rows of PIL default-font text and bars
  (6c5f135f43079b8d); `grad16_gray.png` 2048x1536 16-bit grey ramp (43c279274f42a655).

Identity harness: encode every input (lossy fast and balanced from the JPEGs, mosaics and
synthetic; lossless efforts 1 and 2 from Kodak 01-06, mosaic0 and the synthetic PNGs) with
each binary, sha256 every stream, decode every stream (HEAD's streams with both decoders,
and each binary's own streams), sha256 every decoded raster. Higher-effort lossless streams
(efforts 3, 5, 7, 9 on kodim05; 7 on ui and grad16) were added for the decoder's MaTree-usage
change. Timing: `abtime.sh`, five interleaved A/B pairs per command, process wall time,
reporting min and median.

## Raw results

### 1. Identity (final binary vs HEAD, full harness)

- 76 encodes: 68 byte-identical; the 8 that differ are exactly the effort-2 lossless streams
  (the intended density change, §3), 18,213,425 → 17,734,619 B, none larger.
- 76 HEAD streams decoded by the new decoder: 76 identical outputs.
- 20 of 20 new lossless streams decode to the same pixels as HEAD's streams.
- Lossy bytes also identical at `--threads 1` and with `JPXL_DISABLE_AVX2=1` (kodim05 fast);
  lossless bytes identical at `--threads 1/2/3/4` (mosaic0 effort 1) and at `--threads 1`
  for efforts 1, 2, 7 (kodim05, ui, grad16).
- `cargo test --workspace --release`: 1658 passed, 0 failed (1653 at HEAD; five new tests).
  `cargo clippy --workspace --all-targets -- -D warnings` and `cargo fmt --all --check`
  clean.

### 2. Timing, final binary vs HEAD (ms, min / median of 5)

| Command | HEAD | new | median ratio |
| --- | --- | --- | --- |
| decode lossy 3072x2048 (fast stream) | 1325 / 1381 | 564 / 587 | 2.35x |
| decode lossy 3072x2048 (balanced stream) | 1310 / 1376 | 564 / 607 | 2.27x |
| decode lossless effort 1, 3072x2048 | 2039 / 2140 | 409 / 430 | 4.98x |
| decode lossy kodim05 (fast) | 94 / 103 | 47 / 50 | 2.06x |
| decode lossless effort 1 kodim05 | 131 / 134 | 49 / 50 | 2.68x |
| encode `--quality --effort fast`, mosaic0.jpg | 1811 / 1892 | 1639 / 1698 | 1.11x |
| encode `--quality --effort balanced`, mosaic0.jpg | 2070 / 2334 | 1911 / 1958 | 1.19x |
| encode `--quality --effort balanced`, kodim05.jpg | 197 / 198 | 180 / 188 | 1.05x |
| encode lossless effort 1, mosaic0.png | 3187 / 3276 | 1832 / 1877 | 1.75x |
| encode lossless effort 1, kodim05.png | 351 / 354 | 305 / 318 | 1.11x |
| encode lossless effort 7, kodim05.png (single runs) | 5707 | 2610 | 2.2x |

Attribution, from perf before each change (6 MP lossy decode unless stated):

| Change (commit) | Profile evidence | Step measured |
| --- | --- | --- |
| Per-frame dequant matrix cache (145499e) | `dct_quant_weights`/`interpolate`/`pow` ~19% of CPU, rebuilt per varblock and channel | 1730 → 1427 ms |
| EPF block-row lanes, decoder (145499e) | `epf_step_rows_fma` 52% of CPU, scalar per tap | 1427 → 846 ms |
| Modular pass groups on workers, MaTree-usage skips (f457c70) | lossless decode 100% serial; `WeightedState::predict` 36% though the effort-1 tree never uses it | lossless 2086 → 430 ms |
| CLI 8-bit identity interleave (f457c70) | ~0.15 s serial u64 division per sample in output | (with the above) lossy 1357 → 674 ms |
| VarDCT HF entropy decode on workers (7c1ea67) | ~0.2 s serial front, `decode_hf_group` on the main thread | 674 → 569 ms |
| Global residual model on workers (96356c1) | ~2 s single-threaded stretch in lossless encode | 3303 → 1823 ms |
| Metric scratch lazily zeroed (97658fd) | one ~0.2 s single-thread `clear_page` window; 27% of all encode samples were page faults | 1801 → 1719 ms |
| Encoder EPF weight lanes (97658fd) | `restoration::epf_step_rows_fma` 10.6% of samples | → 6.9% of samples (wall within noise) |
| Pre-analysis on the executor (ad6f6ee) | `preanalysis_srgb16` ~0.2 s on the main thread | 1888 → 1704 ms (vs HEAD) |
| Planner full scans on workers (45340bb) | effort ≥ 2 planning 100% serial | effort 7 kodim05 5.7 → 2.6 s |

### 3. Density: Weighted predictor at efforts 2-6

At HEAD, efforts 1, 2, 3 and 5 emit identical bytes on kodim05 (617,612) and kodim23, at
2.3-3x effort 1's time; effort 7 emits 585,115. Effort 7 with `--modular-max-depth 0` (single
leaf) also emits 585,115: the gain is the Weighted root, not the split search.

**Negative result (not shipped).** Adding `Predictor::Weighted` to effort 2's cheap sweep
gave kodim05 585,115 B and the 6 MP mosaic −3.3%, but turned the 16-bit grey ramp from
11,834 B into **16,907 B (+43%)**: the cheap tier prices Weighted with a full sequential scan
and the stateless predictors with a stride-8 sample, and the two estimates are not
comparable. It also cost 6x effort 1's time on the mosaic before the planner was threaded.

**Shipped (a7cf31c).** Efforts 2-6 exact-price (Stage C) a single-leaf Weighted tree after
the normal search and keep it only if its exact price is strictly lower. Effort 2, all 24
Kodak images: 23 smaller, 1 equal (kodim08), 0 larger; geomean ratio 0.9687; Kodak total
12,859,484 → 12,421,615 B (−3.41%; −2.69% with the three synthetic images added); per image, e.g. kodim03 452,482 → 430,792, kodim05 617,612 →
585,115 (equal to effort 7), kodim23 470,630 → 444,369. Efforts 3 and 5 match effort 2 on
kodim01-06. Unchanged: ui (132,923 / 130,237 at e5), grad16 (11,834 at e2, 939 at e3/e5).
gradnoise −1.3%; mosaic0 −3.3% at effort 2, 4.3 s vs HEAD's 5.6 s for the larger stream.
Kodak effort-2 time ~0.7 s per image (HEAD ~0.5 s; effort 7 needs ~2.4 s for the same
bytes).

## Conclusion

Every speed change met its gate: pixels and bytes identical on the whole harness, scalar
references kept with lock-step tests (`lane_path_matches_scalar_reference_bitwise`,
`lane_path_matches_the_per_pixel_reference_bitwise`, `usage_reports_exactly_what_the_tree_can_observe`,
`the_threaded_pass_matches_the_serial_one_exactly`), and each lane test was checked to fail
under an injected mutation. The decoder is now 2-5x faster on 6 MP frames; the lossy encoder
gained 10-20% and is bounded by the perceptual metric's blur (already AVX2) and the probe
count, which the ledger records as predictor-bound; effort-1 lossless encode is 1.75x faster.

The density change met its gate on Kodak and the synthetic set. It does **not** establish
behaviour on content unlike these (e.g. paintings, screenshots with large flat areas, 16-bit
photographs), and it was not compared with cjxl (no oracle on this host). Timings are single
host, unpinned, and indicative only; the ratios above are large enough not to depend on that.

## Consequences

- Code: the nine commits listed above. No defaults of the lossy path changed; effort-1
  lossless and efforts 7-9 are byte-identical; efforts 2-6 lossless are smaller.
- Ledger: not updated — the `akr` CLI could not be installed in this session. Records to
  add when it is: evidence for each speed commit (this report), a decision for the effort
  2-6 challenger, and an observation retiring the "effort ladder inert above level 1" claim
  for efforts 2-6 on photographs. The `decode` timings also supersede nothing in the ledger,
  which had no decoder-speed record.
- Still open from scoping, in order of expected value: the lossless encoder emits both a
  per-section-tree and a global-tree candidate and recomputes residuals and LZ77 for the
  final write (~40% of effort-1 wall); the decoder's sRGB `powf` (~14% of lossy decode CPU,
  parallel, cannot be made bit-identical with a custom pow); the lossy encoder's probe
  render still allocates fresh planes for large frames (retention was rejected for memory,
  E1-a); and `--text-routing` (opt-in) halved the synthetic UI image at balanced (140,860 →
  67,686 B at a higher score, 2.5x time), whose default-on decision remains the operator's
  under the Contract B screen.
