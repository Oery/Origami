use std::io::Read;

use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use kagami::minecraft::packets::login::server::{LoginSuccess, SetCompression};
use kagami::minecraft::packets::play::server::KeepAlive;
use kagami::minecraft::Packet;
use kagami::serialization::VarIntReader;
use kagami::tcp::Origin;
use kagami::{minecraft::Packets, tcp::State};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const ORIGIN: Origin = Origin::Server;

pub struct Stream {
    pub stream: TcpStream,
    pub compression_threshold: i32,
    pub state: State,
    buffer: Vec<u8>,
    acc_buffer: BytesMut,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            compression_threshold: 0,
            state: State::Login,
            buffer: vec![0; 500_000],
            acc_buffer: BytesMut::new(),
        }
    }

    pub async fn read_packets(&mut self) -> Result<Vec<Packets>> {
        let buf = &mut self.buffer;
        let mut packets = Vec::new();

        let bytes_read = self.stream.read(buf).await?;
        self.acc_buffer.put_slice(&buf[..bytes_read]);

        loop {
            let Ok((len, bytes)) = (&self.acc_buffer[..]).read_varint_full() else {
                break;
            };

            let total_len = bytes.len() + len as usize;

            if total_len > self.acc_buffer.len() {
                break;
            }

            if let Some(packet) = self.handle_packet(bytes.len(), total_len).await? {
                packets.push(packet);
            }

            self.acc_buffer.advance(total_len);
        }

        Ok(packets)
    }

    async fn handle_packet(&mut self, start: usize, end: usize) -> Result<Option<Packets>> {
        let packet_bytes = &self.acc_buffer[start..end];
        let mut reader = packet_bytes.reader();
        let mut bytes = vec![];

        if self.compression_threshold > 0 {
            let _data_len = reader.read_varint()?;
        }

        let packet_id = reader.read_varint()?;
        reader.read_to_end(&mut bytes)?;

        if let Some(packet) = self.handle_events(packet_id, &bytes).await? {
            return Ok(Some(packet));
        }

        if let Ok(packet) = Packets::deserialize(packet_id, &bytes, &self.state, &ORIGIN) {
            return Ok(Some(packet));
        }

        Ok(None)
    }

    async fn handle_events(&mut self, packet_id: i32, bytes: &[u8]) -> Result<Option<Packets>> {
        // Handle Set Compression
        if packet_id == 0x03 && self.state == State::Login {
            let packet = SetCompression::deserialize_packet(bytes)?;
            self.compression_threshold = packet.threshold;
            return Ok(Some(Packets::SetCompression(packet)));
        }

        // Handle Login Success
        if packet_id == 0x02 && self.state == State::Login {
            self.state = State::Play;
            let packet = LoginSuccess::deserialize_packet(bytes)?;
            return Ok(Some(Packets::LoginSuccess(packet)));
        }

        // Handle Keep Alive
        if packet_id == 0x00 && self.state == State::Play {
            let packet = KeepAlive::deserialize_packet(bytes)?;
            let bytes = packet.serialize_raw(self.compression_threshold)?;
            self.stream.write_all(&bytes).await?;
            return Ok(Some(Packets::ServerKeepAlive(packet)));
        }

        Ok(None)
    }
}
