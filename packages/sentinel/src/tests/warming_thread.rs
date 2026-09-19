// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for the background warming thread's shutdown handshake.
//!
//! The worker sleeps on a condition variable whose predicate is two facts —
//! whether the staging area holds warming work, and whether shutdown has been
//! asked for — and it holds the staging mutex from the moment it reads them
//! until the wait releases it. A writer that changes either fact outside that
//! mutex can place the change and its wake-up inside that window, where the
//! wake-up reaches a thread that has not yet begun to wait; the worker then
//! sleeps on a predicate that has already changed and nothing changes it
//! again. Shutdown is the transition where that costs a hang rather than a
//! delay, because the joining thread waits for a worker that will never look
//! at the flag again.
//!
//! The test here cannot make the window open on demand: it is a few
//! instructions wide and the scheduler decides. What it can do is take the
//! bet often enough that a lost wake-up shows up as a thread that never
//! finishes, and bound the wait so the failure arrives as a failed assertion
//! rather than as a suite that stops.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`shutdown_returns_under_repeated_spawn_and_stop_cycles`] | warmup | Shutting the warming thread down returns, every time, over a long run of spawn-and-stop cycles that does nothing else — the arrangement that puts the request at its most likely to land while the worker is between reading its predicate and sleeping on it. A shutdown that is lost in that window does not fail loudly: the worker sleeps on, the join waits for it, and the sentinel's own drop never completes, so what a host would see is a process that stops rather than an error it can act on. |
//! | [`dropping_a_sentinel_consumes_a_failed_worker_join`] | warmup | A warming worker can fail before its owner is destroyed. Destruction still completes without unwinding, because the drop path records the failed join instead of turning a background failure into a destructor panic. |
//! | [`reset_consumes_a_failed_worker_join`] | warmup | Reset follows the same host-preserving policy as destruction: a worker that has already failed is joined and recorded, then reset rebuilds the sentinel instead of panicking over a failure that happened in the background. |
//! | [`a_failed_warming_worker_still_brings_every_cell_online`] | warmup | A sentinel whose warming worker has died still brings every cell the selector pays for online: the cell the worker was holding when it went, and every cell staged afterwards. The dispatch keys on whether a worker is present, so a dead worker left standing would send every later reconciliation down the background branch — past the synchronous drain, into a notification nobody receives — and the engine would go on issuing reports from a cell set that had stopped growing. |
//! | [`selection_refresh_survives_warming_handoffs`] | warmup | Waiting, in-flight and ready cells keep the latest selection flag across both worker return paths. |

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::NoiseSchedule;
use crate::sentinel::staging::StagingArea;
use crate::sentinel::warming_thread::WarmingThreadHandle;
use crate::{SentinelConfig, SpectralSentinel};

/// How many spawn-and-stop cycles the witness runs.
const CYCLES: usize = 1_000;

/// How long the witness waits for those cycles before calling the handshake
/// broken. Well inside the package's five-second per-test budget, and orders
/// of magnitude above the cycles' own cost.
const DEADLINE: Duration = Duration::from_secs(3);

// ─── Shutdown handshake ─────────────────────────────────────

