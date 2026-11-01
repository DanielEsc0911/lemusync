//! `lemusyncd [--headless]`: the always-on sync daemon. Spec: docs/specs/architecture.md
//! §Process model, docs/specs/reliability.md §Always on.
//!
//! Order matters: the engine lock first (one engine per user profile, B3), then logging, the
//! IPC socket, the service thread and, unless `--headless`, the tray, notifications and
//! sleep hooks. `SIGTERM`/`SIGINT` stop it gracefully: the running step finishes (or stops
//! at its next await), leases are released, the socket is unlinked, exit 0.

use std::path::Path;
use std::process::ExitCode;

use lemusync_daemon::i18n::{I18n, s};
use lemusync_daemon::logging;
use lemusync_daemon::run::{Desktop, run};
use lemusync_engine::{
    DisabledSecrets, Engine, EngineEnv, EngineError, Locations, Service, ServiceConfig,
};

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("lemusyncd {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }
    let headless = args.iter().any(|a| a == "--headless");
    if let Some(bad) = args.iter().find(|a| *a != "--headless") {
        eprintln!("usage: lemusyncd [--headless] [--version] (unknown argument {bad})");
        return ExitCode::from(2);
    }
    let t = I18n::from_env();
    match start(headless, &t) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("{msg}");
            tracing::error!("{msg}");
            ExitCode::FAILURE
        }
    }
}

/// `LEMUSYNC_TEST_ROOT` puts config, data and the socket under one folder, as for the CLI.
fn env() -> Result<EngineEnv, EngineError> {
    let mut env = EngineEnv::system()?;
    if let Some(root) = std::env::var_os("LEMUSYNC_TEST_ROOT").filter(|r| !r.is_empty()) {
        env.locations = Locations::under(Path::new(&root));
        // Tests never reach the user's real keystore (security §Credentials).
        env.secrets = std::sync::Arc::new(DisabledSecrets);
    }
    Ok(env)
}

fn failed(t: &I18n, e: &dyn std::fmt::Display) -> String {
    t.t(
        "daemon-start-failed",
        &[("error", s(lemusync_engine::redact(&e.to_string())))],
    )
}

fn start(headless: bool, t: &I18n) -> Result<(), String> {
    let env = env().map_err(|e| failed(t, &e))?;
    let data_dir = env.locations.data_dir.clone();
    let engine = match Engine::open(env) {
        Ok(e) => e,
        Err(EngineError::Locked(path)) => {
            return Err(t.t("daemon-already-running", &[("path", s(path))]));
        }
        Err(e) => return Err(failed(t, &e)),
    };
    let log = logging::init(&data_dir).map_err(|e| failed(t, &e))?;
    tracing::info!(
        "lemusyncd {} starting (headless: {headless}, log {})",
        env!("CARGO_PKG_VERSION"),
        log.display()
    );
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| failed(t, &e))?;
    let engine_thread = rt.block_on(async {
        let path = lemusync_ipc::socket::default_path().map_err(|e| failed(t, &e))?;
        let server = lemusync_ipc::Server::bind(&path).map_err(|e| failed(t, &e))?;
        let (service, handle) = Service::new(engine, ServiceConfig::default());
        let thread = service.spawn().map_err(|e| failed(t, &e))?;
        let desktop = (!headless).then(Desktop::start);
        run(handle, server, desktop, I18n::from_env(), signals()).await;
        Ok::<_, String>(thread)
    })?;
    // The engine (and its state lock) goes once the service thread ends.
    drop(engine_thread.join());
    tracing::info!("lemusyncd stopped");
    Ok(())
}

/// Resolves on `SIGTERM` or `SIGINT`.
async fn signals() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let Ok(mut term) = signal(SignalKind::terminate()) else {
            let _ = tokio::signal::ctrl_c().await;
            return;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = tokio::signal::ctrl_c() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
