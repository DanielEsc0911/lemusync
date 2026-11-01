//! Engine-side budgets (reliability §Budgets): cold start to ready, warm rescan of 10 000
//! files (an unchanged save folder, dry-run sync), idle RSS of an open engine, and, through
//! `Service` (the daemon's loop), a save write to the start of its upload. Run with `just bench`
//! (release); CI's `budgets` job compares the rescan time with the spec (`cargo xtask
//! budgets-check`).
#![allow(clippy::unwrap_used)] // bench setup: a failure here is a failed run

#[path = "../tests/common/mod.rs"]
mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use common::{FileSecrets, Pc, SAVE, env_with, map};
use criterion::{Criterion, criterion_group, criterion_main};
use lemusync_api::{Event, TransferPhase};
use lemusync_engine::{
    Engine, FakeProbe, MapChoice, SecretStore, Service, ServiceConfig, SyncOptions,
};

const FILES: usize = 10_000;
/// Quiet period for the latency run; the budget is this plus at most 2 s.
const QUIET: Duration = Duration::from_secs(1);
const WRITES: usize = 5;

/// Resident set size of this process in MiB, from `/proc/self/statm` (Linux only).
fn rss_mib() -> Option<f64> {
    let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
    let pages: f64 = statm.split_whitespace().nth(1)?.parse().ok()?;
    Some(pages * 4096.0 / 1_048_576.0)
}

fn budgets(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("a");
    let mut pc = Pc::new(&root, true, b"save");
    let many = pc.cemu.join(SAVE).join("80000001/0/many");
    std::fs::create_dir_all(&many).unwrap();
    for i in 0..FILES - 1 {
        std::fs::write(many.join(format!("f{i:05}.bin")), i.to_le_bytes()).unwrap();
    }
    rt.block_on(async {
        pc.engine
            .backend_add_fs(&dir.path().join("remote"))
            .unwrap();
        pc.engine.init("pass phrase").await.unwrap();
        pc.engine.players_add("Ana").await.unwrap();
        map(&mut pc, "Ana", "80000001", MapChoice::Ask).await;
        let r = pc.sync().await;
        assert!(
            r.streams[0].files >= u32::try_from(FILES).unwrap(),
            "all files uploaded"
        );
    });
    let Pc { engine, .. } = pc;
    drop(engine);

    let secrets: Arc<dyn SecretStore> = Arc::new(FileSecrets::under(&root));
    let env = || env_with(&root, Arc::new(FakeProbe::default()), secrets.clone());
    // Ready = state DB open, repo unlocked from the keystore, one remote read answered.
    c.bench_function("cold_start_to_ready", |b| {
        b.iter(|| {
            let mut e = Engine::open(env()).unwrap();
            assert_eq!(rt.block_on(e.players_list()).unwrap().len(), 1);
        });
    });

    let mut engine = Engine::open(env()).unwrap();
    if let Some(mib) = rss_mib() {
        eprintln!("idle RSS of the bench process with an open engine: {mib:.1} MiB");
    }
    c.bench_function("rescan_10k_files_warm", |b| {
        b.iter(|| {
            let r = rt
                .block_on(engine.sync(SyncOptions { dry_run: true }))
                .unwrap();
            assert_eq!(r.streams.len(), 1);
        });
    });
    change_to_upload(&rt, engine, &pc_save(&root));
}

fn pc_save(root: &std::path::Path) -> std::path::PathBuf {
    root.join("home/.var/app/info.cemu.Cemu/data/Cemu")
        .join(SAVE)
        .join("80000001/0/game_data.sav")
}

/// Change detected → upload started, through `Service` with the default timings but a 1 s
/// quiet period: the time from a save write to the first `Uploading` progress event, minus
/// the quiet period. Printed, not a criterion bench (each sample waits out the quiet period).
fn change_to_upload(rt: &tokio::runtime::Runtime, engine: Engine, file: &std::path::Path) {
    let cfg = ServiceConfig {
        quiet: QUIET,
        ..ServiceConfig::default()
    };
    let (service, handle) = Service::new(engine, cfg);
    let thread = service.spawn().unwrap();
    let mut over = rt.block_on(async {
        // The first sync at start, then the watcher is registered.
        let mut rx = handle.subscribe();
        while handle.status().last_sync_ms.is_none() {
            let _ = tokio::time::timeout(Duration::from_millis(100), rx.recv()).await;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
        let mut over = Vec::new();
        for i in 0..WRITES {
            let mut rx = handle.subscribe();
            let t0 = Instant::now();
            std::fs::write(file, format!("change {i}")).unwrap();
            loop {
                let left = Duration::from_secs(30).saturating_sub(t0.elapsed());
                let e = tokio::time::timeout(left, rx.recv()).await.unwrap();
                if let Ok(Event::TransferProgress(p)) = e
                    && p.phase == TransferPhase::Uploading
                {
                    break;
                }
            }
            over.push(t0.elapsed().saturating_sub(QUIET));
            // Let that sync finish before the next write.
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
        handle.stop().await;
        over
    });
    drop(thread.join());
    over.sort();
    eprintln!(
        "change detected -> upload started: quiet period + median {:.0} ms, max {:.0} ms ({WRITES} writes)",
        over[WRITES / 2].as_secs_f64() * 1000.0,
        over[WRITES - 1].as_secs_f64() * 1000.0
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = budgets
}
criterion_main!(benches);