/// Shutting the warming thread down returns, every time, over a long run of
/// spawn-and-stop cycles that does nothing else — the arrangement that puts
/// the request at its most likely to land while the worker is between reading
/// its predicate and sleeping on it. A shutdown that is lost in that window
/// does not fail loudly: the worker sleeps on, the join waits for it, and the
/// sentinel's own drop never completes, so what a host would see is a process
/// that stops rather than an error it can act on.
///
/// ´claim:warmup:shutting-the-warming-thread-down-returns-however-the-request-races-the-worker-going-to-sleep´
/// ´test:crate:shutdown-returns-under-repeated-spawn-and-stop-cycles´
#[test]
fn shutdown_returns_under_repeated_spawn_and_stop_cycles() {
    let (done, finished) = mpsc::channel();

    // The cycles run on their own thread so that a lost wake-up is a
    // deadline this thread can observe. Joining them directly would make
    // the failure a hang, which no assertion can report.
    let cycles = std::thread::spawn(move || {
        for _ in 0..CYCLES {
            let staging = Arc::new(Mutex::new(StagingArea::<u128>::new()));
            let handle = WarmingThreadHandle::spawn(&staging, 4, Some(7)).expect("the environment must grant a warming thread");
            handle.shutdown();
        }
        // The receiver is gone only if the witness has already reported the
        // deadline, so there is nothing for this thread to do about it.
        let _reported = done.send(());
    });

    match finished.recv_timeout(DEADLINE) {
        Ok(()) => {}
        Err(RecvTimeoutError::Timeout) => {
            panic!(
                "{CYCLES} spawn-and-stop cycles did not finish within {DEADLINE:?}: a shutdown request was \
                 stored and notified while the worker was between reading its predicate and sleeping on it, \
                 so the worker never saw it and the join never returned"
            )
        }
        Err(RecvTimeoutError::Disconnected) => panic!("the cycling thread ended without reporting"),
    }

    cycles.join().expect("cycling thread panicked");
}

/// A warming worker can fail before its owner is destroyed. Destruction
/// still completes without unwinding, because the drop path records the failed
/// join instead of turning a background failure into a destructor panic.
///
/// ´claim:warmup:sentinel-destruction-consumes-a-failed-worker-join´
/// ´test:crate:dropping-a-sentinel-consumes-a-failed-worker-join´
#[test]
fn dropping_a_sentinel_consumes_a_failed_worker_join() {
    let config = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(Vec::new()),
        background_warming: true,
        ..SentinelConfig::<u64>::default()
    };
    let sentinel = SpectralSentinel::<u128, u64, 128>::new(config).unwrap();
    sentinel.fail_warming_worker_for_test();

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(sentinel)));

    assert!(outcome.is_ok(), "dropping the sentinel must consume the failed worker join");
}

/// Reset follows the same host-preserving policy as destruction: a worker
/// that has already failed is joined and recorded, then reset rebuilds the
/// sentinel instead of panicking over a failure that happened in the
/// background.
///
/// ´claim:warmup:sentinel-reset-consumes-a-failed-worker-join´
/// ´test:crate:reset-consumes-a-failed-worker-join´
#[test]
fn reset_consumes_a_failed_worker_join() {
    let config = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(Vec::new()),
        background_warming: true,
        ..SentinelConfig::<u64>::default()
    };
    let mut sentinel = SpectralSentinel::<u128, u64, 128>::new(config).unwrap();
    sentinel.fail_warming_worker_for_test();

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| sentinel.reset()));

    assert!(outcome.is_ok(), "reset must consume the failed worker join");
}

