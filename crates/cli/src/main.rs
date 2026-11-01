//! `lemusync`: command-line interface. Spec: docs/specs/config-and-cli.md.

mod config_cmd;
mod connection;
mod daemon_client;
mod doctor_desktop;
mod games_cmd;
mod i18n;
mod logs_cmd;
mod output;
mod pause;
mod play;
mod progress;
mod prompt;
mod service;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{CommandFactory as _, FromArgMatches as _, Parser, Subcommand};
use lemusync_api::StreamAction;
use lemusync_engine::{
    BackendConfig, DisabledSecrets, Engine, EngineEnv, EngineError, JoinSecret, Locations,
    MapChoice, MapOutcome, S3Backend, SyncOptions, VersionId,
};

use crate::daemon_client::Api;

use crate::i18n::{I18n, s};

#[derive(Parser)]
#[command(name = "lemusync", version)]
struct Cli {
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    Init,
    Join {
        #[arg(long)]
        recovery: bool,
    },
    Backend {
        #[command(subcommand)]
        cmd: BackendCmd,
    },
    Grant {
        #[command(subcommand)]
        cmd: GrantCmd,
    },
    Players {
        #[command(subcommand)]
        cmd: PlayersCmd,
    },
    Scan,
    Games {
        #[command(subcommand)]
        cmd: Option<GamesCmd>,
    },
    Accounts {
        #[command(subcommand)]
        cmd: AccountsCmd,
    },
    Sync {
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        yes: bool,
    },
    Status,
    History {
        game: String,
        #[arg(long)]
        emu: Option<String>,
        #[arg(long = "as")]
        player: Option<String>,
    },
    Restore {
        game: String,
        what: String,
        #[arg(long)]
        emu: Option<String>,
        #[arg(long = "as")]
        player: Option<String>,
        #[arg(long)]
        yes: bool,
    },
    Doctor,
    Play {
        game: String,
        #[arg(long)]
        emu: Option<String>,
        #[arg(long = "as")]
        player: Option<String>,
        #[arg(last = true)]
        args: Vec<std::ffi::OsString>,
    },
    Conflicts,
    Resolve {
        game: String,
        version: String,
        #[arg(long = "as")]
        player: Option<String>,
    },
    Lease {
        #[command(subcommand)]
        cmd: LeaseCmd,
    },
    Config {
        #[command(subcommand)]
        cmd: config_cmd::ConfigCmd,
    },
    Service {
        #[command(subcommand)]
        cmd: service::ServiceCmd,
    },
    Pause {
        #[arg(long = "for")]
        duration: Option<String>,
    },
    Resume,
    Logs {
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
enum GamesCmd {
    Alias {
        game: String,
        alias: String,
        #[arg(long)]
        emu: Option<String>,
    },
    Unalias {
        alias: String,
    },
}

#[derive(Subcommand)]
enum LeaseCmd {
    Status,
    Release {
        #[arg(long, required = true)]
        force: bool,
        game: String,
    },
}

#[derive(Subcommand)]
enum BackendCmd {
    Add {
        #[command(subcommand)]
        kind: BackendKind,
    },
    Login,
}

#[derive(Subcommand)]
enum BackendKind {
    Gdrive {
        #[arg(long)]
        client_id: String,
    },
    S3 {
        #[arg(long)]
        endpoint: String,
        #[arg(long)]
        bucket: String,
        #[arg(long)]
        access_key_id: String,
        #[arg(long, default_value = "auto")]
        region: String,
    },
    Fs {
        path: PathBuf,
    },
}

#[derive(Subcommand)]
enum GrantCmd {
    Add { path: PathBuf },
    List,
}

#[derive(Subcommand)]
enum PlayersCmd {
    Add { name: String },
    List,
}

#[derive(Subcommand)]
enum AccountsCmd {
    Map {
        player: String,
        emulator: String,
        account: String,
        #[arg(long, conflicts_with = "replace")]
        keep: bool,
        #[arg(long)]
        replace: bool,
    },
    Recover {
        emulator: String,
        account: String,
        #[arg(long)]
        player: String,
    },
    Create {
        player: String,
        emulator: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        yes: bool,
    },
}

/// Sets every command's and argument's help from `help-<path>[-<arg id>]` (P6).
/// clap's own `Usage:`/`Options:` headings and the built-in -h/-V texts stay English (i18n Open questions).
fn localize(cmd: clap::Command, path: &str, t: &I18n) -> clap::Command {
    let key = if path.is_empty() {
        "help-about".to_owned()
    } else {
        format!("help-{path}")
    };
    let mut cmd = match t.get(&key, &[]) {
        Some(text) => cmd.about(text),
        None => cmd,
    };
    let args: Vec<String> = cmd
        .get_arguments()
        .map(|a| a.get_id().to_string())
        .collect();
    for id in args {
        if let Some(text) = t.get(&format!("{key}-{id}"), &[]) {
            cmd = cmd.mut_arg(id, |a| a.help(text));
        }
    }
    let subs: Vec<String> = cmd
        .get_subcommands()
        .map(|c| c.get_name().to_owned())
        .collect();
    for name in subs {
        let sub_path = if path.is_empty() {
            name.clone()
        } else {
            format!("{path}-{name}")
        };
        cmd = cmd.mut_subcommand(&name, |c| localize(c, &sub_path, t));
    }
    cmd
}

fn env() -> Result<EngineEnv, EngineError> {
    let mut env = EngineEnv::system()?;
    if let Some(root) = std::env::var_os("LEMUSYNC_TEST_ROOT") {
        env.locations = Locations::under(Path::new(&root));
        // Tests never reach the user's real keystore (security §Credentials).
        env.secrets = std::sync::Arc::new(DisabledSecrets);
    }
    Ok(env)
}

/// `play <emulator> <game>`, the shape before game references: `play_old_shape` with the
/// new command, or `None`. Only a hint, never run (config-and-cli §Commands).
fn old_play_hint(args: &[String]) -> Option<String> {
    let mut words = args.iter().take_while(|a| *a != "--");
    words
        .find(|a| !a.starts_with('-'))?
        .eq("play")
        .then_some(())?;
    let mut positional = Vec::new();
    let mut skip = false;
    for a in words {
        if std::mem::take(&mut skip) {
            continue;
        }
        if a == "--as" || a == "--emu" {
            skip = true;
        } else if !a.starts_with('-') {
            positional.push(a.as_str());
        }
    }
    let [emu, game, ..] = positional[..] else {
        return None;
    };
    let plain = |w: &str| {
        (1..=64).contains(&w.len())
            && w.bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_'))
    };
    (lemusync_engine::is_emulator(emu) && plain(game))
        .then(|| format!("play_old_shape:lemusync play {game} --emu {emu}"))
}

fn main() -> ExitCode {
    let t = I18n::from_env();
    let matches = match localize(Cli::command(), "", &t).try_get_matches() {
        Ok(m) => m,
        Err(e) => {
            let args: Vec<String> = std::env::args().skip(1).collect();
            if e.kind() == clap::error::ErrorKind::UnknownArgument
                && let Some(hint) = old_play_hint(&args)
            {
                let json = args.iter().any(|a| a == "--json");
                output::error(&t, json, &EngineError::Usage(hint));
                return ExitCode::from(2);
            }
            e.exit()
        }
    };
    let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r,
        Err(e) => {
            output::error(&t, cli.json, &EngineError::Io(e.to_string()));
            return ExitCode::from(1);
        }
    };
    match runtime.block_on(run(&cli, &t)) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            output::error(&t, cli.json, &e);
            ExitCode::from(e.exit_code())
        }
    }
}

