use crate::error::MultihookResult;

pub enum HookAction {}

impl HookAction {
    pub async fn execute(&self, body: &str) -> MultihookResult<()> {
        Ok(())
    }
}
