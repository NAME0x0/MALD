use anyhow::Result;

pub async fn run() -> Result<()> {
    crate::commands::wizard::run().await
}
