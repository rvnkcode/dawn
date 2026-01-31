//! Service implementation for authentication

use crate::domain::auth::port::AuthenticateService;

use super::port::Authenticatable;

pub struct Service<A: Authenticatable> {
    repo: A,
}

impl<A: Authenticatable> Service<A> {
    pub fn new(repo: A) -> Self {
        Self { repo }
    }
}

impl<A: Authenticatable> AuthenticateService for Service<A> {
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.repo.authenticate().await
    }
}
