//! The Linux keystore: a minimal Secret Service client that never starts or creates a store
//! (security §Credentials, §Secret Service activation).
//!
//! - `NameHasOwner` first, on the bus driver (which never activates). No owner means
//!   [`EngineError::SecretsUnavailable`]: a temporary state the scheduler retries with backoff.
//! - Every later call goes to the owner's **unique** name with `NO_AUTO_START`, so even if the
//!   owner exits mid-call the bus has nothing to activate.
//! - Only items whose attributes say `service = lemusync` are searched, read, written or
//!   deleted. Nothing is unlocked (a locked item or collection is "not available yet"), no
//!   collection is created, renamed or made default, and no prompt is ever run.
//! - Secrets cross the bus encrypted (`lemusync_crypto::secret_service`).

use std::collections::HashMap;
use std::time::Duration;

use dbus::Message;
use dbus::arg::{RefArg, Variant};
use dbus::channel::Channel;
use lemusync_crypto::secret_service::{ALGORITHM, DhSession, TransportKey};
use zeroize::Zeroizing;

use crate::EngineError;
use crate::secrets::SecretStore;

pub const SECRETS_NAME: &str = "org.freedesktop.secrets";
const SS_PATH: &str = "/org/freedesktop/secrets";
const SERVICE_IFACE: &str = "org.freedesktop.Secret.Service";
const COLLECTION_IFACE: &str = "org.freedesktop.Secret.Collection";
const ITEM_IFACE: &str = "org.freedesktop.Secret.Item";
const TIMEOUT: Duration = Duration::from_secs(5);

/// Our items carry these attributes (plus `username` = the secret's name). `target` and
/// `service` + `username` match what `keyring` 3 wrote before, so older items are found.
const SERVICE: &str = "lemusync";

