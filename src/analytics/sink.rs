use super::ClickDetail;

/// 点击计数 Sink（聚合模式）
#[async_trait::async_trait]
pub trait ClickSink: Send + Sync {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()>;
}

/// 详细点击日志 Sink（可选实现）
#[async_trait::async_trait]
pub trait DetailedClickSink: Send + Sync {
    /// 记录单条点击日志
    async fn log_click(&self, detail: ClickDetail) -> anyhow::Result<()>;

    /// 批量记录点击日志
    async fn log_clicks_batch(&self, details: Vec<ClickDetail>) -> anyhow::Result<()>;
}

pub struct StdoutSink;

#[async_trait::async_trait]
impl ClickSink for StdoutSink {
    async fn flush_clicks(&self, updates: Vec<(String, usize)>) -> anyhow::Result<()> {
        println!("Flushing clicks: {:?}", updates);
        Ok(())
    }
}

#[async_trait::async_trait]
impl DetailedClickSink for StdoutSink {
    async fn log_click(&self, detail: ClickDetail) -> anyhow::Result<()> {
        println!("Click log: {:?}", detail);
        Ok(())
    }

    async fn log_clicks_batch(&self, details: Vec<ClickDetail>) -> anyhow::Result<()> {
        println!("Click logs batch: {} entries", details.len());
        for detail in &details {
            println!("  - {:?}", detail);
        }
        Ok(())
    }
}
