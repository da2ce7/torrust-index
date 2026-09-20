// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`full_width_u64_cells_keep_distinct_encodings`] | routing | Distinct values routed into one full-width cell have distinct suffix encodings. |
//! | [`full_width_u128_cells_keep_distinct_encodings`] | routing | The full-width routing correction also holds at the widest supported coordinate width. |
//! | [`fresh_graph_has_one_node`] | routing | Before anything is observed the graph is a single cell spanning the whole coordinate domain. There is no partition worth choosing until traffic says where the boundaries should fall, so the sentinel starts with the one cell it can justify and refines outward from there. |
//! | [`fresh_graph_has_one_terminal`] | routing | cites (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´) |
//! | [`fresh_graph_has_zero_total_sum`] | routing | cites (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´) |
//! | [`ingest_feeds_graph_with_delta_one`] | routing | Every value in a batch contributes exactly one unit of importance, whichever range it falls in, so the graph's total after successive batches into unrelated ranges is simply how many values were handed over. Importance is a count of arrivals rather than a weight the caller can set, which is what lets the selector read it as evidence of where traffic is. |
//! | [`empty_ingest_does_not_observe`] | routing | A batch with nothing in it is not an event. The accumulated total stays where it was and no cell is created, so an interval in which nothing arrived neither adds evidence nor moves the partition. Quiet time is therefore invisible to the spatial layer rather than being recorded as an observation of emptiness. |
//! | [`duplicate_values_each_contribute`] | routing | cites (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´) |
//! | [`graph_accumulates_across_batches`] | routing | cites (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´) |
//! | [`lifetime_observations_tracks_total_sum`] | routing | The counter the sentinel keeps and the total the graph accumulates stay equal batch after batch. They are one quantity read from two layers: real observations are the only thing that increments either, and the synthetic data used to warm trackers is deliberately kept out of both. A host can therefore read whichever is nearer to hand without learning which layer maintains it. |
//! | [`concentrated_traffic_splits_nodes`] | routing | Traffic that keeps landing in one narrow range drives that range past the split threshold and the graph refines it, so the partition is bought with observations rather than configured up front. Where the traffic goes is where the resolution appears; the total meanwhile still counts exactly the values handed over, so refining a region does not manufacture evidence. |
//! | [`concentrated_traffic_grows_terminals`] | routing | cites (´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´) |
//! | [`diverse_traffic_respects_budget`] | routing | Traffic spread thinly over many well-separated ranges asks the graph to refine everywhere at once, and the node budget is what keeps that from being unbounded: however many ranges are busy and however low the split threshold is set, the graph holds no more cells than the budget allows. The cost of modelling is a configured ceiling rather than a function of how widely an adversary chooses to scatter. |
//! | [`reset_restores_fresh_graph_state`] | routing | Reset discards the partition as well as the evidence: a graph that had split under load comes back as the single root cell of a fresh sentinel, with nothing accumulated and nothing to land in but the root. Structure is derived from observations, so once the observations are dropped there is no refinement left worth preserving, and a reset sentinel cannot be distinguished from a new one by what its graph holds. |
//! | [`top_of_domain_coordinate_routes_to_a_cell`] | routing | The topmost coordinate of the domain reaches a tracker rather than falling through every cell. Cell intervals are half-open, which has no upper edge case while the coordinate width is narrower than the coordinate type — the bound is then a representable value outside the domain. At the full width the domain's maximum is the type's maximum, there is no value above it to be excluded, and a half-open reading of the topmost interval therefore excludes a coordinate that is genuinely inside the domain. The spatial layer counts that observation either way, so the two readings would disagree: the accumulated total records an arrival that no tracker was ever shown. |
//! | [`an_ordinary_coordinate_still_lands_in_one_cell`] | routing | cites (´claim:routing:the-domains-top-coordinate-reaches-a-tracker-rather-than-falling-through-every-cell´) |
//! | [`values_outside_the_domain_are_counted_nowhere`] | routing | A coordinate the domain cannot name is not an observation of it, and the three layers that would each read it differently are not left to disagree about that. The spatial layer accumulates such a value in the topmost cell, whose interval does not contain it; the encoder reads the low bits of the configured width, so it would hand a tracker the vector of the in-domain value the arrival is congruent to; and the interval scan matches no cell at all, not even the root. Deciding membership once, before any of them, is what keeps the counts one count: the value raises no total, moves no partition and reaches no tracker. |
//! | [`a_signed_coordinate_outside_the_domain_is_counted_nowhere_at_full_width`] | routing | Domain membership is decided by comparison against the domain's own bounds, so a coordinate type the crate does not ship is held to the same domain as the ones it does. The bridge into centred bits is public and nothing closes the set of its implementations, and the coordinate trait is implemented for the floats as well as the unsigned integers, so a host's coordinates may be signed and NaN-capable. Inferring from the bit width that every representable value is in the domain holds only for the unsigned types: at a width that fills a signed or floating type it would admit a value below the origin, a NaN and an infinity. The comparison is the root cell's own containment test and the root is permanent, so whatever the boundary admits the partition delivers, for any coordinate type a host may bring. |
//! | [`a_signed_coordinate_below_the_origin_is_counted_nowhere_at_a_narrow_width`] | routing | cites (´claim:routing:domain-membership-is-decided-by-comparison-for-every-coordinate-type´) |