async fn gdrive_login(engine: &mut Engine, t: &I18n, client_id: &str) -> Result<(), EngineError> {
    let secret = prompt::secret(t, "prompt-client-secret")?;
    let login = engine.gdrive_login_start(client_id, &secret).await?;
    eprintln!("{}\n\n    {}\n", t.t("login-open-url", &[]), login.url());
    eprintln!("{}", t.t("login-waiting", &[]));
    engine.gdrive_login_finish(login).await?;
    eprintln!("{}", t.t("login-done", &[]));
    Ok(())
}

/// `--as <player>`, or the only player in the repo (the daemon applies the same rule).
async fn player_of(api: &mut Api, given: Option<&String>) -> Result<String, EngineError> {
    if let Some(p) = given {
        return Ok(p.clone());
    }
    pick_player(api.players_list().await?)
}

fn pick_player(players: Vec<String>) -> Result<String, EngineError> {
    lemusync_engine::only_player(players)
}

/// The engine in this process, for the commands that only run here (the Google Drive login).
fn local(api: &mut Api) -> Result<&mut Engine, EngineError> {
    match api {
        Api::Local(e) => Ok(e),
        Api::Daemon(_) => Err(EngineError::Usage("daemon_running_oauth".into())),
    }
}

fn now_ms_u64() -> u64 {
    u64::try_from(now_ms()).unwrap_or(0)
}

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
}

