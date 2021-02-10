pub struct Channel {
    pub sender: std::sync::mpsc::Sender<String>,
    pub receiver: std::sync::mpsc::Receiver<String>,
}