/// A sentinel whose warming worker has died still brings every cell the
/// selector pays for online: the cell the worker was holding when it went, and
/// every cell staged afterwards. The dispatch keys on whether a worker is
/// present, so a dead worker left standing would send every later
/// reconciliation down the background branch — past the synchronous drain, into
/// a notification nobody receives — and the engine would go on issuing reports
/// from a cell set that had stopped growing.
///
/// ´claim:warmup:a-failed-warming-worker-still-brings-every-cell-online´
/// ´test:crate:a-failed-warming-worker-still-brings-every-cell-online´
#[test]
fn a_failed_warming_worker_still_brings_every_cell_online() {
    let config = SentinelConfig::<u64> {
        analysis_k: 1,
        split_threshold: 1,
        d_create: 1,
        d_evict: 2,
        max_rank: 1,
        noise_batch_size: 1,
        noise_schedule: NoiseSchedule::Explicit(vec![0, 2]),
        background_warming: false,
        noise_seed: Some(7),
        ..SentinelConfig::default()
    };
    let mut sentinel = SpectralSentinel::<u64, u64, 16>::new(config).expect("the test configuration must be valid");

    // The state the strand needs — a cell below the root that is online, so
    // that killing a worker can take it off the producing set — is built here
    // rather than waited for. The analysis set is a function of the value
    // stream alone, so these eight batches decide which cells the selector
    // names on every machine; what differs between the two warming modes is
    // only whether those cells are online yet. With no worker the drain runs
    // inside the reconciliation, so each call returns with every cell the
    // schedule asks for already promoted, where a worker would have brought
    // them online whenever it was next scheduled — and on a machine with few
    // cores, that can be after this loop.
    for step in 0..8_u64 {
        sentinel.ingest(&[step * 4_096]);
    }

    // The worker exists from here, so the strand below is a real one: the
    // failure it induces runs through the worker's own poisoned-staging path,
    // and the recovery measured afterwards is the one that follows a worker
    // that died holding a cell.
    sentinel.start_a_warming_worker_for_test();

    let stranded = sentinel.strand_a_cell_on_a_failed_warming_worker_for_test();
    assert!(
        !sentinel.cell_gnodes().contains(&stranded),
        "the stranded cell must start off the producing set, or the witness proves nothing"
    );

    // One ingest is the whole recovery: the reconciliation it runs is where the
    // dead worker is reaped, where the checkout record it left is dropped, and
    // where the drain that replaces it runs.
    sentinel.ingest(&[0]);

    assert!(
        sentinel.cell_gnodes().contains(&stranded),
        "the cell the worker was holding when it died must be built again and brought online, \
         not left stranded between the producing set and a checkout record nothing will redeem"
    );

    // Cells staged after the worker died have no one to warm them but the
    // synchronous drain, which only runs if the dispatch can see that the
    // worker is gone.
    for step in 8..16_u64 {
        sentinel.ingest(&[step * 4_096]);
    }

    let producing = sentinel.cell_gnodes();
    let waiting: Vec<_> = sentinel
        .analysis_set()
        .full()
        .iter()
        .map(|entry| entry.gnode)
        .filter(|gnode| !producing.contains(gnode))
        .collect();
    assert!(
        waiting.is_empty(),
        "every cell the selector pays for must be online after the worker died, but {waiting:?} are still waiting"
    );
    assert_eq!(
        sentinel.health().warming_trackers,
        0,
        "no cell may be left in the staging area once the synchronous drain has taken over"
    );
}

/// Waiting, in-flight and ready cells keep the latest selection flag across both worker return paths.
///
/// ´claim:warmup:selection-refresh-survives-warming-handoffs´
/// ´test:crate:selection-refresh-survives-warming-handoffs´
#[test]
fn selection_refresh_survives_warming_handoffs() {
    let config = crate::SentinelConfig::<u64>::default();
    let cell = crate::sentinel::CellState {
        tracker: crate::sentinel::tracker::SubspaceTracker::new(4, &config, config.cusum_slow_decay),
        depth: 1,
        width: 4,
        start: 0_u128,
        end: 16,
        is_competitive: false,
    };
    let gnode = torrust_mudlark::GNodeId::from_parts(1, 0);
    let mut staging = StagingArea::new();
    staging.enqueue(gnode, cell, 2);
    staging.update_competitive(gnode, true);
    assert_eq!(staging.warming_competitive_count(), 1);

    let (_, warming) = staging.take_highest_priority().unwrap();
    staging.update_competitive(gnode, false);
    assert_eq!(staging.warming_competitive_count(), 0);
    staging.return_warming(gnode, warming);
    assert_eq!(staging.warming_competitive_count(), 0);

    let (_, warming) = staging.take_highest_priority().unwrap();
    assert!(!warming.cell.is_competitive);
    staging.update_competitive(gnode, true);
    assert_eq!(staging.warming_competitive_count(), 1);
    staging.finish_warming(gnode, warming.cell);
    let (_, ready) = staging.take_ready().pop().unwrap();
    assert!(ready.is_competitive);

    staging.enqueue(gnode, ready, 0);
    staging.update_competitive(gnode, false);
    assert!(!staging.take_ready().pop().unwrap().1.is_competitive);
}
