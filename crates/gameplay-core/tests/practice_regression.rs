//! Compare the multiplayer practice adapter with the frozen, approved combat
//! implementation on the SAME platform. Neither implementation calls the other.
#[path = "fixtures/approved_practice/lib.rs"]
mod approved;
// The frozen source originally used crate-root imports; retain those unchanged.
pub use approved::*;
use burnhop_gameplay_core as current;

fn current_input(input: InputCommand) -> current::InputCommand {
    current::InputCommand {
        tick: input.tick,
        move_x: match input.move_x {
            MoveAxis::Left => current::MoveAxis::Left,
            MoveAxis::Idle => current::MoveAxis::Idle,
            MoveAxis::Right => current::MoveAxis::Right,
        },
        jump_pressed: input.jump_pressed,
        jet_pressed: input.jet_pressed,
        jet_held: input.jet_held,
        reset: input.reset,
        release_input: input.release_input,
        aim_at: input.aim_at.map(|p| current::Vec2 { x: p.x, y: p.y }),
        fire_held: input.fire_held,
        reload_pressed: input.reload_pressed,
        select_weapon: input.select_weapon.map(|weapon| match weapon {
            WeaponId::Pistol => current::WeaponId::Pistol,
            WeaponId::M416 => current::WeaponId::M416,
        }),
    }
}

fn practice_trace() -> u64 {
    let mut actual_world = current::World::new(&current::PRACTICE_ARENA);
    let mut actual_combat = current::CombatState::default();
    let mut world = World::new(&PRACTICE_ARENA);
    let mut combat = CombatState::default();
    let mut hash = 0xcbf29ce484222325_u64;
    for tick in 0..12000 {
        let input = InputCommand {
            tick,
            move_x: match tick % 900 {
                100..300 => MoveAxis::Right,
                400..550 => MoveAxis::Left,
                _ => MoveAxis::Idle,
            },
            jump_pressed: tick % 143 == 0,
            jet_pressed: tick % 431 == 17,
            jet_held: tick % 431 >= 17 && tick % 431 < 61,
            reset: tick % 1777 == 1776,
            release_input: tick % 617 == 0,
            aim_at: Some(body_center(combat.bot_body)),
            fire_held: tick % 211 < 180,
            reload_pressed: tick % 71 == 0,
            select_weapon: match tick % 251 {
                0 => Some(WeaponId::M416),
                123 => Some(WeaponId::Pistol),
                _ => None,
            },
        };
        let events = step_practice(&mut world, &mut combat, input, &PRACTICE_ARENA).unwrap();
        let actual_events = current::step_practice(
            &mut actual_world,
            &mut actual_combat,
            current_input(input),
            &current::PRACTICE_ARENA,
        )
        .unwrap();
        // Compare every Debug field on EVERY tick, with no float rounding or tolerance.
        // Only the deliberate Player/Bot -> One/Two label rename is normalized.
        let actual_text = format!("{actual_world:?}{actual_combat:?}{actual_events:?}")
            .replace("Player", "One")
            .replace("Bot", "Two");
        let text = format!("{world:?}{combat:?}{events:?}")
            .replace("Player", "One")
            .replace("Bot", "Two");
        assert_eq!(
            actual_text, text,
            "practice differs from approved source at tick {tick}"
        );
        for b in text.bytes() {
            hash = (hash ^ u64::from(b)).wrapping_mul(0x100000001b3);
        }
    }
    hash
}

#[test]
fn approved_practice_matches_pre_refactor_12000_tick_trace() {
    let hash = practice_trace();
    println!("Approved source and current practice matched all 12000 ticks; trace hash: {hash}");
    // Keep the historical golden on the target where it was captured. The
    // per-tick differential assertions above run unconditionally on all targets.
    // std f64::hypot does not promise identical precision across platforms.
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        assert_eq!(hash, 11_325_689_209_779_929_004);
    }
}
