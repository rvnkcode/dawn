pub trait CalendarService {
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}

pub trait CalendarRepository {
    fn authenticate(&self) -> impl Future<Output = anyhow::Result<()>>;
}
