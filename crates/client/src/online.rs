//! Native multiplayer adapter: movement prediction and authoritative presentation.
use crate::Playground;
use burnhop_gameplay_core::*;
use burnhop_protocol::{prediction::Prediction, transport::NetworkClient};
use std::{net::SocketAddr, time::Duration};

pub struct Online {
    pub network: NetworkClient,
    pub prediction: Option<Prediction>,
    pub actors: [Option<Actor>; MAX_PLAYERS],
    snapshot_age: f64,
    was_focused: bool,
    script: Option<NativeRoute>,
}
impl Online {
    pub fn new(address: SocketAddr, script: bool) -> Result<Self, String> {
        let bytes = renet_id();
        Ok(Self {
            network: NetworkClient::connect(address, bytes)?,
            prediction: None,
            actors: [None; MAX_PLAYERS],
            snapshot_age: 0.,
            was_focused: true,
            script: script.then(NativeRoute::default),
        })
    }
    pub fn label(&self) -> String {
        let actor = self.network.welcome.map_or(String::new(), |w| {
            format!(" - Player {}", w.actor.index() + 1)
        });
        format!("{}{actor}", self.network.status.label())
    }
    pub fn cancel_script(&mut self) {
        if let Some(script) = &mut self.script {
            script.complete = true;
        }
        if let Some(prediction) = &mut self.prediction {
            self.network
                .release(prediction.next_sequence.saturating_sub(1));
            prediction.clear_intent();
        }
    }
    pub fn scripted(&self) -> bool {
        self.script.as_ref().is_some_and(|s| !s.complete)
    }
}
fn renet_id() -> u64 {
    // Unique process/session identity only; this is not account authentication.
    let time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    (time as u64) ^ (u64::from(std::process::id()) << 32)
}
pub fn simulate(game: &mut Playground, elapsed: f64) {
    let mut online = game.online.take().expect("online resource");
    online.snapshot_age += elapsed;
    let was_status = online.network.status.clone();
    let snapshots = online
        .network
        .update(Duration::from_secs_f64(elapsed.max(0.)));
    if online.prediction.is_none()
        && let Some(welcome) = online.network.welcome
    {
        online.prediction = Some(Prediction::new(welcome));
        game.input.clear();
        game.snap_camera = true;
        println!(
            "ONLINE assigned actor={:?} generation={} start_tick={}",
            welcome.actor, welcome.generation, welcome.start_tick
        );
    }
    if let Some(prediction) = &mut online.prediction {
        prediction.observe_timing(online.network.measured_rtt(), elapsed);
        for snapshot in snapshots {
            let result = prediction.reconcile(snapshot);
            if !result.accepted {
                continue;
            }
            online.snapshot_age = 0.;
            if result.life_changed {
                game.input.clear();
                online
                    .network
                    .release(prediction.next_sequence.saturating_sub(1));
                game.snap_camera = true;
                if let Some(script) = &mut online.script {
                    script.neutral_ticks = 12;
                }
                println!(
                    "ONLINE lifecycle actor={:?} tick={} life={:?} deaths={}",
                    prediction.welcome.actor,
                    snapshot.state.tick,
                    prediction.local.unwrap().combat.life,
                    prediction.local.unwrap().deaths
                );
            }
            for (i, changed) in result.changed.iter().enumerate() {
                if *changed {
                    game.feedback.clear_actor(ActorId::ALL[i]);
                }
            }
            game.feedback.record_shots(result.shots.into_iter());
        }
    }
    let scripted = online.scripted();
    if online.was_focused && !game.focused && !scripted {
        game.input.clear();
        if let Some(prediction) = &mut online.prediction {
            online
                .network
                .release(prediction.next_sequence.saturating_sub(1));
            prediction.clear_intent();
        }
        println!("ONLINE focus lost: input released; server continues");
    }
    online.was_focused = game.focused;
    // Multiplayer keeps polling and sending neutral commands while unfocused.
    let (ticks, _) = game.clock.advance(elapsed);
    game.alpha = 1.;
    let ticks = if online.network.status.terminal() {
        0
    } else {
        ticks
    };
    for _ in 0..ticks {
        if let Some(prediction) = &mut online.prediction {
            let mut command = if game.focused {
                game.input.command(prediction.next_sequence)
            } else {
                InputCommand {
                    release_input: true,
                    ..Default::default()
                }
            };
            if let Some(script) = &mut online.script
                && !script.complete
            {
                command = script.command(prediction);
            }
            command.reset = false;
            if let Some(bundle) = prediction.command(command) {
                online.network.send_inputs(bundle);
            }
        }
    }
    if let Some(prediction) = &online.prediction
        && let Some(local) = prediction.local
    {
        game.world.player = local.movement;
        game.previous = local.movement;
        if let Some(snapshot) = prediction.latest {
            game.world.tick = snapshot.state.tick;
            // Combat HUD always comes from authoritative state, never replayed input.
            let confirmed = snapshot.state.actors[prediction.welcome.actor.index()].unwrap();
            game.combat.player = confirmed.combat;
            game.combat.kills = confirmed.kills;
            game.combat.deaths = confirmed.deaths;
            let render_tick = snapshot.state.tick as f64 + (online.snapshot_age * 60.).min(6.) - 6.;
            online.actors = ActorId::ALL.map(|id| {
                if id == prediction.welcome.actor {
                    Some(Actor {
                        combat: confirmed.combat,
                        ..local
                    })
                } else {
                    prediction.remote_at(id, render_tick)
                }
            });
        }
    }
    online.network.flush();
    if online.network.status != was_status {
        println!("ONLINE {}", online.label());
    }
    if online.network.status.terminal() {
        online.actors = [None; MAX_PLAYERS];
        game.input.clear();
        game.feedback = Default::default();
        if let Some(prediction) = &mut online.prediction {
            prediction.clear_intent();
        }
        game.world.player.thrusting = false;
    }
    game.online = Some(online);
}
/// Tick-input route in TWO ACTUAL rendered clients, using the ordinary UDP path.
/// Explicit opt-in runs injected commands even in background; ordinary device
/// controls always clear on focus loss. This is not a human input/feel test.
#[derive(Default)]
struct NativeRoute {
    ticks: u64,
    neutral_ticks: u16,
    complete: bool,
}
impl NativeRoute {
    fn command(&mut self, prediction: &Prediction) -> InputCommand {
        let mut command = InputCommand::default();
        let Some(snapshot) = prediction.latest else {
            return command;
        };
        let Some(local) = snapshot.state.actors[prediction.welcome.actor.index()] else {
            return command;
        };
        let Some(remote) = snapshot
            .state
            .actors
            .iter()
            .flatten()
            .find(|a| a.id != local.id)
            .copied()
        else {
            return command;
        };
        self.ticks += 1;
        if local.kills >= 1 && local.deaths >= 1 && local.combat.alive() && remote.combat.alive() {
            self.complete = true;
            println!(
                "ONLINE PLAYTEST COMPLETE actor={:?}: movement, jump, jet, fire, kill, death, respawn through real UDP; injected input",
                local.id
            );
            return InputCommand {
                release_input: true,
                ..Default::default()
            };
        }
        if self.neutral_ticks > 0 {
            self.neutral_ticks -= 1;
            return command;
        }
        if self.ticks < 50 {
            command.move_x = MoveAxis::Right;
        } else if self.ticks < 100 {
            command.move_x = MoveAxis::Left;
        }
        if self.ticks == 110 {
            command.jump_pressed = true;
        }
        // Visual smoke coverage: a short jet pulse through ordinary input.
        // This opt-in route does not alter live controls or simulation rules.
        command.jet_pressed = self.ticks == 120;
        command.jet_held = (120..150).contains(&self.ticks);
        if self.ticks > 210 {
            command.select_weapon = Some(WeaponId::M416);
            command.aim_at = Some(body_center(remote.movement.body));
            command.fire_held = local.combat.alive()
                && remote.combat.alive()
                && local.kills == 0
                && (local.id == ActorId::One || local.deaths > 0);
            command.reload_pressed = local.combat.weapon().ammo == 0;
        }
        command
    }
}
