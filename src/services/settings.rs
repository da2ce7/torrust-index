//! Settings service.
use std::sync::Arc;

use super::user::DbUserRepository;
use crate::config::{Configuration, ConfigurationPublic, TorrustIndex};
use crate::errors::ServiceError;
use crate::models::user::UserId;

pub struct Service {
    configuration: Arc<Configuration>,
    user_repository: Arc<DbUserRepository>,
}

impl Service {
    #[must_use]
    pub fn new(configuration: Arc<Configuration>, user_repository: Arc<DbUserRepository>) -> Service {
        Service {
            configuration,
            user_repository,
        }
    }

    /// It gets all the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_all(&self, user_id: &UserId) -> Result<TorrustIndex, ServiceError> {
        let user = self.user_repository.get_compact(user_id).await?;

        // Check if user is administrator
        // todo: extract authorization service
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

        Ok(self.configuration.get_all().await)
    }

    /// It gets only the public settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_public(&self) -> ConfigurationPublic {
        self.configuration.get_public().await
    }

    /// It gets the site name from the settings.
    ///
    /// # Errors
    ///
    /// It returns an error if the user does not have the required permissions.
    pub async fn get_site_name(&self) -> String {
        self.configuration.get_site_name().await
    }
}
