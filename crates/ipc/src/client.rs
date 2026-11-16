//! The CLI and UI side: connect, handshake, call, subscribe.

use std::path::Path;

use futures_util::Stream;
use lemusync_api::{Event, HelloReply, Request, Response};

use crate::IpcError;

/// A handshaken connection to the daemon.
pub struct Client {
    #[cfg(unix)]
    stream: tokio::net::UnixStream,
    daemon: HelloReply,
}

/// What the daemon answers to `Hello`.
#[cfg(unix)]
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum HelloAnswer {
    Ok(HelloReply),
    Refused(Box<Response>),
}

impl Client {
    /// Connects to `path` and does the handshake as `client_name`.
    ///
    /// # Errors
    /// [`IpcError::NotRunning`] when nothing listens; [`IpcError::Rejected`] on a version
    /// mismatch; I/O and frame errors; [`IpcError::Unsupported`] on Windows.
    #[cfg(unix)]
    pub async fn connect(path: &Path, client_name: &str) -> Result<Client, IpcError> {
        use std::io::ErrorKind;

        use lemusync_api::{API_VERSION, Hello};

        use crate::frame::{read_frame, write_frame};

        let mut stream = match tokio::net::UnixStream::connect(path).await {
            Ok(s) => s,
            Err(e) if matches!(e.kind(), ErrorKind::NotFound | ErrorKind::ConnectionRefused) => {
                return Err(IpcError::NotRunning);
            }
            Err(e) => return Err(e.into()),
        };
        let hello = Hello {
            api_version: API_VERSION,
            client: client_name.into(),
        };
        write_frame(&mut stream, &hello).await?;
        match read_frame::<HelloAnswer, _>(&mut stream).await? {
            Some(HelloAnswer::Ok(daemon)) => Ok(Client { stream, daemon }),
            Some(HelloAnswer::Refused(r)) => match *r {
                Response::Error(body) => Err(IpcError::Rejected(body)),
                _ => Err(IpcError::Protocol),
            },
            None => Err(IpcError::Closed),
        }
    }

    /// See the Unix version.
    ///
    /// # Errors
    /// Always [`IpcError::Unsupported`].
    #[cfg(not(unix))]
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)] // same signature as on Unix
    pub async fn connect(path: &Path, client_name: &str) -> Result<Client, IpcError> {
        let _ = (path, client_name);
        Err(IpcError::Unsupported)
    }

    /// The daemon's handshake reply.
    #[must_use]
    pub fn daemon(&self) -> &HelloReply {
        &self.daemon
    }

    /// Sends one request and waits for its response.
    ///
    /// # Errors
    /// [`IpcError::Closed`] when the daemon hangs up; I/O and frame errors.
    #[cfg(unix)]
    pub async fn call(&mut self, request: Request) -> Result<Response, IpcError> {
        crate::frame::write_frame(&mut self.stream, &request).await?;
        crate::frame::read_frame(&mut self.stream)
            .await?
            .ok_or(IpcError::Closed)
    }

    /// See the Unix version.
    ///
    /// # Errors
    /// Always [`IpcError::Unsupported`].
    #[cfg(not(unix))]
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)] // same signature as on Unix
    pub async fn call(&mut self, request: Request) -> Result<Response, IpcError> {
        let _ = request;
        Err(IpcError::Unsupported)
    }

    /// Sends one request without waiting: for `play`, whose answer is several frames.
    ///
    /// # Errors
    /// I/O and frame errors.
    #[cfg(unix)]
    pub async fn send(&mut self, request: &Request) -> Result<(), IpcError> {
        crate::frame::write_frame(&mut self.stream, request).await
    }

    /// See the Unix version.
    ///
    /// # Errors
    /// Always [`IpcError::Unsupported`].
    #[cfg(not(unix))]
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)] // same signature as on Unix
    pub async fn send(&mut self, request: &Request) -> Result<(), IpcError> {
        let _ = request;
        Err(IpcError::Unsupported)
    }

    /// The next response frame.
    ///
    /// # Errors
    /// [`IpcError::Closed`] when the daemon hangs up; I/O and frame errors.
    #[cfg(unix)]
    pub async fn recv(&mut self) -> Result<Response, IpcError> {
        crate::frame::read_frame(&mut self.stream)
            .await?
            .ok_or(IpcError::Closed)
    }

    /// See the Unix version.
    ///
    /// # Errors
    /// Always [`IpcError::Unsupported`].
    #[cfg(not(unix))]
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)] // same signature as on Unix
    pub async fn recv(&mut self) -> Result<Response, IpcError> {
        Err(IpcError::Unsupported)
    }

    /// Sends `Subscribe` and turns the connection into a stream of events. Events this
    /// version cannot decode are skipped. The stream ends when the daemon closes; after an
    /// error it yields nothing more.
    ///
    /// # Errors
    /// [`IpcError::Rejected`] or [`IpcError::Protocol`] when the daemon doesn't answer
    /// `subscribed`; I/O and frame errors.
    pub async fn subscribe(
        mut self,
    ) -> Result<impl Stream<Item = Result<Event, IpcError>>, IpcError> {
        match self.call(Request::Subscribe).await? {
            Response::Subscribed => {}
            Response::Error(body) => return Err(IpcError::Rejected(body)),
            _ => return Err(IpcError::Protocol),
        }
        Ok(futures_util::stream::unfold(
            Some(self),
            |client| async move {
                let mut client = client?;
                match client.next_event().await {
                    Ok(Some(e)) => Some((Ok(e), Some(client))),
                    Ok(None) => None,
                    Err(e) => Some((Err(e), None)),
                }
            },
        ))
    }

    #[cfg(unix)]
    async fn next_event(&mut self) -> Result<Option<Event>, IpcError> {
        // Within one API version a newer daemon may add events (or fields an old payload
        // lacks): a frame that is JSON but not an `Event` we know is skipped. A body that
        // is not JSON, an oversized frame and I/O errors stay fatal.
        loop {
            let Some(value) =
                crate::frame::read_frame::<serde_json::Value, _>(&mut self.stream).await?
            else {
                return Ok(None);
            };
            if let Ok(event) = serde_json::from_value::<Event>(value) {
                return Ok(Some(event));
            }
            tracing::debug!("skipped an event this version does not know");
        }
    }

    #[cfg(not(unix))]
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)] // same signature as on Unix
    async fn next_event(&mut self) -> Result<Option<Event>, IpcError> {
        Err(IpcError::Unsupported)
    }
}
