//! Native review companion: one real window joins first, then seven UDP clients.
//! A local phase file selects gather / jets / combat / depart / reuse / stop.
//! All positioning and firing uses ordinary authoritative input commands.
use burnhop_gameplay_core::*;
use burnhop_server::{
    Server, ServerClock,
    reliability::{Synthetic, synthetic_command},
};
use std::time::{Duration, Instant};
fn main() -> Result<(), String> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let address = args
        .first()
        .ok_or("rendered_scene IP:port /absolute/phase-file")?
        .parse()
        .map_err(|_| "address")?;
    let path = args.get(1).ok_or("phase file")?;
    let mut server = Server::bind(address)?;
    let mut bots: Vec<Synthetic> = Vec::new();
    let mut clock = ServerClock::default();
    let mut last = Instant::now();
    let mut report = last;
    let mut phase_at = last;
    let mut auto_started = None;
    let automatic = args.get(2).is_some_and(|a| a == "--auto");
    let mut phase = "gather".to_owned();
    let mut departed = false;
    let mut replaced = false;
    println!(
        "RENDER_SCENE address={} waiting for one native client",
        server.address()
    );
    loop {
        let now = Instant::now();
        let dt = now.duration_since(last);
        last = now;
        let mut next = std::fs::read_to_string(path)
            .unwrap_or_else(|_| "gather".into())
            .trim()
            .to_owned();
        if automatic && next != "stop" {
            if server.player_count() == 8 && auto_started.is_none() {
                auto_started = Some(now);
            }
            if let Some(start) = auto_started {
                let age = now.duration_since(start).as_secs_f64();
                next = if age < 12. {
                    "gather"
                } else if age < 16. {
                    "jets"
                } else if age < 22. {
                    "gather"
                } else if age < 25. {
                    "depart"
                } else if age < 35. {
                    "reuse"
                } else {
                    "combat"
                }
                .into();
            }
        }
        if next == "stop" {
            break;
        }
        if next != phase {
            phase = next;
            phase_at = now;
            println!("RENDER_SCENE phase={phase}");
        }
        if bots.is_empty() && server.player_count() == 1 {
            for i in 0..7 {
                bots.push(Synthetic::new(server.address(), 9000 + i)?);
            }
        }
        if phase == "depart" && !departed && bots.len() == 7 {
            bots[6].network.close("rendered departure");
            departed = true;
        }
        if phase == "reuse" && departed && !replaced {
            bots[6] = Synthetic::new(server.address(), 10000)?;
            replaced = true;
        }
        for (index, bot) in bots.iter_mut().enumerate() {
            if departed && !replaced && index == 6 {
                continue;
            }
            let snapshots = bot.network.update(dt);
            if bot.prediction.is_none()
                && let Some(w) = bot.network.welcome
            {
                bot.prediction = Some(burnhop_protocol::prediction::Prediction::new(w));
            }
            if let Some(p) = &mut bot.prediction {
                p.observe_timing(bot.network.measured_rtt(), dt.as_secs_f64());
                for snapshot in snapshots {
                    p.reconcile(snapshot);
                }
                for _ in 0..bot.clock.advance(dt.as_secs_f64()) {
                    bot.ticks += 1;
                    let mut input = InputCommand::default();
                    if phase == "combat" {
                        input = synthetic_command(p, bot.ticks, true);
                    } else if let Some(actor) = p.local
                        && actor.combat.alive()
                        && !actor.require_neutral
                    {
                        let x = 450. + actor.id.index() as f64 * 80.;
                        let dx = x - actor.movement.body.x;
                        input.move_x = if (actor.movement.grounded && actor.movement.body.y < 1100.)
                            || dx > 4.
                        {
                            MoveAxis::Right
                        } else if dx < -4. {
                            MoveAxis::Left
                        } else {
                            MoveAxis::Idle
                        };
                        input.aim_at = Some(Vec2 { x: 390., y: 1186. });
                        if phase == "jets" {
                            let t = (now.duration_since(phase_at).as_secs_f64() * 60.) as u64 % 240;
                            input.jet_pressed = t < 2;
                            input.jet_held = t < 50;
                        }
                    }
                    if let Some(bundle) = p.command(input) {
                        bot.network.send_inputs(bundle);
                    }
                }
            }
            bot.network.flush();
        }
        server.poll(dt)?;
        for _ in 0..clock.advance(dt.as_secs_f64()) {
            server.tick();
            if server.state.tick.is_multiple_of(2) {
                server.snapshot();
            }
        }
        server.flush();
        if now.duration_since(report).as_secs_f64() > 5. {
            println!(
                "RENDER_SCENE tick={} actors={:?}",
                server.state.tick,
                server.state.actors.map(|a| a.map(|a| (
                    a.id,
                    a.generation,
                    a.movement.body.x,
                    a.movement.body.y,
                    a.combat.health,
                    a.kills,
                    a.deaths
                )))
            );
            report = now;
        }
        std::thread::sleep(Duration::from_millis(1));
    }
    Ok(())
}
