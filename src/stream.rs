use std::io::Read;

use anyhow::Result;
use bytes::{Buf, BufMut, BytesMut};
use gami_mc_protocol::packets::login::server::{LoginSuccess, SetCompression};
use gami_mc_protocol::packets::play::server::KeepAlive;
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::{Origins, States};
use gami_mc_protocol::serialization::VarIntReader;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::sync::mpsc;

const ORIGIN: Origins = Origins::Server;

pub struct Stream {
    pub reader: OwnedReadHalf,
    pub tx: mpsc::Sender<Vec<u8>>,
    pub compression_threshold: i32,
    pub state: States,
    buffer: Vec<u8>,
    acc_buffer: BytesMut,
}

impl Stream {
    pub fn new(reader: OwnedReadHalf, tx: mpsc::Sender<Vec<u8>>) -> Self {
        Self {
            reader,
            tx,
            compression_threshold: -1,
            state: States::Login,
            buffer: vec![0; 500_000],
            acc_buffer: BytesMut::new(),
        }
    }

    pub fn listen(&mut self, mut writer: OwnedWriteHalf, mut rx: mpsc::Receiver<Vec<u8>>) {
        tokio::spawn(async move {
            while let Some(i) = rx.recv().await {
                writer.write_all(&i).await.unwrap();
            }
        });
    }

    pub async fn read_packets(&mut self) -> Result<Vec<Packets>> {
        let buf = &mut self.buffer;
        let mut packets = Vec::new();

        let bytes_read = self.reader.read(buf).await?;
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

        // match Packets::deserialize(packet_id, &bytes, &self.state, &ORIGIN) {
        //     Ok(packet) => return Ok(Some(packet)),
        //     Err(e) => eprintln!("Error deserializing packet: {:?}", e),
        // }

        if let Ok(packet) = Packets::deserialize(packet_id, &self.state, &ORIGIN, &bytes) {
            return Ok(Some(packet));
        }

        Ok(None)
    }

    async fn handle_events(&mut self, packet_id: i32, bytes: &[u8]) -> Result<Option<Packets>> {
        // Handle Set Compression
        if packet_id == 0x03 && self.state == States::Login {
            let packet = SetCompression::deserialize(bytes)?;
            self.compression_threshold = packet.threshold;
            return Ok(Some(Packets::SetCompression(packet)));
        }

        // Handle Login Success
        if packet_id == 0x02 && self.state == States::Login {
            self.state = States::Play;
            let packet = LoginSuccess::deserialize(bytes)?;
            return Ok(Some(Packets::LoginSuccess(packet)));
        }

        // Handle Keep Alive
        if packet_id == 0x00 && self.state == States::Play {
            let packet = KeepAlive::deserialize(bytes)?;
            let bytes = packet.serialize(self.compression_threshold)?;
            self.tx.send(bytes).await?;
            return Ok(Some(Packets::ServerKeepAlive(packet)));
        }

        Ok(None)
    }
}
