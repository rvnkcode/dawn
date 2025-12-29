use super::*;

fn compose_description(filters: &[String], description: &[String]) -> anyhow::Result<Description> {
    let description_text = filters
        .iter()
        .chain(description.iter())
        .map(|s| s.trim())
        .collect::<Vec<_>>()
        .join(" ");
    Ok(Description::new(&description_text)?)
}

impl<TS: TaskService> Handler<TS> {
    pub fn add(&self, filters: &[String], args: &Modification) -> anyhow::Result<()> {
        let description = compose_description(filters, &args.description)?;
        let request = TaskCreation { description };
        self.task_service.add(request)?;
        let count = self.task_service.count_pending();
        println!("Created task {}.", count);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::strs;

    #[test]
    fn test_compose_description_with_filters_and_description() {
        let filters = strs(&["urgent", "work"]);
        let description = strs(&["complete", "project"]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent work complete project");
    }

    #[test]
    fn test_compose_description_with_only_description() {
        let filters = strs(&[]);
        let description = strs(&["buy", "milk"]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "buy milk");
    }

    #[test]
    fn test_compose_description_with_only_filters() {
        let filters = strs(&["urgent", "task"]);
        let description = strs(&[]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent task");
    }

    #[test]
    fn test_compose_description_empty() {
        let filters = strs(&[]);
        let description = strs(&[]);

        let result = compose_description(&filters, &description);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_description_whitespace_only() {
        let filters = strs(&["  "]);
        let description = strs(&["   "]);

        let result = compose_description(&filters, &description);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_description_trims_whitespace() {
        let filters = strs(&["  urgent  "]);
        let description = strs(&["  task  "]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "urgent task");
    }

    #[test]
    fn test_compose_description_single_word() {
        let filters = strs(&[]);
        let description = strs(&["hello"]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "hello");
    }

    #[test]
    fn test_compose_description_single_word_in_filters() {
        let filters = strs(&["greet"]);
        let description = strs(&[]);

        let result = compose_description(&filters, &description);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "greet");
    }
}
