//! Renet/Netcode adapters. No Bevy dependency, no custom reliable transport.
use super::*;
use renet::{ChannelConfig, ConnectionConfig, RenetClient, SendType};
use renet_netcode::{ClientAuthentication, NetcodeClientTransport};
use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub fn config() -> ConnectionConfig {
    let channels = vec![
        ChannelConfig {
            channel_id: CONTROL,
            max_memory_usage_bytes: 16 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(100),
            },
        },
        ChannelConfig {
            channel_id: STATE,
            max_memory_usage_bytes: 16 * 1024,
            send_type: SendType::Unreliable,
        },
    ];
    ConnectionConfig {
        available_bytes_per_tick: 4096,
        server_channels_config: channels.clone(),
        client_channels_config: channels,
    }
}
pub fn now() -> Duration {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    Connecting,
    Connected,
    Disconnected(String),
    CompatibilityError,
}
impl ConnectionState {
    pub fn terminal(&self) -> bool {
        matches!(self, Self::Disconnected(_) | Self::CompatibilityError)
    }
    pub fn label(&self) -> String {
        match self {
            Self::Connecting => "CONNECTING".into(),
            Self::Connected => "CONNECTED".into(),
            Self::Disconnected(reason) => format!("DISCONNECTED: {reason} - close and join again"),
            Self::CompatibilityError => {
                "COMPATIBILITY ERROR - client/server builds must match".into()
            }
        }
    }
}
pub struct NetworkClient {
    pub connection: RenetClient,
    transport: NetcodeClientTransport,
    pub status: ConnectionState,
    pub welcome: Option<Welcome>,
    hello_sent: bool,
    hello: Message,
    elapsed: f64,
    last_snapshot: f64,
}
impl NetworkClient {
    pub fn connect(address: SocketAddr, client_id: u64) -> Result<Self, String> {
        Self::connect_with_compatibility(address, client_id, PROTOCOL_VERSION, GAMEPLAY_VERSION)
    }
    pub fn connect_with_compatibility(
        address: SocketAddr,
        client_id: u64,
        protocol: u16,
        gameplay: u64,
    ) -> Result<Self, String> {
        let bind = if address.is_ipv4() {
            "0.0.0.0:0"
        } else {
            "[::]:0"
        };
        let socket = UdpSocket::bind(bind).map_err(|e| e.to_string())?;
        let auth = ClientAuthentication::Unsecure {
            server_addr: address,
            client_id,
            user_data: None,
            protocol_id: TRANSPORT_ID,
        };
        let transport =
            NetcodeClientTransport::new(now(), auth, socket).map_err(|e| e.to_string())?;
        Ok(Self {
            connection: RenetClient::new(config()),
            transport,
            status: ConnectionState::Connecting,
            welcome: None,
            hello_sent: false,
            hello: Message::Hello { protocol, gameplay },
            elapsed: 0.,
            last_snapshot: 0.,
        })
    }
    pub fn update(&mut self, elapsed: Duration) -> Vec<Snapshot> {
        let mut snapshots = Vec::new();
        if self.status.terminal() {
            return snapshots;
        }
        self.elapsed += elapsed.as_secs_f64();
        self.connection.update(elapsed);
        if let Err(error) = self.transport.update(elapsed, &mut self.connection) {
            self.close(&error.to_string());
            return snapshots;
        }
        if self.connection.is_connected() && !self.hello_sent {
            self.connection.send_message(CONTROL, encode(&self.hello));
            self.hello_sent = true;
        }
        for channel in [CONTROL, STATE] {
            for count in 0..=64 {
                let Some(bytes) = self.connection.receive_message(channel) else {
                    break;
                };
                if count == 64 {
                    self.close("server message rate exceeded");
                    return Vec::new();
                }
                match decode(&bytes) {
                    Ok(Message::Welcome(w)) if channel == CONTROL && self.welcome.is_none() => {
                        self.welcome = Some(w);
                    }
                    Ok(Message::Reject(Rejection::Compatibility)) => {
                        self.status = ConnectionState::CompatibilityError;
                        self.transport.disconnect();
                        return Vec::new();
                    }
                    Ok(Message::Reject(reason)) => {
                        self.close(&format!("server rejected: {reason:?}"));
                        return Vec::new();
                    }
                    Ok(Message::Snapshot(s)) if channel == STATE && self.welcome.is_some() => {
                        self.last_snapshot = self.elapsed;
                        self.status = ConnectionState::Connected;
                        if snapshots.len() == SNAPSHOT_LIMIT {
                            snapshots.remove(0);
                        }
                        snapshots.push(s);
                    }
                    Ok(Message::Snapshot(_)) if channel == STATE && self.welcome.is_none() => {}
                    _ => {
                        self.close("invalid server message");
                        return Vec::new();
                    }
                }
            }
        }
        if self.elapsed - self.last_snapshot > 5.0 {
            self.close("no authoritative snapshot for five seconds");
        }
        snapshots
    }
    pub fn send_inputs(&mut self, bundle: [Option<NetInput>; 3]) {
        if self.status == ConnectionState::Connected {
            self.connection
                .send_message(STATE, encode(&Message::Inputs(bundle)));
        }
    }
    pub fn release(&mut self, through: u64) {
        if self.status == ConnectionState::Connected {
            self.connection
                .send_message(CONTROL, encode(&Message::Release { through }));
        }
    }
    pub fn flush(&mut self) {
        if !self.status.terminal()
            && self.connection.is_connected()
            && let Err(e) = self.transport.send_packets(&mut self.connection)
        {
            self.close(&e.to_string());
        }
    }
    pub fn close(&mut self, reason: &str) {
        self.status = ConnectionState::Disconnected(reason.into());
        self.transport.disconnect();
    }
}
impl Drop for NetworkClient {
    fn drop(&mut self) {
        self.transport.disconnect();
    }
}
