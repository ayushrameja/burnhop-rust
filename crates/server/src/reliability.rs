//! Opt-in real UDP impairment/soak harness. The proxy delays encrypted datagrams,
//! including Renet slices, ACKs and Netcode handshakes. It is never gameplay transport.
use crate::{Server, ServerClock};
use burnhop_gameplay_core::*;
use burnhop_protocol::{prediction::Prediction, transport::NetworkClient, *};
use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug)]
pub struct Profile {
    pub name: &'static str,
    pub rtt_ms: f64,
    pub jitter_ms: f64,
    pub loss: f64,
    pub stall: bool,
}
impl Profile {
    pub fn named(name: &str) -> Result<Self, String> {
        let (rtt_ms, jitter_ms, loss, stall) = match name {
            "baseline" => (0., 0., 0., false),
            "50" => (50., 5., 0., false),
            "100" => (100., 15., 0.01, false),
            "150" => (150., 25., 0.03, false),
            "stall" => (100., 15., 0.01, true),
            _ => return Err("profile: baseline, 50, 100, 150, stall".into()),
        };
        let name = match name {
            "50" => "50",
            "100" => "100",
            "150" => "150",
            "stall" => "stall",
            _ => "baseline",
        };
        Ok(Self {
            name,
            rtt_ms,
            jitter_ms,
            loss,
            stall,
        })
    }
}
struct Datagram {
    due: f64,
    bytes: Vec<u8>,
    upstream: bool,
}
pub struct Proxy {
    downstream: UdpSocket,
    upstream: UdpSocket,
    client: Option<SocketAddr>,
    server: SocketAddr,
    pending: Vec<Datagram>,
    seed: u64,
    profile: Profile,
    pub bytes_up: u64,
    pub bytes_down: u64,
    pub lost: u64,
    pub packets: u64,
    pub peak: usize,
    pub max_datagram: usize,
}
impl Proxy {
    pub fn new(server: SocketAddr, profile: Profile, seed: u64) -> Result<Self, String> {
        let downstream = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        let upstream = UdpSocket::bind("127.0.0.1:0").map_err(|e| e.to_string())?;
        downstream
            .set_nonblocking(true)
            .map_err(|e| e.to_string())?;
        upstream.set_nonblocking(true).map_err(|e| e.to_string())?;
        Ok(Self {
            downstream,
            upstream,
            server,
            client: None,
            pending: Vec::new(),
            seed,
            profile,
            bytes_up: 0,
            bytes_down: 0,
            lost: 0,
            packets: 0,
            peak: 0,
            max_datagram: 0,
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.downstream.local_addr().unwrap()
    }
    fn random(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.seed >> 32) as f64 / (u32::MAX as f64 + 1.)
    }
    pub fn pump(&mut self, now: f64) {
        for upstream in [true, false] {
            for _ in 0..128 {
                let mut bytes = [0; 2048];
                let result = if upstream {
                    self.downstream.recv_from(&mut bytes)
                } else {
                    self.upstream.recv_from(&mut bytes)
                };
                let (size, address) = match result {
                    Ok(value) => value,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(e) => panic!("UDP proxy receive: {e}"),
                };
                if upstream {
                    self.client = Some(address);
                    self.bytes_up += size as u64;
                } else {
                    assert_eq!(address, self.server);
                    self.bytes_down += size as u64;
                }
                self.packets += 1;
                self.max_datagram = self.max_datagram.max(size);
                if self.random() < self.profile.loss {
                    self.lost += 1;
                    continue;
                }
                // Each direction: uniform base RTT/2 ± stated RTT jitter/2.
                let delay = (self.profile.rtt_ms
                    + (self.random() * 2. - 1.) * self.profile.jitter_ms)
                    / 2000.;
                assert!(
                    self.pending.len() < 512,
                    "proxy queue exceeded declared bound"
                );
                self.pending.push(Datagram {
                    due: now + delay,
                    bytes: bytes[..size].to_vec(),
                    upstream,
                });
                self.peak = self.peak.max(self.pending.len());
            }
        }
        // Independent due times naturally reorder packets. No forced FIFO and no retries.
        let mut i = 0;
        while i < self.pending.len() {
            if self.pending[i].due > now {
                i += 1;
                continue;
            }
            let packet = self.pending.remove(i);
            if packet.upstream {
                self.upstream.send_to(&packet.bytes, self.server).unwrap();
            } else if let Some(client) = self.client {
                self.downstream.send_to(&packet.bytes, client).unwrap();
            }
        }
    }
}

pub struct Synthetic {
    pub network: NetworkClient,
    pub prediction: Option<Prediction>,
    pub clock: ServerClock,
    pub corrections: diagnostics::Samples,
    pub history_peak: usize,
    pub snapshots_peak: usize,
    pub respawns: u64,
    pub accepted: u64,
    pub ticks: u64,
}
impl Synthetic {
    pub fn new(address: SocketAddr, id: u64) -> Result<Self, String> {
        Ok(Self {
            network: NetworkClient::connect(address, id)?,
            prediction: None,
            clock: Default::default(),
            corrections: Default::default(),
            history_peak: 0,
            snapshots_peak: 0,
            respawns: 0,
            accepted: 0,
            ticks: 0,
        })
    }
    pub fn frame(&mut self, elapsed: Duration, fire: bool) {
        let snapshots = self.network.update(elapsed);
        if self.prediction.is_none()
            && let Some(welcome) = self.network.welcome
        {
            self.prediction = Some(Prediction::new(welcome));
        }
        if let Some(prediction) = &mut self.prediction {
            prediction.observe_timing(self.network.measured_rtt(), elapsed.as_secs_f64());
            for snapshot in snapshots {
                let result = prediction.reconcile(snapshot);
                if result.accepted {
                    self.accepted += 1;
                }
                if result.life_changed {
                    self.respawns += 1;
                }
                self.snapshots_peak = self.snapshots_peak.max(prediction.snapshot_len());
            }
            for _ in 0..self.clock.advance(elapsed.as_secs_f64()) {
                self.ticks += 1;
                let command = synthetic_command(prediction, self.ticks, fire);
                if let Some(bundle) = prediction.command(command) {
                    self.network.send_inputs(bundle);
                }
            }
            self.history_peak = self.history_peak.max(prediction.history_len());
        }
        self.network.flush();
    }
}
/// Ordinary input only: move toward the nearest rival, drop from platforms, aim
/// at visible bodies, reload/switch when empty. No teleports, health or ammo cheats.
pub fn synthetic_command(p: &Prediction, tick: u64, fire: bool) -> InputCommand {
    let Some(local) = p.local else {
        return InputCommand::default();
    };
    if local.require_neutral || !local.combat.alive() {
        return InputCommand::default();
    }
    let Some(state) = p.latest.map(|s| s.state) else {
        return InputCommand::default();
    };
    let target = state
        .actors
        .iter()
        .flatten()
        .filter(|a| a.id != local.id && a.combat.alive())
        .min_by(|a, b| {
            let distance = |a: &Actor| {
                (a.movement.body.x - local.movement.body.x)
                    .hypot(a.movement.body.y - local.movement.body.y)
            };
            distance(a).total_cmp(&distance(b))
        });
    let Some(target) = target else {
        return InputCommand::default();
    };
    let dx = target.movement.body.x - local.movement.body.x;
    let mut command = InputCommand {
        aim_at: Some(body_center(target.movement.body)),
        fire_held: fire,
        ..Default::default()
    };
    // Alternating traversal while staying around opponents produces moving combat.
    command.move_x = if !local.movement.grounded || local.movement.body.y < 1100. || dx.abs() > 260.
    {
        if dx > 0. {
            MoveAxis::Right
        } else {
            MoveAxis::Left
        }
    } else if tick % 240 < 60 {
        MoveAxis::Right
    } else if tick % 240 < 120 {
        MoveAxis::Left
    } else {
        MoveAxis::Idle
    };
    // Escape authored platform tops by walking to the nearer edge.
    if local.movement.grounded && local.movement.body.y < 1100. {
        command.move_x = MoveAxis::Right;
    }
    command.jump_pressed = tick % 240 == 10;
    command.jet_pressed = tick % 480 == 80;
    command.jet_held = (80..95).contains(&(tick % 480));
    let w = local.combat.weapon();
    command.reload_pressed = w.ammo == 0 && tick.is_multiple_of(3);
    command.select_weapon =
        if local.combat.selected == WeaponId::Pistol && local.combat.weapons[1].ammo > 0 {
            Some(WeaponId::M416)
        } else if w.ammo == 0 && matches!(w.reserve, Reserve::Rounds(0)) {
            Some(if w.id == WeaponId::M416 {
                WeaponId::Pistol
            } else {
                WeaponId::M416
            })
        } else {
            None
        };
    command
}

pub fn run(seconds: f64, profile: Profile, count: usize, churn: bool) -> Result<(), String> {
    if !(2..=8).contains(&count) || !(2.0..=7200.).contains(&seconds) {
        return Err("players 2..8; duration 2..7200 seconds".into());
    }
    let mut server = Server::bind("127.0.0.1:0".parse().unwrap())?;
    let mut proxies: Vec<_> = (0..count)
        .map(|i| Proxy::new(server.address(), profile, 0xB08 + i as u64))
        .collect::<Result<_, _>>()?;
    let mut clients: Vec<_> = proxies
        .iter()
        .enumerate()
        .map(|(i, proxy)| Synthetic::new(proxy.address(), 100 + i as u64))
        .collect::<Result<_, _>>()?;
    let mut clock = ServerClock::default();
    let began = Instant::now();
    let mut last = began;
    let mut last_client = began;
    let mut next_churn = 45.;
    let mut replacement = None;
    let mut joins = count;
    let mut deaths = 0;
    let mut respawns = 0;
    let mut shots = 0;
    let mut full = false;
    let mut start_stats = None;
    let mut resumed = false;
    let mut report_at = 5.;
    let mut rss_initial = None;
    let mut rss_peak = 0;
    let mut retired_correction_max: f64 = 0.;
    let mut retired_history_peak = 0;
    let mut reconciliations = 0;
    let mut corrections = 0;
    while began.elapsed().as_secs_f64() < seconds {
        let now = Instant::now();
        let elapsed = now.duration_since(last);
        last = now;
        let age = began.elapsed().as_secs_f64();
        for proxy in &mut proxies {
            proxy.pump(age);
        }
        let stalled = profile.stall && (8.0..9.0).contains(&age);
        if !stalled {
            let client_elapsed = now.duration_since(last_client);
            last_client = now;
            for (i, client) in clients.iter_mut().enumerate() {
                if replacement.is_none_or(|(index, _)| i != index) {
                    client.frame(client_elapsed, true);
                }
            }
        }
        server.poll(elapsed)?;
        for _ in 0..clock.advance(elapsed.as_secs_f64()) {
            let events = server.tick();
            deaths += events.died.iter().filter(|&&v| v).count();
            respawns += events.respawned.iter().filter(|&&v| v).count();
            shots += events.shots.iter().filter(|s| s.is_some()).count();
            if server.state.tick.is_multiple_of(2) {
                server.snapshot();
            }
        }
        server.flush();
        if server.player_count() == count {
            full = true;
        }
        if age > 4. && start_stats.is_none() {
            start_stats = Some(server.input_stats());
        }
        if age > 4. && replacement.is_none() && server.player_count() != count {
            return Err(format!(
                "unexpected player count {} at {age:.3}; statuses {:?}",
                server.player_count(),
                clients
                    .iter()
                    .map(|c| &c.network.status)
                    .collect::<Vec<_>>()
            ));
        }
        if profile.stall && (9.0..11.0).contains(&age) {
            resumed |= server
                .state
                .actors
                .iter()
                .flatten()
                .all(|a| a.combat.alive() == (a.combat.health > 0))
                && clients.iter().all(|c| {
                    c.prediction
                        .as_ref()
                        .and_then(|p| p.latest)
                        .is_some_and(|s| s.last_applied + 12 > s.ack)
                });
        }
        for actor in server.state.actors.iter().flatten() {
            let b = actor.movement.body;
            assert!(
                b.x.is_finite()
                    && b.y.is_finite()
                    && b.x >= -0.01
                    && b.x <= 2364.01
                    && b.y >= -0.01
                    && b.y <= 1152.01
            );
        }
        if churn && age >= next_churn && replacement.is_none() {
            let index = (joins - count) % count;
            clients[index].network.close("scheduled soak departure");
            replacement = Some((index, age + 0.6));
            next_churn += 45.;
        }
        if let Some((index, due)) = replacement
            && age >= due
        {
            let old = clients[index].network.welcome.unwrap();
            assert!(server.state.actors[old.actor.index()].is_none());
            if let Some(p) = &clients[index].prediction {
                retired_correction_max = retired_correction_max.max(p.corrections.max);
                retired_history_peak = retired_history_peak.max(clients[index].history_peak);
                reconciliations += p.corrections.count;
                corrections += p.corrections.nonzero;
            }
            clients[index] = Synthetic::new(proxies[index].address(), 100 + joins as u64)?;
            joins += 1;
            replacement = None;
            // Handshake grace before occupancy assertions (clock still advances).
            while clients[index].network.welcome.is_none() {
                let now = Instant::now();
                let dt = now.duration_since(last);
                last = now;
                last_client = now;
                for proxy in &mut proxies {
                    proxy.pump(began.elapsed().as_secs_f64());
                }
                for client in &mut clients {
                    client.frame(dt, true);
                }
                server.poll(dt)?;
                for _ in 0..clock.advance(dt.as_secs_f64()) {
                    let events = server.tick();
                    deaths += events.died.iter().filter(|&&v| v).count();
                    respawns += events.respawned.iter().filter(|&&v| v).count();
                    shots += events.shots.iter().flatten().count();
                    if server.state.tick.is_multiple_of(2) {
                        server.snapshot();
                    }
                }
                server.flush();
                if began.elapsed().as_secs_f64() > due + 3. {
                    return Err("replacement handshake timeout".into());
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            let new = clients[index].network.welcome.unwrap();
            assert_eq!(new.actor, old.actor);
            assert_ne!(new.generation, old.generation);
            let actor = server.state.actors[new.actor.index()].unwrap();
            assert_eq!((actor.kills, actor.deaths), (0, 0));
        }
        if age >= report_at {
            if let Some(rss) = rss_kib() {
                rss_initial.get_or_insert(rss);
                rss_peak = rss_peak.max(rss);
            }
            eprintln!("RELIABILITY memory_rss_kib={:?}", rss_kib());
            eprintln!(
                "RELIABILITY progress seconds={age:.1} players={} kills={deaths} respawns={respawns} dropped={} queue_peak={}",
                server.player_count(),
                clock.dropped_ticks,
                server.queue_peak()
            );
            report_at += 30.;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    let duration = began.elapsed().as_secs_f64();
    let stats = server.input_stats();
    let start = start_stats.unwrap_or_default();
    let applied = stats.applied - start.applied;
    let missing = stats.missing - start.missing;
    let correction_p95 = clients
        .iter()
        .filter_map(|c| c.prediction.as_ref())
        .map(|p| p.corrections.percentile(0.95))
        .fold(0., f64::max);
    let correction_max = clients
        .iter()
        .filter_map(|c| c.prediction.as_ref())
        .map(|p| p.corrections.max)
        .fold(retired_correction_max, f64::max);
    let rss_final = rss_kib();
    if let Some(rss) = rss_final {
        rss_peak = rss_peak.max(rss);
    }
    for p in clients.iter().filter_map(|c| c.prediction.as_ref()) {
        reconciliations += p.corrections.count;
        corrections += p.corrections.nonzero;
    }
    let initial_rss = rss_initial.map_or("null".into(), |v| v.to_string());
    let final_rss = rss_final.map_or("null".into(), |v| v.to_string());
    println!(
        "{{\"profile\":\"{}\",\"duration_s\":{duration:.3},\"players\":{count},\"joins\":{joins},\"ticks\":{},\"shots\":{shots},\"deaths\":{deaths},\"respawns\":{respawns},\"applied\":{applied},\"missing\":{missing},\"late_copies\":{},\"tick_us_p95\":{:.3},\"tick_us_max\":{:.3},\"dropped_ticks\":{},\"snapshot_max\":{},\"app_sent_bytes\":{},\"app_received_bytes\":{},\"udp_up_bytes\":{},\"udp_down_bytes\":{},\"packet_loss_count\":{},\"packets\":{},\"datagram_max\":{},\"proxy_peak\":{},\"queue_peak\":{},\"history_peak\":{},\"correction_p95_px\":{correction_p95:.3},\"correction_max_px\":{correction_max:.3},\"stall_recovered\":{resumed},\"reconciliations\":{reconciliations},\"corrections\":{corrections},\"rss_initial_kib\":{initial_rss},\"rss_final_kib\":{final_rss},\"rss_peak_kib\":{rss_peak}}}",
        profile.name,
        server.state.tick,
        stats.late,
        server.tick_work_us.percentile(0.95),
        server.tick_work_us.max,
        clock.dropped_ticks,
        server.snapshot_max,
        server.application_sent,
        server.application_received,
        proxies.iter().map(|p| p.bytes_up).sum::<u64>(),
        proxies.iter().map(|p| p.bytes_down).sum::<u64>(),
        proxies.iter().map(|p| p.lost).sum::<u64>(),
        proxies.iter().map(|p| p.packets).sum::<u64>(),
        proxies.iter().map(|p| p.max_datagram).max().unwrap_or(0),
        proxies.iter().map(|p| p.peak).max().unwrap_or(0),
        server.queue_peak(),
        clients
            .iter()
            .map(|c| c.history_peak)
            .max()
            .unwrap_or(0)
            .max(retired_history_peak)
    );
    if !full
        || (seconds > 15. && (deaths == 0 || respawns == 0))
        || (profile.stall && seconds > 11. && !resumed)
    {
        return Err("occupancy/combat/stall pass criterion failed".into());
    }
    Ok(())
}

fn rss_kib() -> Option<u64> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        let output = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;
        String::from_utf8(output.stdout).ok()?.trim().parse().ok()
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        None
    }
}
