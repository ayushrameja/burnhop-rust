//! Debug-only injected visual fixtures. These are rendering evidence, not gameplay
//! or physical-input tests. Enable explicitly with BURNHOP_CHARACTER_REVIEW_DIR.
use crate::{
    Playground,
    appearance::Appearance,
    menu::{self, Action, Screen},
};
use bevy::{
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::PrimaryWindow,
};
#[derive(Default)]
pub struct Review {
    frame: usize,
    directory: Option<std::path::PathBuf>,
}
pub fn fixtures(
    mut commands: Commands,
    mut game: ResMut<Playground>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
    mut state: Local<Review>,
    mut exit: MessageWriter<AppExit>,
) {
    if state.frame == 0 {
        state.directory = std::env::var_os("BURNHOP_CHARACTER_REVIEW_DIR").map(Into::into);
    }
    state.frame += 1;
    let Some(dir) = state.directory.clone() else {
        return;
    };
    let stage = (state.frame - 1) / 60;
    const NAMES: [&str; 10] = [
        "01-field-right-aim",
        "02-field-left-crouch",
        "03-scout-right-jet",
        "04-scout-left-reload",
        "05-scout-right-death",
        "06-base-left-aim",
        "07-compact-field-preview",
        "08-compact-save-error",
        "09-desktop-beret-preview",
        "10-desktop-base-preview",
    ];
    if stage >= NAMES.len() {
        println!("CHARACTER FIXTURES complete: 10 injected native frames; no human approval");
        exit.write(AppExit::Success);
        return;
    }
    if (state.frame - 1).is_multiple_of(60) {
        menu::act(
            &mut game,
            if stage.is_multiple_of(2) {
                Action::Ember
            } else {
                Action::Practice
            },
        );
        game.appearance.message.clear();
        game.appearance.saved = Appearance::preset(if stage < 2 {
            2
        } else if stage < 5 {
            3
        } else {
            1
        });
        if (2..5).contains(&stage) {
            // deep skin, uncovered blond swept hair, rust shirt
            for _ in 0..3 {
                game.appearance.saved.cycle(1, false);
            }
            for _ in 0..2 {
                game.appearance.saved.cycle(4, false);
            }
            game.appearance.saved.cycle(3, false);
            for _ in 0..2 {
                game.appearance.saved.cycle(6, false);
            }
        }
        game.appearance.draft = game.appearance.saved;
        window.resolution.set(
            if stage == 6 || stage == 7 {
                480.
            } else {
                1280.
            },
            if stage == 6 || stage == 7 { 320. } else { 720. },
        );
        if stage >= 6 {
            menu::act(&mut game, Action::Character);
            if stage == 6 {
                game.appearance.draft = Appearance::preset(2);
                game.appearance.left = true;
                game.appearance.pose = 1;
            }
            if stage == 7 {
                game.appearance.path = Err("Permission denied".into());
                game.appearance.draft = Appearance::preset(3);
                menu::act(&mut game, Action::CharacterApply);
            }
            if stage == 8 {
                game.appearance.draft.cycle(4, true);
                game.appearance.left = false;
                game.appearance.pose = 0;
            }
            if stage == 9 {
                game.appearance.draft = Appearance::preset(1);
                game.appearance.left = false;
                game.appearance.pose = 0;
            }
            menu::character_labels(&mut game);
        }
        println!(
            "CHARACTER FIXTURE {} saved={:?} draft={:?}",
            NAMES[stage], game.appearance.saved, game.appearance.draft
        );
    }
    if stage < 6 {
        // Hold only this rendered fixture; ordinary simulation/input tests are separate.
        game.menu.screen = Screen::Playing;
        game.focused = true;
        game.world.player.body.x = if game.ember() { 200. } else { 390. };
        game.world.player.body.y = if game.ember() { 1112. } else { 1152. };
        game.world.player.body.height = 68.;
        game.world.player.grounded = true;
        game.world.player.thrusting = false;
        game.world.player.velocity = Default::default();
        game.combat.player = burnhop_gameplay_core::Combatant::new(false);
        game.combat.player.selected = burnhop_gameplay_core::WeaponId::M416;
        game.combat.player.aim = burnhop_gameplay_core::Vec2 {
            x: if stage.is_multiple_of(2) { 0.8 } else { -0.8 },
            y: -0.6,
        };
        if stage == 1 {
            game.world.player.body.y += 68. - 54.20060507330696;
            game.world.player.body.height = 54.20060507330696;
        }
        if stage == 2 {
            game.world.player.body.y -= 70.;
            game.world.player.grounded = false;
            game.world.player.thrusting = true;
        }
        if stage == 3 {
            game.combat.player.weapons[1].reload_ticks = 60;
        }
        if stage == 4 {
            game.combat.player.health = 0;
            game.combat.player.life = burnhop_gameplay_core::LifeState::Dead {
                remaining_ticks: 180,
            };
        }
        game.previous = game.world.player;
        game.alpha = 1.;
        game.snap_camera = true;
    }
    if state.frame % 60 == 35 {
        std::fs::create_dir_all(&dir).expect("fixture output directory");
        let path = dir.join(format!("{}.ppm", NAMES[stage]));
        commands.spawn(Screenshot::primary_window()).observe(
            move |event: On<ScreenshotCaptured>| {
                use std::io::Write;
                let image = event
                    .image
                    .clone()
                    .try_into_dynamic()
                    .expect("fixture RGB")
                    .to_rgb8();
                let mut file = std::fs::File::create(&path).expect("fixture file");
                write!(file, "P6\n{} {}\n255\n", image.width(), image.height()).unwrap();
                file.write_all(image.as_raw()).unwrap();
            },
        );
    }
}
