//! Conflicts: list them and write resolution versions (sync-model §Conflicts).

use std::collections::BTreeMap;
use std::path::Path;

use lemusync_api::{ConflictInfo, HeadInfo, StreamAction, StreamReport, Warning};
use lemusync_core::heads::{Graph, heads};
use lemusync_core::manifest::Version;
use lemusync_core::reconcile::resolution_parents;
use lemusync_core::{PlayerId, StreamId, VersionId};

use crate::progress::Meter;
use crate::query::Target;
use crate::restore::{self, Ctx, Source};
use crate::sync::on_disk;
use crate::upload::{UploadMeta, commit_manifest};
use crate::{ConflictPolicy, Engine, EngineError, StreamRow, repo_config};

/// One stream to resolve (`Engine::write_resolution`).
pub(crate) struct Resolution<'a> {
    pub stream: StreamId,
    pub player: PlayerId,
    pub emulator: &'a str,
    pub game: &'a str,
    pub folder: &'a Path,
    pub names: &'a [String],
    pub max_file_bytes: u64,
    pub versions: &'a BTreeMap<VersionId, Version>,
    pub keep: VersionId,
    pub meter: &'a Meter,
}

fn graph_of(versions: &BTreeMap<VersionId, Version>) -> Graph {
    versions
        .iter()
        .map(|(id, v)| (*id, v.parents.clone()))
        .collect()
}

impl Engine {
    /// Sets `conflict_policy` in `config.toml`.
    ///
    /// # Errors
    /// `Io`/`Config` if the config can't be saved.
    pub fn set_conflict_policy(&mut self, policy: ConflictPolicy) -> Result<(), EngineError> {
        self.config.conflict_policy = policy;
        self.save_config()
    }

    /// Every stream with more than one head, with what the player needs to choose.
    ///
    /// # Errors
    /// Storage and crypto errors; unreadable versions are warnings.
    pub async fn conflicts(&mut self) -> Result<(Vec<ConflictInfo>, Vec<Warning>), EngineError> {
        self.ensure_repo().await?;
        let mut warnings = Vec::new();
        let config = repo_config::load(self.repo_ref()?, &self.state)
            .await?
            .config;
        let mut out = Vec::new();
        let names = self.game_names();
        for (pid, p) in &config.players {
            for (emulator, game) in config.games_of(*pid) {
                let stream = self.repo_ref()?.stream_id(*pid, &emulator, &game);
                let versions = self.fetch_graph(&stream, &mut warnings).await?;
                let hs = heads(&graph_of(&versions));
                if hs.len() < 2 {
                    continue;
                }
                let heads = hs
                    .iter()
                    .filter_map(|h| versions.get(h))
                    .map(|v| HeadInfo {
                        version: v.id.to_string(),
                        device_name: if v.device == self.device_id() {
                            self.config.device.name.clone()
                        } else {
                            v.device.to_string()
                        },
                        player: config
                            .players
                            .get(&v.player)
                            .map_or_else(|| v.player.0.to_string(), |p| p.name.clone()),
                        created_at_ms: i64::try_from(v.created_at).unwrap_or(i64::MAX),
                        files: u32::try_from(v.files.len()).unwrap_or(u32::MAX),
                        bytes: v.files.iter().map(|f| f.size).sum(),
                        changed_files: v.meta.changed_files,
                    })
                    .collect();
                out.push(ConflictInfo {
                    player: p.name.clone(),
                    name: crate::games::name_in(&names, &emulator, &game),
                    emulator,
                    game,
                    heads,
                });
            }
        }
        Ok((out, warnings))
    }

    /// Resolves `player`'s conflict on `game` to the head `keep`: writes a resolution version
    /// (the chosen head's files, parents = all heads), then restores it here with a backup of
    /// the current files first (I1). The other heads stay in history.
    ///
    /// # Errors
    /// `Usage` if the stream has no conflict or `keep` isn't one of its heads, or for an
    /// unknown player, mapping or install; restore, storage and crypto errors.
    pub async fn resolve(
        &mut self,
        player: &str,
        emulator: &str,
        game: &str,
        keep: VersionId,
    ) -> Result<(StreamReport, Vec<Warning>), EngineError> {
        self.recover_journal()?;
        let Target {
            stream,
            player: pid,
            account,
            profile,
            folder,
            names,
        } = self.target(player, emulator, game).await?;
        let game = game.to_lowercase();
        let mut warnings = Vec::new();
        let versions = self.fetch_graph(&stream, &mut warnings).await?;
        let meter = Meter::new(self.progress.clone(), player, emulator, &game);
        let v = self
            .write_resolution(&Resolution {
                stream,
                player: pid,
                emulator,
                game: &game,
                folder: &folder,
                names: &names,
                max_file_bytes: profile.max_file_bytes(),
                versions: &versions,
                keep,
                meter: &meter,
            })
            .await?;
        let (files, bytes) = meter.transferred();
        Ok((
            StreamReport {
                player: player.to_owned(),
                emulator: emulator.to_owned(),
                game,
                account: Some(account),
                action: StreamAction::Resolved,
                version: Some(v.id.to_string()),
                heads: vec![v.id.to_string()],
                done: true,
                files,
                bytes,
            },
            warnings,
        ))
    }

    /// Commits the resolution manifest (reusing `keep`'s blobs: nothing is re-uploaded), then
    /// restores it with backup and records it as `base`. A crash after the commit is safe: the
    /// next sync sees one head that descends from `base` and fast-forwards with backup.
    pub(crate) async fn write_resolution(
        &self,
        r: &Resolution<'_>,
    ) -> Result<Version, EngineError> {
        let parents = resolution_parents(&graph_of(r.versions));
        if parents.len() < 2 {
            return Err(EngineError::Usage(format!("no_conflict:{}", r.game)));
        }
        if !parents.contains(&r.keep) {
            return Err(EngineError::Usage(format!("not_a_head:{}", r.keep)));
        }
        let chosen = r
            .versions
            .get(&r.keep)
            .ok_or_else(|| EngineError::Data("head vanished".into()))?;
        let repo = self.repo_ref()?;
        let meta = UploadMeta {
            stream: r.stream,
            parents,
            device: self.device_id(),
            player: r.player,
            emulator: r.emulator,
            base_files: &chosen.files,
            note: None,
            takeover_from: None,
        };
        let v = commit_manifest(repo, meta, chosen.files.clone(), &self.failpoints).await?;
        self.failpoints.hit("resolve.after_commit")?;
        let ctx = Ctx {
            repo: Some(repo),
            state: &self.state,
            data_dir: &self.env.locations.data_dir,
            grants: &self.config.grants,
            probe: self.env.processes.as_ref(),
            process_names: r.names,
            max_file_bytes: r.max_file_bytes,
            fp: &self.failpoints,
            meter: r.meter,
        };
        restore::restore(&ctx, &r.stream, r.folder, Source::Remote(&v), "pre-resolve").await?;
        let mut row = self.state.stream(&r.stream)?.unwrap_or(StreamRow {
            stream: r.stream,
            player: r.player,
            emulator: r.emulator.to_owned(),
            game: r.game.to_owned(),
            base: None,
            base_files: vec![],
            highest_seen: None,
            replace_local: false,
        });
        row.base = Some(v.id);
        row.base_files = on_disk(r.folder, &v.files);
        row.highest_seen = row.highest_seen.max(Some(v.id));
        row.replace_local = false;
        self.state.put_stream(&row)?;
        Ok(v)
    }
}
