//! Category service.
use std::sync::Arc;

use super::user::DbUserRepository;
use crate::databases::database::{Category, Database, Error as DatabaseError};
use crate::errors::ServiceError;
use crate::models::category::CategoryId;
use crate::models::user::UserId;

pub struct Service {
    category_repository: Arc<DbCategoryRepository>,
    user_repository: Arc<DbUserRepository>,
}

impl Service {
    #[must_use]
    pub fn new(category_repository: Arc<DbCategoryRepository>, user_repository: Arc<DbUserRepository>) -> Service {
        Service {
            category_repository,
            user_repository,
        }
    }

    /// Adds a new category.
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The user does not have the required permissions.
    /// * There is a database error.
    pub async fn add_category(&self, category_name: &str, user_id: &UserId) -> Result<i64, ServiceError> {
        let user = self.user_repository.get_compact_user(user_id).await?;

        // Check if user is administrator
        // todo: extract authorization service
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

        match self.category_repository.add_category(category_name).await {
            Ok(id) => Ok(id),
            Err(e) => match e {
                DatabaseError::CategoryAlreadyExists => Err(ServiceError::CategoryExists),
                _ => Err(ServiceError::DatabaseError),
            },
        }
    }

    /// Deletes a new category.
    ///
    /// # Errors
    ///
    /// It returns an error if:
    ///
    /// * The user does not have the required permissions.
    /// * There is a database error.
    pub async fn delete_category(&self, category_name: &str, user_id: &UserId) -> Result<(), ServiceError> {
        let user = self.user_repository.get_compact_user(user_id).await?;

        // Check if user is administrator
        // todo: extract authorization service
        if !user.administrator {
            return Err(ServiceError::Unauthorized);
        }

        match self.category_repository.delete_category(category_name).await {
            Ok(_) => Ok(()),
            Err(e) => match e {
                DatabaseError::CategoryNotFound => Err(ServiceError::CategoryNotFound),
                _ => Err(ServiceError::DatabaseError),
            },
        }
    }
}

pub struct DbCategoryRepository {
    database: Arc<Box<dyn Database>>,
}

impl DbCategoryRepository {
    #[must_use]
    pub fn new(database: Arc<Box<dyn Database>>) -> Self {
        Self { database }
    }

    /// It returns the categories.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn get_categories(&self) -> Result<Vec<Category>, DatabaseError> {
        self.database.get_categories().await
    }

    /// Adds a new category.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn add_category(&self, category_name: &str) -> Result<CategoryId, DatabaseError> {
        self.database.insert_category_and_get_id(category_name).await
    }

    /// Deletes a new category.
    ///
    /// # Errors
    ///
    /// It returns an error if there is a database error.
    pub async fn delete_category(&self, category_name: &str) -> Result<(), DatabaseError> {
        self.database.delete_category(category_name).await
    }
}
