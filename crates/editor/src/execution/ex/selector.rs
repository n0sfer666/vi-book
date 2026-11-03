use crate::{
    autocmd::{Error, Selector},
    execution::Context,
    workspace::DocId,
};

impl Context<'_> {
    pub fn autocmd_selector(&self, source: &str, active: Option<DocId>) -> Result<Selector, Error> {
        if source == "<buffer=abuf>"
            && let Some(id) = active
        {
            return Ok(Selector::Buffer(id));
        }
        let number = if source == "<buffer>" {
            0
        } else if let Some(text) = source
            .strip_prefix("<buffer=")
            .and_then(|s| s.strip_suffix('>'))
        {
            text.parse::<u64>().unwrap_or(0)
        } else {
            return Ok(Selector::File(source.into()));
        };
        self.buffer(number)
            .map(Selector::Buffer)
            .ok_or(Error::Buffer(number))
    }
}
