use std::io::Cursor;

use anyhow::Result;
use bytes::{Bytes, BytesMut};
use gami_mc_protocol::packets::login::server::{LoginSuccess, SetCompression};
use gami_mc_protocol::packets::play::server::KeepAlive;
use gami_mc_protocol::packets::{Packet, Packets};
use gami_mc_protocol::registry::tcp::{Origin, State};
use gami_mc_protocol::serialization::encoding::{decode_varint_length, VarIntReader};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio::spawn;
use tokio::sync::mpsc;

const ORIGIN: Origin = Origin::Server;

pub struct Stream {
    pub reader: OwnedReadHalf,
    pub compression_threshold: i32,
    pub state: State,
    tx: mpsc::Sender<Bytes>,
    buffer: BytesMut,
}

impl Stream {
    pub fn new(stream: TcpStream) -> Self {
        let (reader, writer) = stream.into_split();
        let (tx, rx) = mpsc::channel::<Bytes>(500);

        spawn(Self::writer_task(writer, rx));

        Self {
            reader,
            tx,
            compression_threshold: -1,
            state: State::Login,
            buffer: BytesMut::with_capacity(500_000),
        }
    }

    async fn writer_task(mut writer: OwnedWriteHalf, mut rx: mpsc::Receiver<Bytes>) -> Result<()> {
        while let Some(data) = rx.recv().await {
            writer.write_all(&data).await?;
            writer.flush().await?;
        }
        writer.shutdown().await?;
        Ok(())
    }

    pub async fn read_packets(&mut self) -> Result<Vec<Packets>> {
        let n = self.reader.read_buf(&mut self.buffer).await?;
        let mut packets = Vec::new();

        if n == 0 {
            return Ok(packets);
        }

        while let Ok((length, vbytes)) = decode_varint_length(&self.buffer) {
            if self.buffer.len() < vbytes + length {
                break;
            }

            if let Some(packet) = self.handle_packet(vbytes, vbytes + length).await? {
                packets.push(packet);
            }

            self.buffer = self.buffer[vbytes + length..].into();
        }

        Ok(packets)
    }

    async fn handle_packet(&mut self, start: usize, end: usize) -> Result<Option<Packets>> {
        let mut cursor = Cursor::new(&self.buffer[start..end]);

        if self.compression_threshold > 0 {
            let _data_len = cursor.read_varint()?;
        }

        let packet_id = cursor.read_varint()?;
        let body_start = cursor.position() as usize + 1;

        if let Some(packet) = self.handle_event(packet_id, body_start, end).await? {
            return Ok(Some(packet));
        }

        let bytes = &self.buffer[body_start..end];
        match Packets::deserialize(packet_id, &self.state, &ORIGIN, bytes) {
            Ok(packet) => return Ok(Some(packet)),
            Err(e) => {
                if !e.to_string().contains("Unknown packet") {
                    eprintln!("Error deserializing packet {packet_id}: {:?}", e);
                }
            }
        }

        Ok(None)
    }

    async fn handle_event(&mut self, id: i32, start: usize, end: usize) -> Result<Option<Packets>> {
        let bytes = &self.buffer[start..end];

        // Handle Set Compression
        if id == 0x03 && self.state == State::Login {
            let packet = SetCompression::deserialize(bytes)?;
            self.compression_threshold = packet.threshold;
            return Ok(Some(Packets::SetCompression(packet)));
        }

        // Handle Login Success
        if id == 0x02 && self.state == State::Login {
            self.state = State::Play;
            let packet = LoginSuccess::deserialize(bytes)?;
            return Ok(Some(Packets::LoginSuccess(packet)));
        }

        // Handle Keep Alive
        if id == 0x00 && self.state == State::Play {
            let packet = KeepAlive::deserialize(bytes)?;
            self.send_packet(&packet).await?;
            return Ok(Some(Packets::ServerKeepAlive(packet)));
        }

        Ok(None)
    }

    pub async fn send_packet(&self, packet: &impl Packet) -> Result<()> {
        let bytes = Packet::serialize(packet, self.compression_threshold)?;
        self.tx.send(bytes.into()).await?;

        Ok(())
    }

    pub fn send_packet_sync(&self, packet: &impl Packet) -> Result<()> {
        let bytes = Packet::serialize(packet, self.compression_threshold)?;
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = tx.send(bytes.into()).await {
                eprintln!("Error sending packet: {:?}", e);
            }
        });

        Ok(())
    }
}
