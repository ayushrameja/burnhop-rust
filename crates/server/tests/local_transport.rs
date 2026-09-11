//! Synthetic clients over actual local UDP sockets. Injected application-message
//! delay/loss is deterministic test scheduling, not a measured internet network.
use burnhop_gameplay_core::*;
use burnhop_protocol::{
    prediction::Prediction,
    transport::{ConnectionState, NetworkClient},
    *,
};
use burnhop_server::{Server, ServerClock};
use std::{collections::VecDeque, time::Duration};
const DELTA: Duration = Duration::from_nanos(16_666_667);
struct Client {
    network: NetworkClient,
    prediction: Option<Prediction>,
    outgoing: VecDeque<(u64, [Option<NetInput>; 3])>,
    incoming: VecDeque<(u64, Snapshot)>,
    life_changes: usize,
    fire: bool,
}
impl Client {
    fn new(server: &Server, id: u64) -> Self {
        Self {
            network: NetworkClient::connect(server.address(), id).unwrap(),
            prediction: None,
            outgoing: VecDeque::new(),
            incoming: VecDeque::new(),
            life_changes: 0,
            fire: false,
        }
    }
    fn frame(&mut self, frame: u64, impaired: bool) {
        let snapshots = self.network.update(DELTA);
        if self.prediction.is_none()
            && let Some(w) = self.network.welcome
        {
            self.prediction = Some(Prediction::new(w));
        }
        for snapshot in snapshots {
            if impaired && frame.is_multiple_of(11) {
                continue;
            }
            self.incoming
                .push_back((frame + if impaired { frame % 4 } else { 0 }, snapshot));
        }
        let mut waiting = VecDeque::new();
        while let Some((due, snapshot)) = self.incoming.pop_front() {
            if due > frame {
                waiting.push_back((due, snapshot));
                continue;
            }
            if let Some(p) = &mut self.prediction {
                let result = p.reconcile(snapshot);
                if result.life_changed {
                    self.life_changes += 1;
                    self.network.release(p.next_sequence.saturating_sub(1));
                }
            }
        }
        self.incoming = waiting;
        if let Some(p) = &mut self.prediction {
            let mut command = InputCommand::default();
            if let Some(snapshot) = p.latest {
                let id = p.welcome.actor;
                if let Some(other) = snapshot.state.actors[id.other().index()] {
                    if self.fire
                        && p.local
                            .is_some_and(|a| !a.require_neutral && a.combat.alive())
                    {
                        command.aim_at = Some(body_center(other.movement.body));
                        command.fire_held = true;
                        command.select_weapon = Some(WeaponId::M416);
                        command.reload_pressed = p.local.unwrap().combat.weapon().ammo == 0;
                    }
                    if (30..50).contains(&frame) {
                        command.move_x = MoveAxis::Right;
                    }
                    if (50..70).contains(&frame) {
                        command.move_x = MoveAxis::Left;
                    }
                }
            }
            if let Some(bundle) = p.command(command)
                && !(impaired && frame.is_multiple_of(13))
            {
                self.outgoing
                    .push_back((frame + if impaired { (frame * 3) % 4 } else { 0 }, bundle));
            }
        }
        let mut waiting = VecDeque::new();
        while let Some((due, bundle)) = self.outgoing.pop_front() {
            if due > frame {
                waiting.push_back((due, bundle));
            } else {
                self.network.send_inputs(bundle);
            }
        }
        self.outgoing = waiting;
        assert!(self.incoming.len() <= 8 && self.outgoing.len() <= 8);
        self.network.flush();
    }
}
fn frame(server: &mut Server, clients: &mut [&mut Client], tick: u64, impaired: bool) {
    for client in clients {
        client.frame(tick, impaired);
    }
    server.poll(DELTA).unwrap();
    server.tick();
    server.snapshot();
    server.flush();
    // Give real UDP I/O a scheduling opportunity; simulation time is explicit.
    std::thread::sleep(Duration::from_micros(100));
}
fn duel(impaired: bool) {
    let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut a = Client::new(&server, 100);
    let mut b = Client::new(&server, 200);
    for tick in 0..100 {
        frame(&mut server, &mut [&mut a, &mut b], tick, impaired);
    }
    assert_eq!(server.player_count(), 2);
    assert_eq!(a.network.status, ConnectionState::Connected);
    assert_eq!(b.network.status, ConnectionState::Connected);
    assert_ne!(
        a.prediction.as_ref().unwrap().welcome.actor,
        b.prediction.as_ref().unwrap().welcome.actor
    );
    assert!(server.state.actors[0].unwrap().movement.body.x != PRACTICE_ARENA.spawn.x);
    a.fire = true;
    let mut a_killed = false;
    let mut b_killed = false;
    for tick in 100..1300 {
        frame(&mut server, &mut [&mut a, &mut b], tick, impaired);
        assert_eq!(
            server.player_count(),
            2,
            "clients: {:?}, {:?}",
            a.network.status,
            b.network.status
        );
        let ai = a.prediction.as_ref().unwrap().welcome.actor.index();
        let bi = b.prediction.as_ref().unwrap().welcome.actor.index();
        let aa = server.state.actors[ai].unwrap();
        let bb = server.state.actors[bi].unwrap();
        if aa.kills > 0 {
            a_killed = true;
            a.fire = false;
        }
        if a_killed && bb.combat.alive() && bb.deaths > 0 {
            b.fire = true;
        }
        if bb.kills > 0 {
            b_killed = true;
            b.fire = false;
        }
        if a_killed
            && b_killed
            && aa.combat.alive()
            && bb.combat.alive()
            && a.life_changes >= 2
            && b.life_changes >= 2
        {
            break;
        }
    }
    assert!(
        a_killed && b_killed,
        "both players must land lethal shared-core damage"
    );
    assert!(a.life_changes >= 2 && b.life_changes >= 2);
    let old = b.prediction.as_ref().unwrap().welcome;
    b.network.close("test leave");
    for tick in 1400..1410 {
        frame(&mut server, &mut [&mut a], tick, false);
    }
    assert_eq!(server.player_count(), 1);
    assert!(server.state.actors[old.actor.index()].is_none());
    let mut fresh = Client::new(&server, 300);
    for tick in 1410..1450 {
        frame(&mut server, &mut [&mut a, &mut fresh], tick, false);
    }
    assert_eq!(server.player_count(), 2);
    assert_eq!(fresh.network.status, ConnectionState::Connected);
    let new = fresh.prediction.as_ref().unwrap().welcome;
    assert_eq!(new.actor, old.actor);
    assert_ne!(new.generation, old.generation);
    // A keeps pumping transport but stops submitting commands: server removes it.
    for _ in 0..200 {
        a.network.update(DELTA);
        a.network.flush();
        fresh.frame(1500, false);
        server.poll(DELTA).unwrap();
        server.tick();
        server.snapshot();
        server.flush();
    }
    assert!(server.state.actors[new.actor.other().index()].is_none());
}
#[test]
fn real_udp_server_two_synthetic_clients_duel_disconnect_and_fresh_join() {
    duel(false);
}
#[test]
fn real_udp_with_injected_zero_to_three_tick_jitter_loss_and_reordering() {
    duel(true);
}
#[test]
fn real_transport_rejects_incompatible_clients_before_actor_assignment() {
    let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut client = NetworkClient::connect_with_compatibility(
        server.address(),
        900,
        PROTOCOL_VERSION + 1,
        GAMEPLAY_VERSION,
    )
    .unwrap();
    for _ in 0..100 {
        client.update(DELTA);
        client.flush();
        server.poll(DELTA).unwrap();
        server.tick();
        server.snapshot();
        server.flush();
    }
    assert_eq!(server.player_count(), 0);
    assert_eq!(client.status, ConnectionState::CompatibilityError);
}
#[test]
fn real_transport_rejects_actor_spoofing_and_malformed_messages_cleans_up() {
    for spoof in [true, false] {
        let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let mut client = Client::new(&server, 901);
        for tick in 0..40 {
            frame(&mut server, &mut [&mut client], tick, false);
        }
        let p = client.prediction.as_ref().unwrap();
        let bytes = if spoof {
            encode(&Message::Inputs([
                Some(NetInput {
                    actor: p.welcome.actor.other(),
                    sequence: p.next_sequence,
                    command: InputCommand {
                        tick: p.next_sequence,
                        ..Default::default()
                    },
                }),
                None,
                None,
            ]))
        } else {
            vec![0; MAX_MESSAGE_BYTES + 1]
        };
        client.network.connection.send_message(STATE, bytes);
        client.network.flush();
        for tick in 40..60 {
            frame(&mut server, &mut [&mut client], tick, false);
        }
        assert_eq!(server.player_count(), 0);
        assert!(client.network.status.terminal());
    }
}
#[test]
fn headless_clock_bounds_overload_and_does_not_replay_elapsed_backlog() {
    let mut clock = ServerClock::default();
    assert_eq!(clock.advance(2.0), 5);
    assert!(clock.dropped_ticks >= 114);
    assert_eq!(clock.advance(0.0), 0);
    assert_eq!(clock.advance(DT), 1);
    assert_eq!(clock.advance(f64::NAN), 0);
}

#[test]
fn real_transport_rejects_a_third_player_and_an_input_message_flood() {
    let mut server = Server::bind("127.0.0.1:0".parse().unwrap()).unwrap();
    let mut a = Client::new(&server, 1001);
    let mut b = Client::new(&server, 1002);
    for tick in 0..40 {
        frame(&mut server, &mut [&mut a, &mut b], tick, false);
    }
    let mut third = Client::new(&server, 1003);
    for tick in 40..80 {
        frame(&mut server, &mut [&mut a, &mut b, &mut third], tick, false);
    }
    assert_eq!(server.player_count(), 2);
    assert!(third.network.status.terminal());
    // Valid-shaped duplicates still consume the message-rate allowance.
    for _ in 0..30 {
        a.network
            .connection
            .send_message(STATE, encode(&Message::Inputs([None; 3])));
    }
    a.network.flush();
    for tick in 80..100 {
        frame(&mut server, &mut [&mut a, &mut b], tick, false);
    }
    assert_eq!(server.player_count(), 1);
    assert!(a.network.status.terminal());
    assert_eq!(b.network.status, ConnectionState::Connected);
}