//! Graph routing — how an observed value reaches the cell that will
//! analyse it.
//!
//! Ingestion has one job at the spatial layer: hand every raw value to the
//! G-V Graph as a single unit of importance. Nothing is weighted, discounted
//! for repetition, or batched away, so the graph's accumulated total is a
//! count of arrivals and nothing else. That is what makes the sentinel's own
//! lifetime counter and the graph's total two readings of one number rather
//! than two tallies that could drift apart.
//!
//! Structure then follows traffic. A range that keeps receiving values crosses
//! the split threshold and is refined into finer cells, which is how the
//! sentinel comes to model where traffic actually is rather than a partition
//! chosen in advance. Refinement is not free, so it is bounded: the node
//! budget caps how many cells the graph will hold however many distinct
//! ranges the traffic touches. Spreading traffic thinly across the domain
//! therefore costs a configured ceiling of memory rather than unbounded
//! growth, which matters because that spread is something an adversary
//! chooses.
//!
//! Both ends of that lifecycle are observable. A fresh sentinel is one root
//! cell spanning the whole domain with nothing accumulated, and `reset()`
//! returns it to exactly that state — the learned partition is derived from
//! observations, so discarding the observations leaves nothing worth keeping
//! behind.

mod common;

use common::{cell_values, test_config};
use torrust_mudlark::Coordinate;
use torrust_sentinel::{CentredBitSource, CentredBits, Sentinel128, SentinelConfig, SpectralSentinel};

// ── Fresh state ─────────────────────────────────────────────

/// Before anything is observed the graph is a single cell spanning the whole
/// coordinate domain. There is no partition worth choosing until traffic says
/// where the boundaries should fall, so the sentinel starts with the one cell
/// it can justify and refines outward from there.
///
/// ´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´
/// ´test:integration:fresh-graph-has-one-node´
#[test]
fn fresh_graph_has_one_node() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().node_count(), 1);
}

/// That single cell is also a leaf. Nothing has been split, so the one node
/// the graph holds is the one place an observation can land, and the count of
/// cells traffic can reach agrees with the count of nodes that exist.
///
/// (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´)
/// ´test:integration:fresh-graph-has-one-terminal´
#[test]
fn fresh_graph_has_one_terminal() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().terminal_count(), 1);
}

/// The other half of the fresh state: that root cell carries no accumulated
/// importance either. A cell exists because the domain has to be covered, not
/// because anything was seen in it, so structure and evidence start out
/// independent of one another.
///
/// (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´)
/// ´test:integration:fresh-graph-has-zero-total-sum´
#[test]
fn fresh_graph_has_zero_total_sum() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().total_sum(), 0);
}

// ── Basic routing ───────────────────────────────────────────

