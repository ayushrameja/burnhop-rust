//! Opt-in injected traversal review. Each stage has an explicit initial fixture;
//! between that fixture and landing only normal 60 Hz core commands are used.
use burnhop_gameplay_core::{ember::EMBER_RELAY as MAP, offline::*, *};
#[derive(Clone, Copy, Debug)]
pub struct Stage {
    pub name: &'static str,
    pub start: (f64, f64),
    pub target: (f64, f64),
    pub landing: (f64, f64),
    pub jet: u16,
    pub jump: bool,
    pub vx: f64,
    pub crouch: bool,
}
const fn stage(
    name: &'static str,
    start: (f64, f64),
    target: (f64, f64),
    landing: (f64, f64),
    jet: u16,
) -> Stage {
    Stage {
        name,
        start,
        target,
        landing,
        jet,
        jump: false,
        vx: 0.,
        crouch: false,
    }
}
pub const STAGES: &[Stage] = &[
    Stage {
        jump: true,
        ..stage("cover-hop", (285., 1180.), (447., 1188.1), (420., 620.), 0)
    },
    Stage {
        jump: true,
        vx: 320.,
        ..stage(
            "yard-shaft-jump",
            (757., 1240.),
            (950., 1188.6667),
            (920., 1100.),
            0,
        )
    },
    stage("F01", (220., 1180.), (365., 960.), (300., 460.), 42),
    stage("F02", (400., 960.), (693., 820.), (650., 770.), 42),
    stage("F03", (720., 820.), (1120., 900.), (1070., 1210.), 42),
    stage(
        "crown-F08",
        (1150., 900.),
        (1497., 650.),
        (1470., 1570.),
        48,
    ),
    stage(
        "crown-to-F06",
        (1510., 650.),
        (1950., 760.),
        (1910., 2030.),
        42,
    ),
    stage(
        "risky-F03-F05",
        (1160., 900.),
        (1675., 1010.),
        (1640., 1760.),
        42,
    ),
    Stage {
        jump: true,
        vx: 320.,
        ..stage(
            "lower-west-east",
            (1404., 1560.),
            (1630., 1580.),
            (1580., 1780.),
            0,
        )
    },
    Stage {
        jump: true,
        vx: 320.,
        ..stage(
            "lower-island-east",
            (1744., 1580.),
            (1960., 1560.),
            (1900., 2200.),
            0,
        )
    },
    Stage {
        jump: true,
        vx: -320.,
        ..stage(
            "lower-east-island",
            (1900., 1560.),
            (1700., 1580.),
            (1580., 1780.),
            0,
        )
    },
    Stage {
        jump: true,
        vx: -320.,
        ..stage(
            "lower-island-west",
            (1580., 1580.),
            (1380., 1573.7142857),
            (1300., 1440.),
            0,
        )
    },
    stage(
        "M2-U05-F04",
        (1730., 1580.),
        (1635., 1270.),
        (1600., 1720.),
        54,
    ),
    stage(
        "M2-F04-F05",
        (1600., 1270.),
        (1680., 1010.),
        (1640., 1760.),
        48,
    ),
    stage(
        "M2-F05-F06",
        (1700., 1010.),
        (1950., 760.),
        (1910., 2030.),
        48,
    ),
    stage(
        "M1-floor-U12",
        (935., 1640.),
        (842., 1440.),
        (810., 910.),
        40,
    ),
    stage(
        "M1-U12-surface",
        (840., 1440.),
        (742., 1240.),
        (620., 800.),
        42,
    ),
    stage(
        "M3-floor-U11",
        (2720., 1640.),
        (2620., 1390.),
        (2570., 2690.),
        48,
    ),
    stage(
        "M3-U11-surface",
        (2650., 1390.),
        (2730., 1132.),
        (2700., 2900.),
        48,
    ),
    stage(
        "M4-outer",
        (3100., 1540.),
        (3030., 1200.),
        (2900., 3100.),
        54,
    ),
    Stage {
        jump: true,
        ..stage("F06-F07", (1980., 760.), (2240., 930.), (2190., 2330.), 0)
    },
    stage(
        "F09-lookout",
        (2990., 1200.),
        (2800., 880.),
        (2760., 2880.),
        54,
    ),
    Stage {
        crouch: true,
        ..stage(
            "west-crouch",
            (1010., 1640.),
            (1260., 1640.),
            (900., 1300.),
            0,
        )
    },
    Stage {
        crouch: true,
        ..stage(
            "west-crouch-reverse",
            (1252., 1640.),
            (1010., 1640.),
            (900., 1300.),
            0,
        )
    },
    Stage {
        crouch: true,
        ..stage(
            "east-crouch",
            (2410., 1640.),
            (2700., 1640.),
            (2400., 2880.),
            0,
        )
    },
    Stage {
        crouch: true,
        ..stage(
            "east-crouch-reverse",
            (2690., 1640.),
            (2405., 1640.),
            (2400., 2880.),
            0,
        )
    },
];
pub fn initial(stage: Stage, fuel: f64) -> OfflinePracticeState {
    let mut s = OfflinePracticeState::new(MapId::EmberRelay).unwrap();
    s.world.player.body = Rect::new(stage.start.0, stage.start.1 - 68., 36., 68.);
    s.world.player.grounded = burnhop_gameplay_core::mixed::supported(s.world.player.body, &MAP);
    s.world.player.coyote_ticks = if s.world.player.grounded { 8 } else { 0 };
    s.world.player.velocity.x = stage.vx;
    s.world.player.fuel = fuel;
    // Traversal fixture: no bot damage, without changing movement/fuel tuning.
    s.combat.bot.life = LifeState::Dead {
        remaining_ticks: u16::MAX,
    };
    s.combat.bot.health = 0;
    s
}
pub fn command(stage: Stage, s: &OfflinePracticeState, t: u16) -> OfflineCommand {
    let p = s.world.player;
    let mut goal = stage.target.0;
    if stage.jet > 0 && p.body.y + p.body.height > stage.target.1 - 2. && p.velocity.y < 0. {
        if stage.start.0 + 36. <= stage.landing.0 {
            goal = goal.min(stage.landing.0 - 40.);
        } else if stage.start.0 >= stage.landing.1 {
            goal = goal.max(stage.landing.1 + 4.);
        }
    }
    let dx = goal - p.body.x;
    let v = p.velocity.x;
    let stop = v * v / (2. * 2300.) + v.abs() / 60.;
    let axis = if dx.abs() < 1. && v.abs() < 15. {
        MoveAxis::Idle
    } else if v.abs() > 20. && dx.signum() == v.signum() && dx.abs() < stop {
        if v > 0. {
            MoveAxis::Left
        } else {
            MoveAxis::Right
        }
    } else if dx > 0. {
        MoveAxis::Right
    } else {
        MoveAxis::Left
    };
    OfflineCommand {
        input: InputCommand {
            tick: s.world.tick,
            move_x: if stage.crouch && t < 11 {
                MoveAxis::Idle
            } else {
                axis
            },
            jump_pressed: stage.jump && t == 0,
            jet_pressed: stage.jet > 0 && t == 0,
            jet_held: t < stage.jet,
            ..Default::default()
        },
        crouch_held: stage.crouch,
        jump_held: false,
    }
}
pub fn landed(stage: Stage, s: &OfflinePracticeState, t: u16) -> bool {
    t > 20
        && s.world.player.grounded
        && (s.world.player.body.x - stage.target.0).abs() < 6.
        && (s.world.player.body.y + s.world.player.body.height - stage.target.1).abs() < 12.
        && s.world.player.velocity.x.abs() < 35.
}
