//! A play session over the daemon link (3c): one connection sends `play`, then receives
//! steps and questions and answers each question on the same connection, as the CLI's
//! `daemon_client::play` does. The session ends with its report, or when the app stops
//! listening: the connection drops, and the daemon gives the non-interactive answer to any
//! later question while the emulator session itself still uploads and releases its lease
//! (architecture §Process model). Spec: docs/specs/ui/app-shell.md (Data flow).

use lemusync_api::{PlayAnswer, Request, Response};
use lemusync_ipc::{Client, IpcError};
use tokio::sync::mpsc;

use super::Link;
use super::calls::{call_error_from, connect, connection_from, failed_from};
use super::flow_model::{PlayAnswerDto, PlayQuestionDto, PlayResultDto, PlayStepDto};
use super::model::ConnectionState;

/// What the session reports, in order. `Done` and `Failed` are last.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlayUpdate {
    Step(PlayStepDto),
    Question(PlayQuestionDto),
    Done(PlayResultDto),
    Failed(ConnectionState),
}

/// Sends the player's answers. Only the first answer after a question is used; answers
/// sent while no question waits are dropped when the next question arrives.
#[derive(Clone, Debug)]
pub(crate) struct Answerer(mpsc::UnboundedSender<PlayAnswerDto>);

impl Answerer {
    /// Queues an answer; `false` once the session has ended.
    pub(crate) fn answer(&self, a: PlayAnswerDto) -> bool {
        self.0.send(a).is_ok()
    }
}

/// Starts a session on the current tokio runtime. Dropping the receiver ends it at once,
/// even while the emulator runs (the daemon keeps the session).
pub(crate) fn start(
    link: Link,
    player: String,
    emulator: String,
    game: String,
) -> (mpsc::Receiver<PlayUpdate>, Answerer) {
    let (tx, rx) = mpsc::channel(16);
    let (atx, arx) = mpsc::unbounded_channel();
    let request = Request::Play {
        player,
        emulator,
        game,
        args: vec![],
    };
    tokio::spawn(async move {
        let last = tokio::select! {
            last = run(&link, request, &tx, arx) => last,
            () = tx.closed() => return,
        };
        if let Some(u) = last {
            let _ = tx.send(u).await;
        }
    });
    (rx, Answerer(atx))
}

/// Runs the session; returns the last update to send, or `None` when it just stops.
async fn run(
    link: &Link,
    request: Request,
    tx: &mpsc::Sender<PlayUpdate>,
    mut answers: mpsc::UnboundedReceiver<PlayAnswerDto>,
) -> Option<PlayUpdate> {
    let mut client: Client = match connect(link).await {
        Ok(c) => c,
        Err(state) => return Some(PlayUpdate::Failed(state)),
    };
    if let Err(e) = client.send(&request).await {
        return Some(PlayUpdate::Failed(call_error_from(e)));
    }
    loop {
        let frame = match client.recv().await {
            Ok(f) => f,
            Err(e) => return Some(PlayUpdate::Failed(call_error_from(e))),
        };
        match frame {
            Response::PlayEvent(e) => {
                tx.send(PlayUpdate::Step(e.into())).await.ok()?;
            }
            Response::Prompt(q) => {
                // Answers given before this question belong to no question.
                while answers.try_recv().is_ok() {}
                tx.send(PlayUpdate::Question(PlayQuestionDto::from(&q)))
                    .await
                    .ok()?;
                // `None`: the app stopped answering; dropping the connection is the refusal.
                let a = answers.recv().await?;
                if let Err(e) = client.send(&Request::Answer(PlayAnswer::from(a))).await {
                    return Some(PlayUpdate::Failed(call_error_from(e)));
                }
            }
            Response::Play(report) => return Some(PlayUpdate::Done(PlayResultDto::from(&report))),
            Response::Error(body) => return Some(PlayUpdate::Failed(failed_from(body))),
            _ => return Some(PlayUpdate::Failed(connection_from(IpcError::Protocol))),
        }
    }
}