/// Every value in a batch contributes exactly one unit of importance,
/// whichever range it falls in, so the graph's total after successive batches
/// into unrelated ranges is simply how many values were handed over.
/// Importance is a count of arrivals rather than a weight the caller can set,
/// which is what lets the selector read it as evidence of where traffic is.
///
/// ´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´
/// ´test:integration:ingest-feeds-graph-with-delta-one´
#[test]
fn ingest_feeds_graph_with_delta_one() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    // First batch: 5 values.
    s.ingest(&cell_values(0xA, 5));
    assert_eq!(s.graph().total_sum(), 5);

    // Second batch: 3 more values in a different range.
    s.ingest(&cell_values(0x3, 3));
    assert_eq!(s.graph().total_sum(), 8);
}

/// A batch with nothing in it is not an event. The accumulated total stays
/// where it was and no cell is created, so an interval in which nothing
/// arrived neither adds evidence nor moves the partition. Quiet time is
/// therefore invisible to the spatial layer rather than being recorded as an
/// observation of emptiness.
///
/// ´claim:routing:an-empty-batch-routes-nothing-and-leaves-the-graph-exactly-as-it-was´
/// ´test:integration:empty-ingest-does-not-observe´
#[test]
fn empty_ingest_does_not_observe() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&[]);
    assert_eq!(s.graph().total_sum(), 0);
    assert_eq!(s.graph().node_count(), 1); // still just the root
}

/// Repetition is traffic, not redundancy: one value handed over several times
/// in a batch counts several times over rather than collapsing into a single
/// arrival. The graph measures how often a region is visited, not how many
/// distinct values it has ever seen, which is why a flood from a single source
/// still moves the structure.
///
/// (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´)
/// ´test:integration:duplicate-values-each-contribute´
#[test]
fn duplicate_values_each_contribute() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    // Ingest three identical values — each should add 1 to total_sum.
    let v = cell_values(0xB, 1)[0];
    s.ingest(&[v, v, v]);
    assert_eq!(s.graph().total_sum(), 3);
}

// ── Accumulation ────────────────────────────────────────────

/// The same unit contribution survives batch boundaries: a long run of
/// batches scattered over many ranges leaves a total equal to everything ever
/// handed over. A batch is a delivery convenience, not an accounting period,
/// so the graph reports lifetime evidence rather than the most recent window.
///
/// (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´)
/// ´test:integration:graph-accumulates-across-batches´
#[test]
fn graph_accumulates_across_batches() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    for nibble in 0..10u128 {
        s.ingest(&cell_values(nibble, 100));
    }

    assert_eq!(s.graph().total_sum(), 1000);
}

/// The counter the sentinel keeps and the total the graph accumulates stay
/// equal batch after batch. They are one quantity read from two layers: real
/// observations are the only thing that increments either, and the synthetic
/// data used to warm trackers is deliberately kept out of both. A host can
/// therefore read whichever is nearer to hand without learning which layer
/// maintains it.
///
/// ´claim:routing:the-sentinels-lifetime-counter-and-the-graphs-total-are-one-quantity-read-twice´
/// ´test:integration:lifetime-observations-tracks-total-sum´
#[test]
fn lifetime_observations_tracks_total_sum() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&cell_values(0xC, 42));
    assert_eq!(s.lifetime_observations(), s.graph().total_sum());

    // Still holds after a second batch.
    s.ingest(&cell_values(0xD, 58));
    assert_eq!(s.lifetime_observations(), s.graph().total_sum());
    assert_eq!(s.lifetime_observations(), 100);
}

// ── Structural evolution ────────────────────────────────────

/// Traffic that keeps landing in one narrow range drives that range past the
/// split threshold and the graph refines it, so the partition is bought with
/// observations rather than configured up front. Where the traffic goes is
/// where the resolution appears; the total meanwhile still counts exactly the
/// values handed over, so refining a region does not manufacture evidence.
///
/// ´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´
/// ´test:integration:concentrated-traffic-splits-nodes´
#[test]
fn concentrated_traffic_splits_nodes() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // All values in the same nibble range — should concentrate
    // into one cell and eventually split it.
    s.ingest(&cell_values(0xA, 50));

    assert_eq!(s.graph().total_sum(), 50);
    assert!(
        s.graph().node_count() > 1,
        "expected splits from concentrated traffic, got {} nodes",
        s.graph().node_count()
    );
}

