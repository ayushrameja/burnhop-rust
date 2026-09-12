//! Opt-in native framebuffer evidence, no simulation mutation. PPM keeps the
//! shipping dependency/feature set unchanged; convert selected captures offline.
use crate::Playground;
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::PrimaryWindow,
};
#[derive(Default)]
pub struct Capture {
    directory: Option<std::path::PathBuf>,
    initialized: bool,
    seen: std::collections::BTreeSet<&'static str>,
    manual: u8,
    last_size: Vec2,
    frames: u64,
}
pub fn capture(
    mut commands: Commands,
    game: Res<Playground>,
    window: Single<&Window, With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut state: Local<Capture>,
) {
    if !state.initialized {
        state.initialized = true;
        state.directory = std::env::var_os("BURNHOP_CAPTURE_DIR").map(Into::into);
        if let Some(path) = &state.directory
            && let Err(e) = std::fs::create_dir_all(path)
        {
            warn!("Capture directory: {e}");
            state.directory = None;
        }
    }
    state.frames += 1;
    let Some(dir) = state.directory.clone() else {
        return;
    };
    if game.world.tick < 10 || state.frames < 60 {
        return;
    }
    let p = game.world.player;
    let c = game.combat.player;
    let mode = if game.online.is_some() {
        "online"
    } else {
        "practice"
    };
    let candidates = [
        (
            "scoreboard",
            game.online.is_some()
                && (keys.pressed(KeyCode::Tab) || std::env::var_os("BURNHOP_REVIEW_TAB").is_some()),
        ),
        (
            "remote-jets",
            game.online.as_ref().is_some_and(|o| {
                o.actors
                    .iter()
                    .flatten()
                    .filter(|a| a.movement.thrusting)
                    .count()
                    >= 5
            }),
        ),
        (
            "eight-players",
            game.online
                .as_ref()
                .is_some_and(|o| o.actors.iter().flatten().count() == 8),
        ),
        (
            "departure",
            state.seen.contains("eight-players")
                && game
                    .online
                    .as_ref()
                    .is_some_and(|o| o.actors.iter().flatten().count() == 7),
        ),
        (
            "replacement",
            game.online
                .as_ref()
                .is_some_and(|o| o.actors.iter().flatten().any(|a| a.generation > 8)),
        ),
        (
            "replacement-visible",
            game.online.as_ref().is_some_and(|o| {
                o.actors.iter().flatten().any(|a| {
                    a.generation > 8
                        && (a.movement.body.x - p.body.x).abs() < 500.
                        && (a.movement.body.y - p.body.y).abs() < 100.
                })
            }),
        ),
        ("spawn", c.alive() && game.world.tick < 120),
        (
            "moving",
            c.alive() && p.grounded && p.velocity.x.abs() > 200.,
        ),
        (
            "jet",
            c.alive() && p.thrusting && p.body.y < 1120. && p.fuel < 90.,
        ),
        ("airborne", c.alive() && !p.grounded && !p.thrusting),
        (
            "rifle",
            c.alive() && c.selected == burnhop_gameplay_core::WeaponId::M416 && c.equip_ticks == 0,
        ),
        (
            "reload",
            c.weapon().reload_ticks > 8 && c.weapon().reload_ticks < c.selected.tuning().reload - 8,
        ),
        ("death", !c.alive()),
        ("respawn", c.alive() && game.combat.deaths > 0),
        ("small", window.width() < 600.),
        ("impact", game.feedback.has_terrain_impact()),
        (
            "pistol-fire",
            game.feedback
                .has_shot(game.local_id(), burnhop_gameplay_core::WeaponId::Pistol),
        ),
        (
            "rifle-fire",
            game.feedback
                .has_shot(game.local_id(), burnhop_gameplay_core::WeaponId::M416),
        ),
        (
            "disconnected",
            game.online
                .as_ref()
                .is_some_and(|o| o.network.status.terminal()),
        ),
    ];
    let automatic = candidates
        .into_iter()
        .find(|(name, ready)| *ready && !state.seen.contains(name));
    let name = if let Some((name, _)) = automatic {
        state.seen.insert(name);
        Some(name.to_owned())
    } else if keys.just_pressed(KeyCode::F9) && state.manual < 20 {
        state.manual += 1;
        Some(format!("manual-{:02}", state.manual))
    } else {
        None
    };
    if state.last_size != window.size() {
        state.last_size = window.size();
        println!(
            "VISUAL viewport={}x{} scale={}",
            window.width(),
            window.height(),
            window.scale_factor()
        );
    }
    if let Some(name) = name {
        let path = dir.join(format!("{mode}-{name}.ppm"));
        println!(
            "VISUAL capture={} tick={} actor={:?} local={:?} remote={:?}",
            name,
            game.world.tick,
            game.online
                .as_ref()
                .and_then(|o| o.network.welcome.map(|w| w.actor)),
            p,
            game.combat.bot.life
        );
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>| {
                use std::io::Write;
                match event.image.clone().try_into_dynamic() {
                    Ok(image) => {
                        let rgb = image.to_rgb8();
                        let result = (|| -> std::io::Result<()> {
                            let mut out = std::io::BufWriter::new(std::fs::File::create(&path)?);
                            write!(out, "P6\n{} {}\n255\n", rgb.width(), rgb.height())?;
                            out.write_all(rgb.as_raw())
                        })();
                        if let Err(e) = result {
                            warn!("Screenshot: {e}");
                        }
                    }
                    Err(e) => warn!("Screenshot conversion: {e}"),
                }
            },
        );
    }
}
