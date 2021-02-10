pub struct Channel<T> {
    pub sender: std::sync::mpsc::Sender<T>,
    pub receiver: std::sync::mpsc::Receiver<T>,
}

impl<T> Channel<T> {
    pub fn new() -> Channel<T> {
        let (tx, rx) = std::sync::mpsc::channel::<T>();
        return Channel {
            sender: tx,
            receiver: rx
        };
    }
}