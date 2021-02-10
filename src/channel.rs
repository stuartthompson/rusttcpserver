pub struct Channel<T> {
    pub sender: std::sync::mpsc::Sender<T>,
    pub receiver: std::sync::mpsc::Receiver<T>,
}