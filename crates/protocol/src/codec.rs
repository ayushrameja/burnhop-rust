//! Explicit little-endian, fixed-layout codec: no lengths supplied by peers and
//! no allocation during decoding. Transport reliability belongs to Renet.
use super::*;

type Result<T> = std::result::Result<T, &'static str>;
trait Wire: Sized {
    fn write(&self, out: &mut Vec<u8>);
    fn read(input: &mut &[u8]) -> Result<Self>;
}
macro_rules! number {
    ($($t:ty),*) => { $(impl Wire for $t {
        fn write(&self, out: &mut Vec<u8>) { out.extend(self.to_le_bytes()); }
        fn read(input: &mut &[u8]) -> Result<Self> {
            let n = std::mem::size_of::<Self>();
            let bytes = input.get(..n).ok_or("truncated number")?;
            let value = Self::from_le_bytes(bytes.try_into().map_err(|_| "number")?);
            *input = &input[n..];
            Ok(value)
        }
    })* };
}
number!(u8, u16, u32, u64);
impl Wire for f64 {
    fn write(&self, out: &mut Vec<u8>) {
        self.to_bits().write(out);
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        let value = Self::from_bits(u64::read(input)?);
        if !value.is_finite() || value.abs() > 1_000_000.0 {
            return Err("invalid float");
        }
        Ok(value)
    }
}
impl Wire for bool {
    fn write(&self, out: &mut Vec<u8>) {
        u8::from(*self).write(out);
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        match u8::read(input)? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid bool"),
        }
    }
}
impl<T: Wire> Wire for Option<T> {
    fn write(&self, out: &mut Vec<u8>) {
        self.is_some().write(out);
        if let Some(value) = self {
            value.write(out);
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        if bool::read(input)? {
            Ok(Some(T::read(input)?))
        } else {
            Ok(None)
        }
    }
}
impl<T: Wire, const N: usize> Wire for [T; N] {
    fn write(&self, out: &mut Vec<u8>) {
        for value in self {
            value.write(out);
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        // Constant-sized Option array avoids unsafe initialization and heap allocation.
        let mut values = [const { None }; N];
        for value in &mut values {
            *value = Some(T::read(input)?);
        }
        Ok(values.map(|v| v.expect("all array elements initialized")))
    }
}
macro_rules! record {
    ($t:ty { $($field:ident),* $(,)? }) => { impl Wire for $t {
        fn write(&self, out: &mut Vec<u8>) { $(self.$field.write(out);)* }
        fn read(input: &mut &[u8]) -> Result<Self> { Ok(Self { $($field: Wire::read(input)?),* }) }
    } };
}
macro_rules! tags {
    ($t:ty { $($tag:literal => $value:path),* $(,)? }) => { impl Wire for $t {
        fn write(&self, out: &mut Vec<u8>) { match self { $($value => ($tag as u8).write(out)),* } }
        fn read(input: &mut &[u8]) -> Result<Self> { match u8::read(input)? { $($tag => Ok($value)),*, _ => Err("invalid enum") } }
    } };
}
tags!(ActorId { 0 => ActorId::One, 1 => ActorId::Two });
tags!(WeaponId { 0 => WeaponId::Pistol, 1 => WeaponId::M416 });
tags!(MoveAxis { 0 => MoveAxis::Idle, 1 => MoveAxis::Left, 2 => MoveAxis::Right });
tags!(Rejection { 0 => Rejection::Compatibility, 1 => Rejection::Full, 2 => Rejection::InvalidInput,
    3 => Rejection::RateLimit, 4 => Rejection::Timeout });
impl Wire for LifeState {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Alive => 0_u16.write(out),
            Self::Dead { remaining_ticks } => remaining_ticks.write(out),
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        match u16::read(input)? {
            0 => Ok(Self::Alive),
            n @ 1..=RESPAWN_TICKS => Ok(Self::Dead { remaining_ticks: n }),
            _ => Err("life timer"),
        }
    }
}
impl Wire for Reserve {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Rounds(n) => n.write(out),
            Self::Unlimited => u16::MAX.write(out),
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        Ok(match u16::read(input)? {
            u16::MAX => Self::Unlimited,
            n => Self::Rounds(n),
        })
    }
}
impl Wire for Impact {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Range => 0_u8.write(out),
            Self::Terrain => 1_u8.write(out),
            Self::Body(id) => {
                2_u8.write(out);
                id.write(out);
            }
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        match u8::read(input)? {
            0 => Ok(Self::Range),
            1 => Ok(Self::Terrain),
            2 => Ok(Self::Body(ActorId::read(input)?)),
            _ => Err("impact"),
        }
    }
}
record!(Vec2 { x, y });
record!(Rect {
    x,
    y,
    width,
    height
});
record!(InputCommand {
    tick,
    move_x,
    jump_pressed,
    jet_pressed,
    jet_held,
    reset,
    release_input,
    aim_at,
    fire_held,
    reload_pressed,
    select_weapon
});
record!(Player {
    body,
    velocity,
    grounded,
    coyote_ticks,
    jump_buffer_ticks,
    fuel,
    fuel_delay_ticks,
    thrust_latched,
    thrusting
});
record!(WeaponState {
    id,
    ammo,
    reserve,
    cooldown_ticks,
    reload_ticks
});
record!(Combatant {
    health,
    life,
    selected,
    weapons,
    equip_ticks,
    aim,
    reload_was_pressed
});
record!(Actor {
    id,
    generation,
    movement,
    combat,
    require_neutral,
    kills,
    deaths
});
record!(MatchState { tick, actors });
record!(Shot {
    shooter,
    weapon,
    origin,
    end,
    impact,
    damage
});
record!(NetInput {
    actor,
    sequence,
    command
});
record!(Welcome {
    actor,
    generation,
    start_tick
});
record!(ConfirmedShot { tick, shot });
record!(Snapshot {
    state,
    ack,
    last_applied,
    shots
});
impl Wire for Message {
    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Self::Hello { protocol, gameplay } => {
                0_u8.write(out);
                protocol.write(out);
                gameplay.write(out);
            }
            Self::Welcome(v) => {
                1_u8.write(out);
                v.write(out);
            }
            Self::Reject(v) => {
                2_u8.write(out);
                v.write(out);
            }
            Self::Inputs(v) => {
                3_u8.write(out);
                v.write(out);
            }
            Self::Release { through } => {
                4_u8.write(out);
                through.write(out);
            }
            Self::Snapshot(v) => {
                5_u8.write(out);
                v.write(out);
            }
        }
    }
    fn read(input: &mut &[u8]) -> Result<Self> {
        Ok(match u8::read(input)? {
            0 => Self::Hello {
                protocol: Wire::read(input)?,
                gameplay: Wire::read(input)?,
            },
            1 => Self::Welcome(Wire::read(input)?),
            2 => Self::Reject(Wire::read(input)?),
            3 => Self::Inputs(Wire::read(input)?),
            4 => Self::Release {
                through: Wire::read(input)?,
            },
            5 => Self::Snapshot(Wire::read(input)?),
            _ => return Err("message tag"),
        })
    }
}
pub fn encode(message: &Message) -> Vec<u8> {
    let mut output = Vec::with_capacity(MAX_MESSAGE_BYTES);
    message.write(&mut output);
    assert!(
        output.len() <= MAX_MESSAGE_BYTES,
        "fixed schema exceeds envelope"
    );
    output
}
pub fn decode(mut input: &[u8]) -> Result<Message> {
    if input.len() > MAX_MESSAGE_BYTES {
        return Err("message too large");
    }
    let message = Message::read(&mut input)?;
    if !input.is_empty() {
        return Err("trailing bytes");
    }
    if let Message::Snapshot(snapshot) = message {
        if snapshot.state.tick > u64::MAX - 10_000 || snapshot.last_applied > snapshot.ack {
            return Err("snapshot tick/ack");
        }
        for (i, actor) in snapshot.state.actors.iter().enumerate() {
            if let Some(a) = actor {
                let p = a.movement;
                let c = a.combat;
                if a.id.index() != i
                    || c.health > MAX_HEALTH
                    || (c.health > 0) != c.alive()
                    || p.body.width != BODY_WIDTH
                    || p.body.height != BODY_HEIGHT
                    || !(0.0..=MAX_FUEL).contains(&p.fuel)
                    || c.weapons
                        .iter()
                        .enumerate()
                        .any(|(i, w)| w.id.index() != i || w.ammo > w.id.tuning().magazine)
                {
                    return Err("invalid actor state");
                }
            }
        }
    }
    Ok(message)
}