/// Refinement creates new places for traffic to land, not merely new interior
/// structure above the old cell: after a split the graph has more than one
/// leaf. Subsequent values in that range are therefore separated from one
/// another instead of continuing to pile into a single undifferentiated cell.
///
/// (´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´)
/// ´test:integration:concentrated-traffic-grows-terminals´
#[test]
fn concentrated_traffic_grows_terminals() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&cell_values(0xA, 50));

    assert!(
        s.graph().terminal_count() > 1,
        "expected terminal_count > 1 after splits, got {}",
        s.graph().terminal_count()
    );
}

// ── Budget enforcement ──────────────────────────────────────

/// Traffic spread thinly over many well-separated ranges asks the graph to
/// refine everywhere at once, and the node budget is what keeps that from
/// being unbounded: however many ranges are busy and however low the split
/// threshold is set, the graph holds no more cells than the budget allows.
/// The cost of modelling is a configured ceiling rather than a function of how
/// widely an adversary chooses to scatter.
///
/// ´claim:routing:the-node-budget-caps-the-graph-however-widely-traffic-is-scattered´
/// ´test:integration:diverse-traffic-respects-budget´
#[test]
fn diverse_traffic_respects_budget() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        budget: 200,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Spread observations across many distinct nibble ranges.
    for nibble in 0..16u128 {
        let values: Vec<u128> = (0u128..500).map(|i| (nibble << 124) | (i << 100)).collect();
        s.ingest(&values);
    }

    assert!(
        s.graph().node_count() <= 200,
        "node count {} exceeds budget 200",
        s.graph().node_count()
    );
}

// ── Reset ───────────────────────────────────────────────────

/// Reset discards the partition as well as the evidence: a graph that had
/// split under load comes back as the single root cell of a fresh sentinel,
/// with nothing accumulated and nothing to land in but the root. Structure is
/// derived from observations, so once the observations are dropped there is no
/// refinement left worth preserving, and a reset sentinel cannot be
/// distinguished from a new one by what its graph holds.
///
/// ´claim:routing:reset-returns-the-graph-to-the-single-root-cell-of-a-fresh-sentinel´
/// ´test:integration:reset-restores-fresh-graph-state´
#[test]
fn reset_restores_fresh_graph_state() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Ingest enough to split.
    s.ingest(&cell_values(0xA, 100));
    assert!(s.graph().total_sum() > 0);
    assert!(s.graph().node_count() > 1);

    s.reset();

    assert_eq!(s.graph().total_sum(), 0);
    assert_eq!(s.graph().node_count(), 1);
    assert_eq!(s.graph().terminal_count(), 1);
}

/// The topmost coordinate of the domain reaches a tracker rather than falling
/// through every cell. Cell intervals are half-open, which has no upper edge
/// case while the coordinate width is narrower than the coordinate type — the
/// bound is then a representable value outside the domain. At the full width
/// the domain's maximum is the type's maximum, there is no value above it to
/// be excluded, and a half-open reading of the topmost interval therefore
/// excludes a coordinate that is genuinely inside the domain. The spatial
/// layer counts that observation either way, so the two readings would
/// disagree: the accumulated total records an arrival that no tracker was ever
/// shown.
///
/// ´claim:routing:the-domains-top-coordinate-reaches-a-tracker-rather-than-falling-through-every-cell´
/// ´test:integration:top-of-domain-coordinate-routes-to-a-cell´
#[test]
fn top_of_domain_coordinate_routes_to_a_cell() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&[u128::MAX]);

    assert_eq!(s.graph().total_sum(), 1, "the spatial layer counts the observation");

    // Competitive and ancestor cells are reported separately, and a fresh
    // sentinel holds only the root, which is an ancestor by construction.
    let routed: usize = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|c| c.sample_count)
        .sum();
    assert_eq!(routed, 1, "and a tracker must be shown the same observation");
}

/// The inclusive reading is confined to the top of the domain, so an ordinary
/// coordinate still lands in exactly one cell of the partition. Widening the
/// upper bound everywhere would put each boundary value in two sibling cells
/// at once and count it twice; widening it only where there is no successor
/// leaves every other boundary exactly as it was.
///
/// (´claim:routing:the-domains-top-coordinate-reaches-a-tracker-rather-than-falling-through-every-cell´)
/// ´test:integration:an-ordinary-coordinate-still-lands-in-one-cell´
#[test]
fn an_ordinary_coordinate_still_lands_in_one_cell() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&cell_values(0x7, 1));

    assert_eq!(s.graph().total_sum(), 1);

    let routed: usize = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|c| c.sample_count)
        .sum();
    assert_eq!(routed, 1, "one arrival is shown to one tracker");
}

