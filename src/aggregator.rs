use tokio::sync::mpsc;
pub struct EvAggregator {
    rx: mpsc::Receiver
}
