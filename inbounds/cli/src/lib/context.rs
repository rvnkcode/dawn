use dawn::domain::task::port::TaskService;

pub struct AppContext<TS: TaskService> {
    pub task_service: TS,
}

impl<TS: TaskService> AppContext<TS> {
    pub fn new(task_service: TS) -> Self {
        Self { task_service }
    }
}
