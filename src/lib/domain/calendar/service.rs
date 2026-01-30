use super::port::{CalendarRepository, CalendarService};

// Generic type 'C' should implement 'CalendarRepository' trait
pub struct Service<C: CalendarRepository> {
    repo: C,
}

impl<C: CalendarRepository> Service<C> {
    pub fn new(repo: C) -> Self {
        Self { repo }
    }
}

impl<C: CalendarRepository> CalendarService for Service<C> {
    async fn authenticate(&self) -> anyhow::Result<()> {
        self.repo.authenticate().await
    }
}