/// A coordinate the domain cannot name is not an observation of it, and the
/// three layers that would each read it differently are not left to disagree
/// about that. The spatial layer accumulates such a value in the topmost cell,
/// whose interval does not contain it; the encoder reads the low bits of the
/// configured width, so it would hand a tracker the vector of the in-domain
/// value the arrival is congruent to; and the interval scan matches no cell at
/// all, not even the root. Deciding membership once, before any of them, is
/// what keeps the counts one count: the value raises no total, moves no
/// partition and reaches no tracker.
///
/// The width here is narrower than the coordinate type, which for an unsigned
/// coordinate is the only shape in which a value outside the domain is
/// representable: at the full width every value `u64` can hold is inside the
/// domain, and the tests named `top_of_domain_coordinate_routes_to_a_cell`,
/// `an_ordinary_coordinate_still_lands_in_one_cell` and
/// `lifetime_observations_tracks_total_sum` hold that reading unchanged. A
/// coordinate type that is signed or NaN-capable has values outside the domain
/// at every width, which is what the two tests at the end of this file cover.
///
/// ´claim:routing:a-coordinate-outside-the-domain-is-counted-in-no-total-and-reaches-no-tracker´
/// ´test:integration:values-outside-the-domain-are-counted-nowhere´
#[test]
fn values_outside_the_domain_are_counted_nowhere() {
    // At N = 8 over a 64-bit coordinate the domain is [0, 256), so 256 is the
    // first representable value outside it — and the value the 8-bit encoder
    // would present as zero.
    let mut s = torrust_sentinel::SpectralSentinel::<u64, u64, 8>::new(test_config()).unwrap();

    let report = s.ingest(&[42, 256]);

    assert_eq!(s.graph().total_sum(), 1, "only the in-domain value moves the graph");
    assert_eq!(
        s.lifetime_observations(),
        1,
        "and only it is counted against the sentinel's lifetime"
    );

    let root = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .find(|cell| cell.depth == 0)
        .expect("the root tracker is permanent and receives every observation");
    assert_eq!(
        root.sample_count, 1,
        "the root is shown the in-domain value alone — nothing aliased onto it"
    );

    // A batch of nothing but out-of-domain values is the same non-event as an
    // empty batch: no total moves and the report describes no observation.
    let empty = s.ingest(&[256, 257, u64::MAX]);

    assert_eq!(s.graph().total_sum(), 1, "the graph total is where the first batch left it");
    assert_eq!(s.lifetime_observations(), 1, "and so is the lifetime count");
    assert!(
        empty.cell_reports.is_empty() && empty.ancestor_reports.is_empty(),
        "no cell was shown anything, so no cell reports"
    );
    assert!(
        empty.oldest_observation_age_micros.is_none(),
        "a batch with no observation in it has no oldest observation to age"
    );
}

/// Check the first full-width midpoint and the maximum in a fresh engine.
fn assert_full_width_encodings<C: torrust_sentinel::CentredBitSource, const N: u32>() {
    let config = SentinelConfig::<u64> {
        split_threshold: 1,
        d_create: 1,
        d_evict: 2,
        analysis_k: 16,
        max_rank: 1,
        noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(Vec::new()),
        ..SentinelConfig::default()
    };
    let top = C::domain_max(N);
    let boundary = C::midpoint(C::zero(), top);
    let mut sentinel = torrust_sentinel::SpectralSentinel::<C, u64, N>::new(config).unwrap();
    let report = sentinel.ingest(&[boundary, top]);
    let boundary_bits = boundary.to_centred_bits(N);
    let top_bits = top.to_centred_bits(N);
    let cells: Vec<_> = report.cell_reports.iter().chain(&report.ancestor_reports).collect();
    assert!(cells.iter().any(|cell| cell.depth == 1), "the fixture must split the root");
    for cell in cells {
        if cell.sample_count == 2 {
            let depth = u8::try_from(cell.depth).unwrap();
            assert_ne!(
                boundary_bits.suffix(depth),
                top_bits.suffix(depth),
                "distinct values in one cell must have distinct encodings at depth {depth}"
            );
        }
    }
}