/// `play`, `conflicts`, `resolve` and `lease` (sync-model §Launcher mode, §Conflicts, §Leases).
#[allow(clippy::too_many_lines)] // one match arm per command, each a few lines
async fn run_play(api: &mut Api, cmd: &Cmd, t: &I18n, json: bool) -> Result<u8, EngineError> {
    match cmd {
        Cmd::Play {
            game,
            emu,
            player,
            args,
        } => {
            let (emulator, game) = &api.resolve_game(game, emu.as_deref()).await?;
            let player = player_of(api, player.as_ref()).await?;
            let names = api.game_names().await;
            let p = play::TermPrompt {
                lang: I18n::env_lang(),
                ask: !json && prompt::interactive(),
                events: !json,
                emulator: emulator.clone(),
            };
            if !json {
                eprintln!(
                    "{}",
                    t.t(
                        "play-starting",
                        &[("game", s(games_cmd::label_of(&names, emulator, game)))]
                    )
                );
            }
            let guard = play::hold_ctrl_c(t, !json);
            let report = api.play(&player, emulator, game, args, &p).await;
            guard.abort();
            let report = report?;
            if json {
                output::json(&report);
            } else {
                let label = games_cmd::label_of(&names, emulator, game);
                output::play(t, emulator, &report, &label, output::color_on());
            }
        }
        Cmd::Conflicts => {
            let (list, warnings) = api.conflicts().await?;
            if json {
                output::json(&serde_json::json!({
                    "api_version": lemusync_api::API_VERSION,
                    "conflicts": list,
                    "warnings": warnings,
                }));
            } else {
                output::conflicts(t, &list);
                output::warnings(t, &warnings);
            }
        }
        Cmd::Resolve {
            game,
            version,
            player,
        } => {
            let keep: VersionId = version
                .parse()
                .map_err(|_| EngineError::Usage(format!("version_invalid:{version}")))?;
            let (emulator, game) = api.resolve_game(game, None).await?;
            let (list, _) = api.conflicts().await?;
            let mut found: Vec<_> = list
                .iter()
                .filter(|c| {
                    c.game == game
                        && c.emulator == emulator
                        && player.as_ref().is_none_or(|p| *p == c.player)
                })
                .collect();
            let c = match found.len() {
                0 => return Err(EngineError::Usage(format!("no_conflict:{game}"))),
                1 => found.remove(0),
                _ => return Err(EngineError::Usage("player_needed".into())),
            };
            let (report, warnings) = api.resolve(&c.player, &c.emulator, &game, keep).await?;
            if json {
                output::json(&report);
            } else {
                output::line(
                    t,
                    "resolve-done",
                    &[
                        ("player", s(c.player.clone())),
                        ("game", s(game.clone())),
                        ("version", s(version.clone())),
                    ],
                );
                output::warnings(t, &warnings);
            }
        }
        Cmd::Lease {
            cmd: LeaseCmd::Status,
        } => {
            let (list, warnings) = api.lease_status().await?;
            if json {
                output::json(&serde_json::json!({
                    "api_version": lemusync_api::API_VERSION,
                    "leases": list,
                    "warnings": warnings,
                }));
            } else {
                output::leases(t, &list, now_ms());
                output::warnings(t, &warnings);
            }
        }
        Cmd::Lease {
            cmd: LeaseCmd::Release { game, .. },
        } => {
            let (_, game) = &api.resolve_game(game, None).await?;
            let released = api.lease_release_game(game).await?;
            if json {
                output::json(&serde_json::json!({
                    "api_version": lemusync_api::API_VERSION,
                    "released": released,
                }));
            } else {
                output::line(
                    t,
                    "lease-released",
                    &[
                        ("game", s(game.to_lowercase())),
                        ("count", fluent_bundle::FluentValue::from(released)),
                    ],
                );
            }
        }
        _ => {}
    }
    Ok(0)
}

