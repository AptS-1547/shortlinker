#[async_trait::async_trait]
pub trait ClickSink: Send + Sync {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()>;
}

pub struct StdoutSink;

#[async_trait::async_trait]
impl ClickSink for StdoutSink {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        println!("Flushing clicks: {:?}", updates);
        Ok(())
    }
}
