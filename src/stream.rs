use kagami::tcp::State;
use tokio::net::TcpStream;

pub struct Stream {
    pub stream: TcpStream,
    pub compression_threshold: i8,
    pub state: State,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            compression_threshold: 0,
            state: State::Login,
        }
    }
}