#[allow(clippy::too_many_lines)] // one match arm per command, each a few lines
async fn run(cli: &Cli, t: &I18n) -> Result<u8, EngineError> {
    let json = cli.json;
    match &cli.command {
        Cmd::Service { cmd } => {
            service::run(cmd, t, json)?;
            return Ok(0);
        }
        Cmd::Logs { follow } => return logs_cmd::run(&env()?.locations.data_dir, *follow).await,
        _ => {}
    }
    let mut api = Api::open(env).await?;
    match (&cli.command, &mut api) {
        // D28: the OAuth loopback listener runs only in this process, never in the daemon,
        // and D35 rules out borrowing the daemon's lock: the daemon has to stop first.
        (
            Cmd::Backend {
                cmd:
                    BackendCmd::Login
                    | BackendCmd::Add {
                        kind: BackendKind::Gdrive { .. },
                    },
            },
            Api::Daemon(_),
        ) => return Err(EngineError::Usage("daemon_running_oauth".into())),
        (Cmd::Pause { .. } | Cmd::Resume, Api::Local(_)) => {
            return Err(EngineError::Usage("daemon_not_running".into()));
        }
        (Cmd::Pause { duration }, Api::Daemon(d)) => {
            let until = match duration {
                Some(text) => Some(now_ms_u64().saturating_add(pause::parse_duration(text)?)),
                None => None,
            };
            d.pause(until).await?;
            pause::done(t, json, until);
            return Ok(0);
        }
        (Cmd::Resume, Api::Daemon(d)) => {
            d.resume().await?;
            pause::resumed(t, json);
            return Ok(0);
        }
        _ => {}
    }
    let mut live = progress::Live::new(json);
    let conn_sink = (!json).then(|| connection::sink(I18n::env_lang(), live.on()));
    match &mut api {
        Api::Local(engine) => {
            engine.set_progress(live.sink());
            if let Some(c) = conn_sink {
                engine.set_connection_sink(c);
            }
        }
        Api::Daemon(d) => {
            let long = matches!(
                cli.command,
                Cmd::Sync { .. } | Cmd::Restore { .. } | Cmd::Accounts { .. } | Cmd::Play { .. }
            );
            if long && !json {
                d.watch_progress(live.sink(), conn_sink).await;
            }
        }
    }
    match &cli.command {
        cmd @ (Cmd::Play { .. } | Cmd::Conflicts | Cmd::Resolve { .. } | Cmd::Lease { .. }) => {
            return run_play(&mut api, cmd, t, json).await;
        }
        Cmd::Config { cmd } => config_cmd::run(&mut api, cmd, t, json).await?,
        Cmd::Service { .. } | Cmd::Logs { .. } | Cmd::Pause { .. } | Cmd::Resume => {}
        Cmd::Init => {
            let pass = prompt::new_passphrase(t)?;
            let (report, recovery) = api.init(&pass).await?;
            eprintln!(
                "{}\n\n    {}\n",
                t.t("init-recovery", &[]),
                recovery.as_str()
            );
            if json {
                output::json(&report);
            } else {
                output::line(
                    t,
                    "init-done",
                    &[
                        ("repo", s(report.repo.clone())),
                        ("device", s(report.device.clone())),
                    ],
                );
                // `SetupIncomplete`: the repo exists but this device must run `join` to finish.
                output::warnings(t, &report.warnings);
            }
        }
        Cmd::Join { recovery } => {
            let secret = if *recovery {
                JoinSecret::Recovery(prompt::secret(t, "prompt-recovery")?.to_string())
            } else {
                JoinSecret::Passphrase(prompt::secret(t, "prompt-passphrase")?.to_string())
            };
            let report = api.join(secret).await?;
            if json {
                output::json(&report);
            } else {
                output::line(
                    t,
                    "join-done",
                    &[
                        ("device", s(report.device.clone())),
                        ("repo", s(report.repo.clone())),
                    ],
                );
            }
        }
        Cmd::Backend {
            cmd: BackendCmd::Add {
                kind: BackendKind::Fs { path },
            },
        } => {
            api.backend_add_fs(path).await?;
            output::line(
                t,
                "backend-fs-done",
                &[("path", s(path.display().to_string()))],
            );
        }
        Cmd::Backend {
            cmd:
                BackendCmd::Add {
                    kind: BackendKind::Gdrive { client_id },
                },
        } => {
            gdrive_login(local(&mut api)?, t, client_id).await?;
        }
        Cmd::Backend {
            cmd:
                BackendCmd::Add {
                    kind:
                        BackendKind::S3 {
                            endpoint,
                            bucket,
                            access_key_id,
                            region,
                        },
                },
        } => {
            let secret = prompt::secret(t, "prompt-s3-secret")?;
            let backend = S3Backend {
                endpoint: endpoint.clone(),
                bucket: bucket.clone(),
                region: region.clone(),
                access_key_id: access_key_id.clone(),
            };
            api.backend_add_s3(backend, &secret).await?;
            output::line(
                t,
                "backend-s3-done",
                &[
                    ("bucket", s(bucket.clone())),
                    ("endpoint", s(endpoint.clone())),
                ],
            );
        }
        Cmd::Backend {
            cmd: BackendCmd::Login,
        } => {
            let engine = local(&mut api)?;
            let Some(BackendConfig::Gdrive { client_id, .. }) =
                engine.local_config().backend.clone()
            else {
                return Err(EngineError::NotSetUp("gdrive_backend"));
            };
            gdrive_login(engine, t, &client_id).await?;
        }
        Cmd::Grant {
            cmd: GrantCmd::Add { path },
        } => {
            let granted = api.grant_add(path).await?;
            output::line(
                t,
                "grant-done",
                &[("path", s(granted.display().to_string()))],
            );
        }
        Cmd::Grant {
            cmd: GrantCmd::List,
        } => {
            let grants = api.grant_list().await?;
            if json {
                output::json(&grants);
            } else if grants.is_empty() {
                output::line(t, "grant-none", &[]);
            } else {
                for g in grants {
                    output::line(t, "grant-done", &[("path", s(g))]);
                }
            }
        }
        Cmd::Players {
            cmd: PlayersCmd::Add { name },
        } => {
            let warnings = api.players_add(name).await?;
            output::line(t, "players-added", &[("name", s(name.clone()))]);
            output::warnings(t, &warnings);
        }
        Cmd::Players {
            cmd: PlayersCmd::List,
        } => {
            let players = api.players_list().await?;
            if json {
                output::json(&players);
            } else if players.is_empty() {
                output::line(t, "players-none", &[]);
            } else {
                for p in players {
                    println!("{p}");
                }
            }
        }
        Cmd::Scan => {
            let report = api.scan().await?;
            if json {
                output::json(&report);
            } else {
                output::scan(t, &report);
            }
        }
        Cmd::Accounts {
            cmd:
                AccountsCmd::Map {
                    player,
                    emulator,
                    account,
                    keep,
                    replace,
                },
        } => {
            let choice = if *replace {
                MapChoice::Replace
            } else if *keep {
                MapChoice::Keep
            } else {
                MapChoice::Ask
            };
            let mut outcome = api.accounts_map(player, emulator, account, choice).await?;
            if let MapOutcome::NeedsChoice { games } = &outcome {
                let games = games.join(", ");
                if json || !prompt::interactive() {
                    return Err(EngineError::Usage(
                        t.t("error-needs-choice", &[("games", s(games))]),
                    ));
                }
                let question = t.t(
                    "prompt-keep-or-replace",
                    &[
                        ("account", s(account.clone())),
                        ("games", s(games.clone())),
                        ("player", s(player.clone())),
                    ],
                );
                let choice = match prompt::keep_or_replace(t, &question)? {
                    Some(true) => MapChoice::Replace,
                    Some(false) => MapChoice::Keep,
                    None => {
                        return Err(EngineError::Usage(
                            t.t("error-needs-choice", &[("games", s(games))]),
                        ));
                    }
                };
                outcome = api.accounts_map(player, emulator, account, choice).await?;
            }
            if matches!(outcome, MapOutcome::Mapped) {
                output::line(
                    t,
                    "map-done",
                    &[
                        ("player", s(player.clone())),
                        ("emulator", s(emulator.clone())),
                        ("account", s(account.clone())),
                    ],
                );
            }
        }
        Cmd::Accounts {
            cmd:
                AccountsCmd::Create {
                    player,
                    emulator,
                    name,
                    yes,
                },
        } => {
            if !*yes {
                let shown = name.clone().unwrap_or_else(|| player.clone());
                let question = t.t(
                    "prompt-create-account",
                    &[
                        ("emulator", s(emulator.clone())),
                        ("name", s(shown)),
                        ("player", s(player.clone())),
                    ],
                );
                if json || !prompt::yes(t, &question)? {
                    return Err(EngineError::Usage(
                        t.t("error-needs-tty", &[("what", s(question))]),
                    ));
                }
            }
            let created = live
                .run(t, api.create_account(player, emulator, name.as_deref()))
                .await?;
            if json {
                output::json(&created);
            } else {
                output::line(
                    t,
                    "create-done",
                    &[
                        ("emulator", s(emulator.clone())),
                        ("name", s(created.name.clone())),
                        ("account", s(created.account.clone())),
                        ("player", s(player.clone())),
                    ],
                );
            }
        }
        Cmd::Accounts {
            cmd:
                AccountsCmd::Recover {
                    emulator,
                    account,
                    player,
                },
        } => {
            let report = live
                .run(t, api.accounts_recover(emulator, account, player))
                .await?;
            if json {
                output::json(&report);
            } else {
                output::line(
                    t,
                    "recover-done",
                    &[
                        ("player", s(player.clone())),
                        ("game", s(report.game.clone())),
                    ],
                );
            }
        }
        Cmd::Sync { dry_run, yes } => {
            let opts = || SyncOptions { dry_run: *dry_run };
            let started = std::time::Instant::now();
            let report = match live.run(t, api.sync(opts())).await {
                Err(EngineError::NeedsConfirmation) => {
                    let ok = *yes
                        || (!json
                            && prompt::interactive()
                            && prompt::yes(t, &t.t("prompt-first-sync", &[]))?);
                    if !ok {
                        return Err(EngineError::NeedsConfirmation);
                    }
                    api.confirm_first_sync().await?;
                    live.run(t, api.sync(opts())).await?
                }
                other => other?,
            };
            let conflicts = report
                .streams
                .iter()
                .filter(|st| {
                    matches!(
                        st.action,
                        StreamAction::Conflict | StreamAction::UploadThenConflict
                    )
                })
                .count();
            if json {
                output::json(&report);
            } else {
                output::sync(t, &report, output::color_on(), Some(started.elapsed()));
                if conflicts > 0 {
                    output::line(
                        t,
                        "sync-conflicts",
                        &[("count", fluent_bundle::FluentValue::from(conflicts))],
                    );
                }
            }
            if conflicts > 0 {
                return Ok(4);
            }
        }
        Cmd::Games { cmd: None } => {
            let report = api.games_report().await?;
            if json {
                output::json(&report);
            } else {
                games_cmd::print(t, &report);
            }
        }
        Cmd::Games {
            cmd: Some(GamesCmd::Alias { game, alias, emu }),
        } => {
            let (emulator, key) = api.alias_add(game, emu.as_deref(), alias).await?;
            let names = api.game_names().await;
            if json {
                output::json(&serde_json::json!({
                    "api_version": lemusync_api::API_VERSION,
                    "alias": alias,
                    "emulator": emulator,
                    "key": key,
                }));
            } else {
                output::line(
                    t,
                    "games-alias-done",
                    &[
                        ("alias", s(alias.clone())),
                        ("game", s(games_cmd::label_of(&names, &emulator, &key))),
                    ],
                );
            }
        }
        Cmd::Games {
            cmd: Some(GamesCmd::Unalias { alias }),
        } => {
            api.alias_remove(alias).await?;
            if json {
                output::json(&serde_json::json!({
                    "api_version": lemusync_api::API_VERSION,
                    "removed": alias,
                }));
            } else {
                output::line(t, "games-unalias-done", &[("alias", s(alias.clone()))]);
            }
        }
        Cmd::Status => {
            let report = api.status().await?;
            if json {
                output::json(&report);
            } else {
                output::status(t, &report);
            }
        }
        Cmd::History { game, emu, player } => {
            let (emulator, game) = &api.resolve_game(game, emu.as_deref()).await?;
            let player = &player_of(&mut api, player.as_ref()).await?;
            let report = api.history(player, emulator, game).await?;
            if json {
                output::json(&report);
            } else {
                output::history(t, &report, &game.to_lowercase());
            }
        }
        Cmd::Restore {
            game,
            what,
            emu,
            player,
            yes,
        } => {
            let (emulator, game) = &api.resolve_game(game, emu.as_deref()).await?;
            let player = &player_of(&mut api, player.as_ref()).await?;
            if !*yes {
                let question = t.t(
                    "prompt-restore",
                    &[("player", s(player.clone())), ("what", s(what.clone()))],
                );
                if json || !prompt::yes(t, &question)? {
                    return Err(EngineError::Usage(
                        t.t("error-needs-tty", &[("what", s(question))]),
                    ));
                }
            }
            let report = live
                .run(t, api.restore(player, emulator, game, what))
                .await?;
            if json {
                output::json(&report);
            } else {
                output::line(
                    t,
                    "restore-done",
                    &[("what", s(what.clone())), ("player", s(player.clone()))],
                );
            }
        }
        Cmd::Doctor => {
            let mut report = api.doctor().await?;
            let version = api.daemon().map(|d| d.version().to_owned());
            report
                .checks
                .extend(doctor_desktop::checks(version.as_deref()).await);
            if json {
                output::json(&report);
            } else {
                output::doctor(t, &report);
            }
            if report.checks.iter().any(|c| !c.ok) {
                return Ok(1);
            }
        }
    }
    Ok(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_player_is_the_only_one_else_which_player() {
        assert_eq!(pick_player(vec!["Ana".into()]).unwrap(), "Ana");
        for players in [vec![], vec!["Ana".to_owned(), "Bea".to_owned()]] {
            assert!(
                matches!(pick_player(players), Err(EngineError::Usage(k)) if k == "player_needed")
            );
        }
    }
}