/// Distinct values routed into one full-width cell have distinct suffix encodings.
///
/// ´claim:routing:full-width-cells-keep-distinct-encodings´
/// ´test:integration:full-width-u64-cells-keep-distinct-encodings´
#[test]
fn full_width_u64_cells_keep_distinct_encodings() {
    assert_full_width_encodings::<u64, 64>();
}

/// The full-width routing correction also holds at the widest supported coordinate width.
///
/// ´test:integration:full-width-u128-cells-keep-distinct-encodings´
#[test]
fn full_width_u128_cells_keep_distinct_encodings() {
    assert_full_width_encodings::<u128, 128>();
}

// ── A host's own coordinate type ────────────────────────────

/// A coordinate type of the kind a downstream host writes and this crate does
/// not ship: a newtype over `f64` whose every coordinate method is `f64`'s own.
///
/// The bridge into centred bits is public and nothing closes the set of its
/// implementations, and the coordinate trait is implemented for the floats as
/// well as for the unsigned integers, so a host's coordinates may be signed and
/// NaN-capable. Nothing here narrows what such a host may write: each method
/// forwards to the implementation Mudlark already publishes for `f64`, so the
/// type claims no behaviour the trait does not already permit, and it carries
/// `f64`'s refusal of a successor value unchanged rather than inventing one the
/// float line does not have. A stand-in that quietly behaved better than `f64`
/// would prove something about itself rather than about what a host may bring.
///
/// The conversion into centred bits reads the value's IEEE 754 bit pattern,
/// which is deterministic and needs no cast. Which bits a host derives from its
/// coordinates is its own affair, and the choice is not under test here: the
/// domain decision is taken before the encoder is reached, so no value these
/// tests expect to be rejected ever arrives at this conversion.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
struct SignedCoordinate(f64);

impl Coordinate for SignedCoordinate {
    const BITS: u32 = <f64 as Coordinate>::BITS;

    fn zero() -> Self {
        Self(<f64 as Coordinate>::zero())
    }

    fn domain_max(n: u32) -> Self {
        Self(<f64 as Coordinate>::domain_max(n))
    }

    fn midpoint(a: Self, b: Self) -> Self {
        Self(<f64 as Coordinate>::midpoint(a.0, b.0))
    }

    fn width(start: Self, end: Self) -> Self {
        Self(<f64 as Coordinate>::width(start.0, end.0))
    }

    fn is_final(start: Self, end: Self, depth: u32, n: u32) -> bool {
        <f64 as Coordinate>::is_final(start.0, end.0, depth, n)
    }

    fn from_u64(v: u64) -> Self {
        Self(<f64 as Coordinate>::from_u64(v))
    }

    fn next_value(self) -> Self {
        Self(<f64 as Coordinate>::next_value(self.0))
    }

    fn to_f64(self) -> f64 {
        <f64 as Coordinate>::to_f64(self.0)
    }

    fn is_nan(self) -> bool {
        <f64 as Coordinate>::is_nan(self.0)
    }

    fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        <f64 as Coordinate>::total_cmp(&self.0, &other.0)
    }
}

impl CentredBitSource for SignedCoordinate {
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        self.0.to_bits().to_centred_bits(n)
    }
}

/// The count of observations the root tracker was shown in one report.
fn root_sample_count<C: Coordinate>(report: &torrust_sentinel::BatchReport<C>) -> usize {
    report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .find(|cell| cell.depth == 0)
        .expect("the root tracker is permanent and receives every observation")
        .sample_count
}