// The link tests drive a fake daemon over a Unix socket; Windows IPC is Phase 2 Task 11.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::link::fake::{Fake, PlayStep, Script, error_body};
    use lemusync_api::{
        ConflictInfo, HeadInfo, LeaseAnswer, PlayEvent, PlayLease, PlayQuestion, PlayReport,
        StreamAction, StreamReport,
    };
    use std::time::Duration;

    fn play(steps: Vec<PlayStep>) -> Fake {
        Fake::start(Script {
            play: steps,
            ..Script::default()
        })
    }

    fn begin(fake: &Fake) -> (mpsc::Receiver<PlayUpdate>, Answerer) {
        start(fake.link.clone(), "Ana".into(), "cemu".into(), "aa".into())
    }

    fn send(r: Response) -> PlayStep {
        PlayStep::Send(Box::new(r))
    }

    fn event(e: PlayEvent) -> PlayStep {
        send(Response::PlayEvent(e))
    }

    fn lease_question(stale: bool) -> PlayStep {
        send(Response::Prompt(PlayQuestion::Lease {
            by: "pc".into(),
            ago_s: 90,
            stale,
        }))
    }

    fn stream(files: u32) -> StreamReport {
        StreamReport {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            account: None,
            action: StreamAction::Upload,
            version: Some("v9".into()),
            heads: vec!["v9".into()],
            done: true,
            files,
            bytes: 10,
        }
    }

    fn report() -> PlayStep {
        send(Response::Play(PlayReport {
            api_version: lemusync_api::API_VERSION,
            pulled: stream(0),
            exit_code: 0,
            uploaded: stream(3),
            lease: PlayLease::TookOver,
            renewals: 2,
            warnings: vec![],
        }))
    }

    async fn next(rx: &mut mpsc::Receiver<PlayUpdate>) -> PlayUpdate {
        tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("an update in time")
            .expect("the session still open")
    }

    /// Waits until the fake has logged `what` (the server runs on its own task).
    async fn logged(fake: &Fake, what: &str) {
        for _ in 0..500 {
            if fake.log().contains(&what) {
                return;
            }
            tokio::time::sleep(Duration::from_millis(2)).await;
        }
        panic!("the fake never logged {what}: {:?}", fake.log());
    }

    #[tokio::test]
    async fn steps_arrive_in_order_then_the_result() {
        let fake = play(vec![
            event(PlayEvent::Pulling),
            event(PlayEvent::Playing),
            event(PlayEvent::Uploading),
            report(),
        ]);
        let (mut rx, _a) = begin(&fake);
        assert_eq!(next(&mut rx).await, PlayUpdate::Step(PlayStepDto::Pulling));
        assert_eq!(next(&mut rx).await, PlayUpdate::Step(PlayStepDto::Playing));
        assert_eq!(
            next(&mut rx).await,
            PlayUpdate::Step(PlayStepDto::Uploading)
        );
        let PlayUpdate::Done(done) = next(&mut rx).await else {
            panic!("expected Done");
        };
        assert_eq!((done.exit_code, done.uploaded.files), (0, 3));
        assert_eq!(done.lease, crate::link::flow_model::PlayLeaseDto::TookOver);
        assert_eq!(rx.recv().await, None, "Done is the last update");
        assert_eq!(
            fake.requests(),
            [Request::Play {
                player: "Ana".into(),
                emulator: "cemu".into(),
                game: "aa".into(),
                args: vec![],
            }]
        );
    }

    #[tokio::test]
    async fn each_lease_choice_is_sent_once_on_the_same_connection() {
        for (dto, wire) in [
            (PlayAnswerDto::Wait, LeaseAnswer::Wait),
            (PlayAnswerDto::PlayAnyway, LeaseAnswer::PlayAnyway),
            (PlayAnswerDto::TakeOver, LeaseAnswer::TakeOver),
        ] {
            let fake = play(vec![lease_question(true), PlayStep::Read, report()]);
            let (mut rx, a) = begin(&fake);
            assert_eq!(
                next(&mut rx).await,
                PlayUpdate::Question(PlayQuestionDto::Lease {
                    by: "pc".into(),
                    ago_s: 90,
                    stale: true
                })
            );
            assert!(a.answer(dto.clone()));
            assert!(
                a.answer(PlayAnswerDto::Refuse),
                "a second answer is queued…"
            );
            assert!(matches!(next(&mut rx).await, PlayUpdate::Done(_)));
            let answers: Vec<Request> = fake.requests().into_iter().skip(1).collect();
            assert_eq!(
                answers,
                [Request::Answer(PlayAnswer::Lease { choice: wire })],
                "…but only the first reaches the daemon ({dto:?})"
            );
            assert_eq!(fake.count("play"), 1, "one connection, one play");
        }
    }

    #[tokio::test]
    async fn an_answer_given_before_the_question_is_dropped() {
        let conflict = ConflictInfo {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            name: None,
            heads: vec![HeadInfo {
                version: "h1".into(),
                device_name: "pc".into(),
                player: "Ana".into(),
                created_at_ms: 1,
                files: 1,
                bytes: 1,
                changed_files: 1,
            }],
        };
        let fake = play(vec![
            event(PlayEvent::Pulling),
            send(Response::Prompt(PlayQuestion::Conflict { conflict })),
            PlayStep::Read,
            report(),
        ]);
        let (mut rx, a) = begin(&fake);
        assert!(a.answer(PlayAnswerDto::TakeOver), "early, stale answer");
        assert_eq!(next(&mut rx).await, PlayUpdate::Step(PlayStepDto::Pulling));
        assert!(matches!(
            next(&mut rx).await,
            PlayUpdate::Question(PlayQuestionDto::Conflict { .. })
        ));
        a.answer(PlayAnswerDto::Keep {
            version: Some("h1".into()),
        });
        assert!(matches!(next(&mut rx).await, PlayUpdate::Done(_)));
        assert_eq!(
            fake.requests().last(),
            Some(&Request::Answer(PlayAnswer::Keep {
                version: Some("h1".into())
            }))
        );
    }

    #[tokio::test]
    async fn dropping_the_session_on_a_question_hangs_up() {
        let fake = play(vec![lease_question(false), PlayStep::Read]);
        let (mut rx, a) = begin(&fake);
        assert!(matches!(next(&mut rx).await, PlayUpdate::Question(_)));
        drop(rx);
        logged(&fake, "eof").await;
        assert!(!a.answer(PlayAnswerDto::Wait), "the session has ended");
    }

    #[tokio::test]
    async fn dropping_the_answerer_on_a_question_hangs_up() {
        let fake = play(vec![lease_question(false), PlayStep::Read]);
        let (mut rx, a) = begin(&fake);
        assert!(matches!(next(&mut rx).await, PlayUpdate::Question(_)));
        drop(a);
        logged(&fake, "eof").await;
        assert_eq!(rx.recv().await, None, "no Failed: the app chose to stop");
    }

    #[tokio::test]
    async fn dropping_the_session_while_playing_hangs_up() {
        let fake = play(vec![event(PlayEvent::Playing), PlayStep::Read]);
        let (mut rx, _a) = begin(&fake);
        assert_eq!(next(&mut rx).await, PlayUpdate::Step(PlayStepDto::Playing));
        drop(rx);
        logged(&fake, "eof").await;
    }

    #[tokio::test]
    async fn a_daemon_error_and_a_hang_up_are_failed() {
        let fake = play(vec![send(Response::Error(error_body(
            "game_not_installed",
        )))]);
        let (mut rx, _a) = begin(&fake);
        assert_eq!(
            next(&mut rx).await,
            PlayUpdate::Failed(ConnectionState::Failed {
                code: "game_not_installed".into(),
                message: "game_not_installed happened".into()
            })
        );

        let fake = play(vec![event(PlayEvent::Playing), PlayStep::Close]);
        let (mut rx, _a) = begin(&fake);
        assert_eq!(next(&mut rx).await, PlayUpdate::Step(PlayStepDto::Playing));
        assert_eq!(
            next(&mut rx).await,
            PlayUpdate::Failed(ConnectionState::NotRunning)
        );
    }

    #[tokio::test]
    async fn no_daemon_is_failed_not_running() {
        let dir = tempfile::tempdir().expect("tempdir");
        let link = Link::at(dir.path().join("daemon.sock"));
        let (mut rx, _a) = start(link, "Ana".into(), "cemu".into(), "aa".into());
        assert_eq!(
            next(&mut rx).await,
            PlayUpdate::Failed(ConnectionState::NotRunning)
        );
    }
}
