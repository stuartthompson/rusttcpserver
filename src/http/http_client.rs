use crate::channel::Channel;

pub struct HttpClient {
    pub address: std::net::SocketAddr,
    pub channel: Channel
}