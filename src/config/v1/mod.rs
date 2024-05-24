pub mod api;
pub mod auth;
pub mod database;
pub mod image_cache;
pub mod mail;
pub mod net;
pub mod tracker;
pub mod tracker_statistics_importer;
pub mod website;

use serde::{Deserialize, Serialize};

use self::api::Api;
use self::auth::{Auth, SecretKey};
use self::database::Database;
use self::image_cache::ImageCache;
use self::mail::Mail;
use self::net::Network;
use self::tracker::{ApiToken, Tracker};
use self::tracker_statistics_importer::TrackerStatisticsImporter;
use self::website::Website;
use super::validator::{ValidationError, Validator};

/// The whole configuration for the index.
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    /// Logging level. Possible values are: `Off`, `Error`, `Warn`, `Info`,
    /// `Debug` and `Trace`. Default is `Info`.
    pub log_level: Option<LogLevel>,
    /// The website customizable values.
    pub website: Website,
    /// The tracker configuration.
    pub tracker: Tracker,
    /// The network configuration.
    pub net: Network,
    /// The authentication configuration.
    pub auth: Auth,
    /// The database configuration.
    pub database: Database,
    /// The SMTP configuration.
    pub mail: Mail,
    /// The image proxy cache configuration.
    pub image_cache: ImageCache,
    /// The API configuration.
    pub api: Api,
    /// The tracker statistics importer job configuration.
    pub tracker_statistics_importer: TrackerStatisticsImporter,
}

impl Settings {
    pub fn override_tracker_api_token(&mut self, tracker_api_token: &ApiToken) {
        self.tracker.override_tracker_api_token(tracker_api_token);
    }

    pub fn override_auth_secret_key(&mut self, auth_secret_key: &str) {
        self.auth.override_secret_key(auth_secret_key);
    }

    pub fn remove_secrets(&mut self) {
        self.tracker.token = ApiToken::new("***");
        if let Some(_password) = self.database.connect_url.password() {
            let _ = self.database.connect_url.set_password(Some("***"));
        }
        "***".clone_into(&mut self.mail.password);
        self.auth.secret_key = SecretKey::new("***");
    }
}

impl Validator for Settings {
    fn validate(&self) -> Result<(), ValidationError> {
        self.tracker.validate()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}
