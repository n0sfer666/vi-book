pub trait VaultQuery {
    fn notes(&self) -> Result<Vec<String>, String>;
    fn tags(&self) -> Result<Vec<String>, String>;
    fn backlinks(&self, path: &str) -> Result<Vec<String>, String>;
}
