//! Headless two-player authoritative server; the binary only owns the wall clock.
use burnhop_gameplay_core::*;
use burnhop_protocol::{
    transport::{config, now},
    *,
};
use renet::{RenetServer, ServerEvent};
use renet_netcode::{NetcodeServerTransport, ServerAuthentication, ServerConfig};
use std::{
    collections::BTreeMap,
    net::{SocketAddr, UdpSocket},
    time::Duration,
};

struct Peer {
    queue: Option<InputQueue>,
    age: f64,
    silent: f64,
    rejected: Option<f64>,
    messages: f64,
}
pub struct Server {
    pub state: MatchState,
    connection: RenetServer,
    transport: NetcodeServerTransport,
    peers: BTreeMap<u64, Peer>,
    generation: u64,
    shots: [Option<ConfirmedShot>; 2],
    pub diagnostics: bool,
}
impl Server {
    pub fn bind(address: SocketAddr) -> Result<Self, String> {
        let socket = UdpSocket::bind(address).map_err(|e| e.to_string())?;
        let actual = socket.local_addr().map_err(|e| e.to_string())?;
        let transport = NetcodeServerTransport::new(
            ServerConfig {
                current_time: now(),
                max_clients: 4,
                protocol_id: TRANSPORT_ID,
                public_addresses: vec![actual],
                authentication: ServerAuthentication::Unsecure,
            },
            socket,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self {
            state: MatchState::default(),
            connection: RenetServer::new(config()),
            transport,
            peers: BTreeMap::new(),
            generation: 0,
            shots: [None; 2],
            diagnostics: false,
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.transport.addresses()[0]
    }
    pub fn player_count(&self) -> usize {
        self.state.actors.iter().flatten().count()
    }
    fn remove_actor(&mut self, client: u64) {
        if let Some(peer) = self.peers.get_mut(&client)
            && let Some(queue) = peer.queue.take()
        {
            self.state.actors[queue.actor.index()] = None;
            self.shots[queue.actor.index()] = None;
            if self.diagnostics {
                println!(
                    "SERVER leave actor={:?} tick={}",
                    queue.actor, self.state.tick
                );
            }
        }
    }
    fn reject(&mut self, client: u64, reason: Rejection) {
        self.remove_actor(client);
        if let Some(peer) = self.peers.get_mut(&client)
            && peer.rejected.is_none()
        {
            peer.rejected = Some(0.0);
            self.connection
                .send_message(client, CONTROL, encode(&Message::Reject(reason)));
        }
    }
    /// Transport clocks use actual elapsed time, including dropped overload time.
    pub fn poll(&mut self, elapsed: Duration) -> Result<(), String> {
        self.connection.update(elapsed);
        self.transport
            .update(elapsed, &mut self.connection)
            .map_err(|e| e.to_string())?;
        while let Some(event) = self.connection.get_event() {
            match event {
                ServerEvent::ClientConnected { client_id } => {
                    self.peers.insert(
                        client_id,
                        Peer {
                            queue: None,
                            age: 0.,
                            silent: 0.,
                            rejected: None,
                            messages: 24.,
                        },
                    );
                }
                ServerEvent::ClientDisconnected { client_id, .. } => {
                    self.remove_actor(client_id);
                    self.peers.remove(&client_id);
                }
            }
        }
        let seconds = elapsed.as_secs_f64();
        for client in self.connection.clients_id() {
            let Some(peer) = self.peers.get_mut(&client) else {
                continue;
            };
            peer.age += seconds;
            peer.silent += seconds;
            peer.messages = (peer.messages + seconds * 120.).min(24.);
            if let Some(queue) = &mut peer.queue {
                queue.refill(seconds);
            }
            if let Some(age) = &mut peer.rejected {
                *age += seconds;
                if *age > 0.5 {
                    self.connection.disconnect(client);
                }
                continue;
            }
            if peer.silent > 3.0 || (peer.queue.is_none() && peer.age > 3.0) {
                self.reject(client, Rejection::Timeout);
                continue;
            }
            for channel in [CONTROL, STATE] {
                for count in 0..=16 {
                    let Some(bytes) = self.connection.receive_message(client, channel) else {
                        break;
                    };
                    let peer = self.peers.get_mut(&client).expect("registered peer");
                    peer.messages -= 1.;
                    if count == 16 || peer.messages < 0. {
                        self.reject(client, Rejection::RateLimit);
                        break;
                    }
                    match decode(&bytes) {
                        Ok(message) => self.handle(client, channel, message),
                        Err(_) => self.reject(client, Rejection::InvalidInput),
                    }
                    if self.peers[&client].rejected.is_some() {
                        break;
                    }
                }
                if self.peers[&client].rejected.is_some() {
                    break;
                }
            }
        }
        Ok(())
    }
    fn handle(&mut self, client: u64, channel: u8, message: Message) {
        match message {
            Message::Hello { protocol, gameplay } if channel == CONTROL => {
                if !compatible(protocol, gameplay) {
                    self.reject(client, Rejection::Compatibility);
                    return;
                }
                if self.peers[&client].queue.is_some() {
                    return;
                }
                let Some(id) = ActorId::ALL
                    .into_iter()
                    .find(|id| self.state.actors[id.index()].is_none())
                else {
                    self.reject(client, Rejection::Full);
                    return;
                };
                self.generation += 1;
                let welcome = Welcome {
                    actor: id,
                    generation: self.generation,
                    start_tick: self.state.tick + INPUT_LEAD,
                };
                self.state.actors[id.index()] =
                    Some(Actor::new(id, self.generation, &PRACTICE_ARENA));
                self.peers.get_mut(&client).unwrap().queue =
                    Some(InputQueue::new(id, welcome.start_tick));
                self.connection
                    .send_message(client, CONTROL, encode(&Message::Welcome(welcome)));
                if self.diagnostics {
                    println!(
                        "SERVER join actor={id:?} generation={} tick={}",
                        self.generation, self.state.tick
                    );
                }
            }
            Message::Inputs(bundle) if channel == STATE => {
                let peer = self.peers.get_mut(&client).unwrap();
                let Some(queue) = &mut peer.queue else {
                    self.reject(client, Rejection::InvalidInput);
                    return;
                };
                for input in bundle.into_iter().flatten() {
                    match queue.admit(input, self.state.tick) {
                        Admission::Accepted => peer.silent = 0.,
                        Admission::DuplicateOrStale => {}
                        Admission::RateLimited => {
                            self.reject(client, Rejection::RateLimit);
                            return;
                        }
                        _ => {
                            self.reject(client, Rejection::InvalidInput);
                            return;
                        }
                    }
                }
            }
            Message::Release { through } if channel == CONTROL => {
                let peer = self.peers.get_mut(&client).unwrap();
                let Some(queue) = &mut peer.queue else {
                    self.reject(client, Rejection::InvalidInput);
                    return;
                };
                if !queue.release(through, self.state.tick) {
                    self.reject(client, Rejection::InvalidInput);
                    return;
                }
                // Next tick clears latches; release is also an immediate barrier to old commands.
                if let Some(actor) = &mut self.state.actors[queue.actor.index()] {
                    actor.require_neutral = true;
                }
            }
            _ => self.reject(client, Rejection::InvalidInput),
        }
    }
    pub fn tick(&mut self) -> MatchEvents {
        let mut inputs = [InputCommand {
            tick: self.state.tick,
            release_input: true,
            ..Default::default()
        }; 2];
        for peer in self.peers.values_mut() {
            if let Some(queue) = &mut peer.queue {
                inputs[queue.actor.index()] = queue.take_tick(self.state.tick);
            }
        }
        let tick = self.state.tick;
        let events =
            step_match(&mut self.state, inputs, &PRACTICE_ARENA).expect("server assigns ticks");
        for (i, shot) in events.shots.iter().enumerate() {
            if let Some(shot) = shot {
                self.shots[i] = Some(ConfirmedShot { tick, shot: *shot });
            }
        }
        if self.diagnostics
            && (events.shots.iter().any(Option::is_some) || events.respawned.iter().any(|v| *v))
        {
            println!(
                "SERVER tick={tick} events={events:?} health={:?}",
                self.state.actors.map(|a| a.map(|a| a.combat.health))
            );
        }
        events
    }
    /// 30 Hz snapshots (binary calls this every second simulation tick).
    pub fn snapshot(&mut self) {
        for (&client, peer) in &self.peers {
            if let Some(queue) = &peer.queue {
                self.connection.send_message(
                    client,
                    STATE,
                    encode(&Message::Snapshot(Snapshot {
                        state: self.state,
                        ack: queue.ack,
                        last_applied: queue.last_applied,
                        shots: self.shots,
                    })),
                );
            }
        }
    }
    pub fn flush(&mut self) {
        self.transport.send_packets(&mut self.connection);
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.transport.disconnect_all(&mut self.connection);
    }
}

/// Admit at most 250 ms; execute at most five ticks, drop whole excess ticks.
/// Simulation slows under overload. Never enlarge DT or run an unbounded catch-up.
#[derive(Default)]
pub struct ServerClock {
    accumulator: f64,
    pub dropped_ticks: u64,
}
impl ServerClock {
    pub fn advance(&mut self, elapsed: f64) -> usize {
        if !elapsed.is_finite() || elapsed < 0. {
            return 0;
        }
        self.dropped_ticks += ((elapsed - elapsed.min(0.25)) / DT).floor() as u64;
        self.accumulator += elapsed.min(0.25);
        let due = ((self.accumulator + 1e-10) / DT).floor() as usize;
        let steps = due.min(5);
        self.accumulator = (self.accumulator - due as f64 * DT).max(0.);
        self.dropped_ticks += (due - steps) as u64;
        steps
    }
}
