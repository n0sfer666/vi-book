#[derive(Default)]
pub struct Applied {
    pub changed: bool,
    pub errors: Vec<String>,
}