type Secret = (dbus::Path<'static>, Vec<u8>, Vec<u8>, String);

/// The session bus's Secret Service, whoever provides it.
pub struct SecretServiceStore {
    /// The bus address; `None` = this session's (`DBUS_SESSION_BUS_ADDRESS`, else
    /// `$XDG_RUNTIME_DIR/bus`). Never autolaunch.
    address: Option<String>,
}

impl SecretServiceStore {
    #[must_use]
    pub fn session() -> Self {
        Self { address: None }
    }

    /// A specific bus (tests run against a throwaway one).
    #[must_use]
    pub fn at(address: &str) -> Self {
        Self {
            address: Some(address.to_owned()),
        }
    }

    fn connect(&self) -> Result<Conn, EngineError> {
        let address = match &self.address {
            Some(a) => a.clone(),
            None => session_address().ok_or_else(|| unavailable("no_bus"))?,
        };
        // `autolaunch:` would spawn a bus daemon: never start anything.
        if address.starts_with("autolaunch:") {
            return Err(unavailable("no_bus"));
        }
        let mut ch = Channel::open_private(&address).map_err(|_| unavailable("no_bus"))?;
        ch.register().map_err(|_| unavailable("no_bus"))?;
        let mut conn = Conn {
            ch,
            owner: String::new(),
        };
        conn.owner = conn.owner_of_secrets()?;
        Ok(conn)
    }

    fn open(&self) -> Result<(Conn, dbus::Path<'static>, TransportKey), EngineError> {
        let conn = self.connect()?;
        let dh = DhSession::new()?;
        let reply = conn.call(SS_PATH, SERVICE_IFACE, "OpenSession", |m| {
            m.append2(ALGORITHM, Variant(dh.public_key().to_vec()))
        })?;
        let (out, session): (Variant<Box<dyn RefArg>>, dbus::Path<'static>) =
            reply.read2().map_err(|_| bad_reply())?;
        let server = dbus::arg::cast::<Vec<u8>>(&out.0).ok_or_else(bad_reply)?;
        let key = dh.finish(server)?;
        Ok((conn, session, key))
    }
}

/// `DBUS_SESSION_BUS_ADDRESS`, else the systemd user bus socket if it exists.
fn session_address() -> Option<String> {
    if let Some(a) = std::env::var("DBUS_SESSION_BUS_ADDRESS")
        .ok()
        .filter(|a| !a.is_empty())
    {
        return Some(a);
    }
    let bus = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR")?).join("bus");
    bus.exists().then(|| format!("unix:path={}", bus.display()))
}

fn unavailable(key: &str) -> EngineError {
    tracing::info!("secrets are not available yet ({key}); will retry");
    EngineError::SecretsUnavailable(key.into())
}

fn bad_reply() -> EngineError {
    EngineError::Secrets("bad_reply".into())
}

fn search_attrs(name: &str) -> HashMap<&str, &str> {
    HashMap::from([("service", SERVICE), ("username", name)])
}

struct Conn {
    ch: Channel,
    /// The unique name (`:1.42`) that owns `org.freedesktop.secrets`.
    owner: String,
}

impl Conn {
    /// `NameHasOwner`, then the owner's unique name. The bus driver never activates.
    fn owner_of_secrets(&self) -> Result<String, EngineError> {
        let driver = |member: &str| -> Result<Message, EngineError> {
            let mut m = Message::new_method_call(
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                member,
            )
            .map_err(|_| bad_reply())?
            .append1(SECRETS_NAME);
            m.set_auto_start(false);
            self.ch
                .send_with_reply_and_block(m, TIMEOUT)
                .map_err(|_| unavailable("no_bus"))
        };
        let has: bool = driver("NameHasOwner")?.read1().map_err(|_| bad_reply())?;
        if !has {
            return Err(unavailable("no_owner"));
        }
        // Gone between the two calls: same as never there.
        driver("GetNameOwner")?
            .read1()
            .map_err(|_| unavailable("no_owner"))
    }

    /// One call to the owner's unique name, `NO_AUTO_START` set.
    fn call(
        &self,
        path: &str,
        iface: &str,
        member: &str,
        args: impl FnOnce(Message) -> Message,
    ) -> Result<Message, EngineError> {
        let mut m = args(
            Message::new_method_call(self.owner.as_str(), path, iface, member)
                .map_err(|_| bad_reply())?,
        );
        m.set_auto_start(false);
        self.ch.send_with_reply_and_block(m, TIMEOUT).map_err(|e| {
            match e.name() {
                // The owner left (logout, crash): wait for the next one.
                Some(
                    "org.freedesktop.DBus.Error.ServiceUnknown"
                    | "org.freedesktop.DBus.Error.NameHasNoOwner"
                    | "org.freedesktop.DBus.Error.NoReply"
                    | "org.freedesktop.DBus.Error.Disconnected",
                ) => unavailable("gone"),
                Some("org.freedesktop.Secret.Error.IsLocked") => unavailable("locked"),
                _ => EngineError::Secrets(lemusync_store::redact(&format!(
                    "{member}: {}",
                    e.name().unwrap_or("error")
                ))),
            }
        })
    }

    /// Our items for `name`. A locked one means the store isn't open yet: never unlock it.
    fn find(&self, name: &str) -> Result<Option<dbus::Path<'static>>, EngineError> {
        let reply = self.call(SS_PATH, SERVICE_IFACE, "SearchItems", |m| {
            m.append1(search_attrs(name))
        })?;
        let (unlocked, locked): (Vec<dbus::Path<'static>>, Vec<dbus::Path<'static>>) =
            reply.read2().map_err(|_| bad_reply())?;
        if !locked.is_empty() {
            return Err(unavailable("locked"));
        }
        match unlocked.len() {
            0 => Ok(None),
            1 => Ok(unlocked.into_iter().next()),
            _ => Err(EngineError::Secrets("ambiguous".into())),
        }
    }

    /// A prompt would be a dialog the user didn't ask for: never run one.
    fn no_prompt(prompt: &dbus::Path<'_>) -> Result<(), EngineError> {
        if &**prompt == "/" {
            Ok(())
        } else {
            Err(EngineError::Secrets("prompt_needed".into()))
        }
    }

    fn locked(&self, path: &str, iface: &str) -> Result<bool, EngineError> {
        let reply = self.call(path, "org.freedesktop.DBus.Properties", "Get", |m| {
            m.append2(iface, "Locked")
        })?;
        let v: Variant<bool> = reply.read1().map_err(|_| bad_reply())?;
        Ok(v.0)
    }
}

impl SecretStore for SecretServiceStore {
    fn get(&self, name: &str) -> Result<Option<Zeroizing<String>>, EngineError> {
        let (conn, session, key) = self.open()?;
        let Some(item) = conn.find(name)? else {
            return Ok(None);
        };
        let reply = conn.call(SS_PATH, SERVICE_IFACE, "GetSecrets", |m| {
            m.append2(vec![item.clone()], session)
        })?;
        let secrets: HashMap<dbus::Path<'static>, Secret> =
            reply.read1().map_err(|_| bad_reply())?;
        let (_, iv, ct, _) = secrets.get(&item).ok_or_else(|| unavailable("locked"))?;
        let plain = key.decrypt(iv, ct)?;
        let text = String::from_utf8(plain.to_vec()).map_err(|_| bad_reply())?;
        Ok(Some(Zeroizing::new(text)))
    }

    fn set(&self, name: &str, value: &str) -> Result<(), EngineError> {
        let (conn, session, key) = self.open()?;
        let (iv, ct) = key.encrypt(value.as_bytes())?;
        let secret: Secret = (session, iv, ct, "text/plain".into());
        if let Some(item) = conn.find(name)? {
            conn.call(&item, ITEM_IFACE, "SetSecret", |m| m.append1(secret))?;
            return Ok(());
        }
        // A new item goes in the user's default collection, which must exist and be open.
        let reply = conn.call(SS_PATH, SERVICE_IFACE, "ReadAlias", |m| {
            m.append1("default")
        })?;
        let collection: dbus::Path<'static> = reply.read1().map_err(|_| bad_reply())?;
        if &*collection == "/" {
            return Err(EngineError::Secrets("no_default_collection".into()));
        }
        if conn.locked(&collection, COLLECTION_IFACE)? {
            return Err(unavailable("locked"));
        }
        let attrs: HashMap<String, String> = HashMap::from([
            ("service".into(), SERVICE.into()),
            ("username".into(), name.into()),
            ("target".into(), "default".into()),
            ("application".into(), "lemusync".into()),
        ]);
        let mut props: HashMap<&str, Variant<Box<dyn RefArg>>> = HashMap::new();
        props.insert(
            "org.freedesktop.Secret.Item.Label",
            Variant(Box::new(format!("LemuSync: {name}"))),
        );
        props.insert(
            "org.freedesktop.Secret.Item.Attributes",
            Variant(Box::new(attrs)),
        );
        let reply = conn.call(&collection, COLLECTION_IFACE, "CreateItem", |m| {
            m.append3(props, secret, true)
        })?;
        let (_, prompt): (dbus::Path<'static>, dbus::Path<'static>) =
            reply.read2().map_err(|_| bad_reply())?;
        Conn::no_prompt(&prompt)
    }

    fn delete(&self, name: &str) -> Result<(), EngineError> {
        let conn = self.connect()?;
        let Some(item) = conn.find(name)? else {
            return Ok(());
        };
        let reply = conn.call(&item, ITEM_IFACE, "Delete", |m| m)?;
        let prompt: dbus::Path<'static> = reply.read1().map_err(|_| bad_reply())?;
        Conn::no_prompt(&prompt)
    }
}
