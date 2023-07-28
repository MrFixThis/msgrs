use std::{
    env,
    net::{SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
    sync::OnceLock,
    vec,
};

use anyhow::Context;
use config::{Config, File};
use serde::Deserialize;

static APP_SETTINGS: OnceLock<Settings> = OnceLock::new();
const SETTINGS_FILE: &str = "Settings.toml";

#[derive(Debug, Deserialize)]
pub struct Settings {
    host: String,
    port: u16,
    workers: usize,
    hb_interval: u64,
    timeout: u64,
}

impl Settings {
    #[inline(always)]
    pub fn workers(&self) -> usize {
        self.workers
    }

    #[inline(always)]
    pub fn hb_interval(&self) -> u64 {
        self.hb_interval
    }

    #[inline(always)]
    pub fn timeout(&self) -> u64 {
        self.timeout
    }
}

impl ToSocketAddrs for Settings {
    type Iter = vec::IntoIter<SocketAddr>;

    fn to_socket_addrs(&self) -> std::io::Result<Self::Iter> {
        use std::io::{Error, ErrorKind};

        match format!("{}:{}", self.host, self.port).parse::<SocketAddrV4>() {
            Ok(addr) => Ok(vec![SocketAddr::V4(addr)].into_iter()),
            Err(_) => match format!("[{}]:{}", self.host, self.port).parse::<SocketAddrV6>() {
                Ok(addr) => Ok(vec![SocketAddr::V6(addr)].into_iter()),
                Err(e) => Err(Error::new(ErrorKind::InvalidData, e)),
            },
        }
    }
}

/// Retrieves the application's `settings`.
///
/// # Panics
/// Panics if the application's `settings` has not been loaded yet.
#[inline]
pub fn get_app_settings() -> &'static Settings {
    APP_SETTINGS
        .get()
        .expect("settings file should already be loaded")
}

/// Loads the application's `settings` located in [`APP_SETTINGS`].
///
/// **Note**: This function must be called at the very first startup point of the server.
pub fn load_settings() -> anyhow::Result<()> {
    let mut settings = env::current_dir()?;
    settings.push(SETTINGS_FILE);

    _ = APP_SETTINGS.set(
        Config::builder()
            .add_source(File::from(settings))
            .build()
            .context("settings file")?
            .try_deserialize()
            .context("settings deserialization")?,
    );

    Ok(())
}