/// Domain membership is decided by comparison against the domain's own bounds,
/// so a coordinate type the crate does not ship is held to the same domain as
/// the ones it does. Inferring from the bit width that every representable
/// value is in the domain holds only for the unsigned types: at a width that
/// fills a signed or floating type it would admit a value below the origin, a
/// NaN and an infinity, each of which the spatial layer, the encoder and the
/// interval scan would then read differently. The comparison is the root cell's
/// own containment test and the root is permanent, so whatever the boundary
/// admits the partition delivers — which is the mandatory delivery holding for
/// any coordinate type a host may bring, not only for the two shipped here.
///
/// A NaN needs no case of its own. Every comparison with a NaN is false, so it
/// is neither at nor above the origin and both arms of the test refuse it.
///
/// ´claim:routing:domain-membership-is-decided-by-comparison-for-every-coordinate-type´
/// ´test:integration:a-signed-coordinate-outside-the-domain-is-counted-nowhere-at-full-width´
#[test]
fn a_signed_coordinate_outside_the_domain_is_counted_nowhere_at_full_width() {
    // At N = 64 the width fills the coordinate type, which is the shape in
    // which an unsigned coordinate has nothing outside the domain at all. This
    // type has three: below the origin, comparable with nothing, and above
    // every bound.
    let below = SignedCoordinate(-1.0);
    let nan = SignedCoordinate(f64::NAN);
    let above = SignedCoordinate(f64::INFINITY);
    let outside = [below, nan, above];

    let mut s = SpectralSentinel::<SignedCoordinate, u64, 64>::new(test_config()).unwrap();

    let report = s.ingest(&[SignedCoordinate(42.0), below, nan, above]);

    assert_eq!(s.graph().total_sum(), 1, "only the in-domain value moves the graph");
    assert_eq!(
        s.lifetime_observations(),
        1,
        "and only it is counted against the sentinel's lifetime"
    );
    assert_eq!(
        root_sample_count(&report),
        1,
        "the root is shown the in-domain value alone — nothing aliased onto it"
    );

    // A batch of nothing but such values is the same non-event as an empty
    // batch, exactly as it is for an unsigned coordinate below the full width.
    let empty = s.ingest(&outside);

    assert_eq!(s.graph().total_sum(), 1, "the graph total is where the first batch left it");
    assert_eq!(s.lifetime_observations(), 1, "and so is the lifetime count");
    assert!(
        empty.cell_reports.is_empty() && empty.ancestor_reports.is_empty(),
        "no cell was shown anything, so no cell reports"
    );
    assert!(
        empty.oldest_observation_age_micros.is_none(),
        "a batch with no observation in it has no oldest observation to age"
    );

    // The domain's own values are untouched by the test that refuses those:
    // the origin is inside the domain and so is any finite value below the
    // upper bound the coordinate type names.
    let accepted = s.ingest(&[SignedCoordinate(0.0), SignedCoordinate(1e18)]);

    assert_eq!(s.graph().total_sum(), 3, "the origin and an ordinary value are observations");
    assert_eq!(s.lifetime_observations(), 3, "and both are counted");
    assert_eq!(root_sample_count(&accepted), 2, "and both reach the root");
}

/// Below the full width the lower bound is what refuses a coordinate the domain
/// cannot name. The upper bound is a representable value there and stays
/// exclusive for every coordinate type, so a negative is rejected exactly as
/// the first value above the domain already is — a width narrower than the
/// coordinate type is not a case the old reading got right either, because a
/// comparison against the upper bound alone has no lower bound at all.
///
/// (´claim:routing:domain-membership-is-decided-by-comparison-for-every-coordinate-type´)
/// ´test:integration:a-signed-coordinate-below-the-origin-is-counted-nowhere-at-a-narrow-width´
#[test]
fn a_signed_coordinate_below_the_origin_is_counted_nowhere_at_a_narrow_width() {
    // At N = 8 the domain is [0, 256): 256 is the first value above it and −1
    // the first below.
    let mut s = SpectralSentinel::<SignedCoordinate, u64, 8>::new(test_config()).unwrap();

    let report = s.ingest(&[SignedCoordinate(-1.0), SignedCoordinate(42.0), SignedCoordinate(256.0)]);

    assert_eq!(s.graph().total_sum(), 1, "only the in-domain value moves the graph");
    assert_eq!(
        s.lifetime_observations(),
        1,
        "and only it is counted against the sentinel's lifetime"
    );
    assert_eq!(
        root_sample_count(&report),
        1,
        "the value below the origin is refused as the one above the bound is"
    );
}
