//! Native session ownership, menus and bounded numeric-address editing.
use crate::{Playground, online::Online};
use bevy::{
    input::{ButtonState, keyboard::KeyboardInput},
    prelude::*,
    window::PrimaryWindow,
};
use burnhop_protocol::transport::ConnectionState;
use burnhop_server::owned::OwnedServer;
use std::net::SocketAddr;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Screen {
    Main,
    Host,
    Join,
    Connecting,
    #[default]
    Playing,
    Paused,
    ConfirmStop,
    Error,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Practice,
    Ember,
    Host,
    Join,
    Quit,
    Edit,
    StartHost,
    Connect,
    Back,
    Cancel,
    Resume,
    Leave,
    ConfirmStop,
}

pub struct Session {
    pub screen: Screen,
    pub selected: usize,
    pub editor: Editor,
    pub notice: String,
    pub address: Option<SocketAddr>,
    pub owned: Option<OwnedServer>,
    pub retry_host: bool,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            screen: Screen::Playing,
            selected: 0,
            editor: Editor::new("127.0.0.1:5000"),
            notice: String::new(),
            address: None,
            owned: None,
            retry_host: false,
        }
    }
}
#[derive(Default)]
pub struct Editor {
    pub text: String,
    cursor: usize,
    selected_all: bool,
}
impl Editor {
    pub fn new(text: &str) -> Self {
        Self {
            text: text.into(),
            cursor: text.len(),
            selected_all: false,
        }
    }
    pub fn key(&mut self, event: &KeyboardInput, shortcut: bool) {
        if shortcut && event.key_code == KeyCode::KeyA {
            self.selected_all = true;
            return;
        }
        match event.key_code {
            KeyCode::Home => {
                self.cursor = 0;
                self.selected_all = false;
            }
            KeyCode::End => {
                self.cursor = self.text.len();
                self.selected_all = false;
            }
            KeyCode::ArrowLeft => {
                self.cursor = self.cursor.saturating_sub(1);
                self.selected_all = false;
            }
            KeyCode::ArrowRight => {
                self.cursor = (self.cursor + 1).min(self.text.len());
                self.selected_all = false;
            }
            KeyCode::Backspace | KeyCode::Delete => {
                if self.selected_all {
                    self.text.clear();
                    self.cursor = 0;
                } else if event.key_code == KeyCode::Backspace && self.cursor > 0 {
                    self.cursor -= 1;
                    self.text.remove(self.cursor);
                } else if event.key_code == KeyCode::Delete && self.cursor < self.text.len() {
                    self.text.remove(self.cursor);
                }
                self.selected_all = false;
            }
            _ if !shortcut => {
                if let Some(text) = &event.text {
                    let text: String = text
                        .chars()
                        .filter(|c| c.is_ascii_hexdigit() || ".:[]%".contains(*c))
                        .collect();
                    if !text.is_empty() {
                        if self.selected_all {
                            self.text.clear();
                            self.cursor = 0;
                            self.selected_all = false;
                        }
                        for c in text.chars() {
                            if self.text.len() == 64 {
                                break;
                            }
                            self.text.insert(self.cursor, c);
                            self.cursor += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn display(&self, focused: bool) -> String {
        if !focused {
            return self.text.clone();
        }
        if self.selected_all {
            return format!("[{}]", self.text);
        }
        let start = self.cursor.saturating_sub(34);
        let end = (start + 40).min(self.text.len());
        format!(
            "{}{}|{}{}",
            if start > 0 { "..." } else { "" },
            &self.text[start..self.cursor],
            &self.text[self.cursor..end],
            if end < self.text.len() { "..." } else { "" }
        )
    }
}
pub fn address(text: &str) -> Result<SocketAddr, String> {
    let address: SocketAddr = text
        .trim()
        .parse()
        .map_err(|_| "Enter numeric IP:port, e.g. 127.0.0.1:5000 or [::1]:5000.".to_string())?;
    if address.port() == 0
        || address.ip().is_unspecified()
        || address.ip().is_multicast()
        || matches!(address.ip(), std::net::IpAddr::V4(ip) if ip.is_broadcast())
    {
        return Err("Use a specific local or LAN IP and a port from 1 to 65535; wildcard addresses cannot be joined.".into());
    }
    Ok(address)
}
pub fn release(game: &mut Playground) {
    game.input.clear();
    if game.ember() {
        game.release_gate = true;
    }
    game.input_blocked = true;
    game.previous = game.world.player;
    game.alpha = 1.;
    game.snap_camera = true;
    if game.online.is_none() {
        game.clock.reset();
    }
    if let Some(online) = &mut game.online {
        online.cancel_script();
    }
}
fn change(game: &mut Playground, screen: Screen) {
    release(game);
    game.menu.screen = screen;
    game.menu.selected = if screen == Screen::Connecting { 1 } else { 0 };
}
/// Reuse the bounded arena/entity pools, replace all per-session simulation/history.
fn reset(game: &mut Playground) {
    game.online = None;
    game.map = Default::default();
    game.stance = Default::default();
    game.release_gate = false;
    game.recovery_tick = None;
    game.ember_review = None;
    game.menu.owned = None;
    game.menu.address = None;
    game.world = burnhop_gameplay_core::World::new(&burnhop_gameplay_core::PRACTICE_ARENA);
    game.combat = Default::default();
    let epoch = game.feedback.epoch.wrapping_add(1);
    game.feedback = Default::default();
    game.feedback.epoch = epoch;
    game.route = None;
    game.combat_route = None;
    game.clock.reset();
    release(game);
}
pub fn act(game: &mut Playground, action: Action) -> bool {
    match action {
        Action::Quit => {
            reset(game);
            return true;
        }
        Action::Ember => {
            reset(game);
            match burnhop_gameplay_core::offline::OfflinePracticeState::new(
                burnhop_gameplay_core::offline::MapId::EmberRelay,
            ) {
                Ok(state) => {
                    game.map = state.map;
                    game.world = state.world;
                    game.combat = state.combat;
                    game.stance = state.stance;
                    change(game, Screen::Playing);
                }
                Err(error) => {
                    game.menu.notice =
                        format!("Ember Relay: {error}. The range remains available.");
                    change(game, Screen::Error);
                }
            }
        }
        Action::Practice => {
            reset(game);
            change(game, Screen::Playing);
        }
        Action::Host | Action::Join => {
            reset(game);
            game.menu.notice.clear();
            game.menu.retry_host = action == Action::Host;
            change(
                game,
                if action == Action::Host {
                    Screen::Host
                } else {
                    Screen::Join
                },
            );
            game.menu.editor.selected_all = true;
        }
        Action::Back | Action::Cancel => {
            reset(game);
            game.menu.notice.clear();
            change(game, Screen::Main);
        }
        Action::Resume => {
            change(game, Screen::Playing);
        }
        Action::Leave if game.menu.owned.is_some() => {
            change(game, Screen::ConfirmStop);
            game.menu.selected = 0;
        }
        Action::Leave | Action::ConfirmStop => {
            reset(game);
            change(game, Screen::Main);
        }
        Action::Edit => {
            game.menu.editor.selected_all = true;
        }
        Action::StartHost | Action::Connect => {
            game.menu.retry_host = action == Action::StartHost;
            let parsed = address(&game.menu.editor.text);
            reset(game);
            let result = parsed.and_then(|addr| {
                if action == Action::StartHost {
                    let host = OwnedServer::start(addr).map_err(|e| format!("Could not host on {addr}: {e}. Choose another port or a local interface IP."))?;
                    game.menu.owned = Some(host);
                }
                let online = Online::new(addr, false).map_err(|e| format!("Could not open a client socket: {e}. Check the address/interface and retry."))?;
                game.menu.address = Some(addr);
                game.online = Some(online);
                Ok(())
            });
            match result {
                Ok(()) => {
                    game.menu.notice.clear();
                    change(game, Screen::Connecting);
                }
                Err(error) => {
                    reset(game);
                    game.menu.notice = error;
                    change(game, Screen::Error);
                }
            }
        }
    }
    false
}
pub fn connection_message(status: &ConnectionState) -> String {
    match status {
        ConnectionState::CompatibilityError => "Builds do not match. Use the same Burnhop build as the host, then retry.".into(),
        ConnectionState::Disconnected(reason) if reason.to_lowercase().contains("full") => "This match is full (8 players). Ask the host for a free seat, or join another address.".into(),
        ConnectionState::Disconnected(reason) if reason.contains("five seconds") || reason.to_lowercase().contains("tim") => "Connection timed out. Check the host is running, the IP/port and the UDP firewall, then retry.".into(),
        ConnectionState::Disconnected(reason) => format!("Connection ended: {reason}. The host may have stopped. Check the address and retry."),
        _ => String::new(),
    }
}
pub fn poll(game: &mut Playground) {
    let error = game
        .menu
        .owned
        .as_ref()
        .and_then(OwnedServer::failure)
        .map(|e| format!("Hosting stopped: {e}. Resources released; you can host again."))
        .or_else(|| {
            game.online
                .as_ref()
                .filter(|o| o.network.status.terminal())
                .map(|o| connection_message(&o.network.status))
        });
    if let Some(error) = error {
        reset(game);
        game.menu.notice = error;
        change(game, Screen::Error);
    } else if game.menu.screen == Screen::Connecting
        && game
            .online
            .as_ref()
            .is_some_and(|o| o.network.status == ConnectionState::Connected)
    {
        change(game, Screen::Playing);
    }
}
pub fn controls(session: &Session) -> Vec<(String, Action, bool)> {
    let item = |label: &str, action| (label.to_string(), action, true);
    match session.screen {
        Screen::Main => vec![
            item("Practice", Action::Practice),
            item("Ember Relay Practice / Offline", Action::Ember),
            item("Host Game", Action::Host),
            item("Join Game", Action::Join),
            item("Quit", Action::Quit),
        ],
        Screen::Host | Screen::Join => vec![
            (
                session.editor.display(session.selected == 0),
                Action::Edit,
                true,
            ),
            (
                if session.screen == Screen::Host {
                    "Start Hosting"
                } else {
                    "Connect"
                }
                .into(),
                if session.screen == Screen::Host {
                    Action::StartHost
                } else {
                    Action::Connect
                },
                address(&session.editor.text).is_ok(),
            ),
            item("Back", Action::Back),
        ],
        Screen::Connecting => vec![
            ("Connecting...".into(), Action::Edit, false),
            item("Cancel", Action::Cancel),
        ],
        Screen::Paused => vec![
            item("Resume", Action::Resume),
            item(
                if session.owned.is_some() {
                    "Stop Hosting..."
                } else if session.address.is_some() {
                    "Leave Match"
                } else {
                    "Return to Main Menu"
                },
                Action::Leave,
            ),
        ],
        Screen::ConfirmStop => vec![
            item("Keep Playing", Action::Resume),
            item("Stop Hosting", Action::ConfirmStop),
        ],
        Screen::Error => vec![
            item(
                "Edit & Retry",
                if session.retry_host {
                    Action::Host
                } else {
                    Action::Join
                },
            ),
            item("Main Menu", Action::Back),
        ],
        Screen::Playing => vec![],
    }
}

#[derive(Component)]
pub struct MenuRoot;
#[derive(Component)]
pub struct Card;
#[derive(Component)]
pub struct Brand;
#[derive(Component)]
pub struct Copy(pub u8);
#[derive(Component)]
pub struct Control(pub usize);
#[derive(Component)]
pub struct ControlText(pub usize);
fn font(size: f32) -> TextFont {
    TextFont {
        font_size: size.into(),
        ..default()
    }
}
pub fn setup(mut commands: Commands) {
    use crate::artwork::{CREAM, color};
    commands
        .spawn((
            MenuRoot,
            GlobalZIndex(100),
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                ..default()
            },
            BackgroundColor(color(0x182727).with_alpha(0.86)),
        ))
        .with_children(|root| {
            root.spawn((
                Brand,
                Text::new("BURNHOP\nFIELD RANGE"),
                font(54.),
                TextColor(color(CREAM)),
                Node {
                    position_type: PositionType::Absolute,
                    left: percent(9),
                    top: percent(30),
                    ..default()
                },
            ));
            root.spawn((
                Card,
                Node {
                    position_type: PositionType::Absolute,
                    width: px(424),
                    padding: UiRect::all(px(20)),
                    flex_direction: FlexDirection::Column,
                    row_gap: px(10),
                    border: UiRect::left(px(3)),
                    ..default()
                },
                BorderColor::all(color(0xb4bc8b)),
                BackgroundColor(color(0x202f2b)),
            ))
            .with_children(|card| {
                card.spawn((Copy(0), Text::default(), font(24.), TextColor(color(CREAM))));
                card.spawn((
                    Copy(1),
                    Text::default(),
                    font(12.),
                    TextColor(color(0xb4bc8b)),
                ));
                card.spawn((
                    Copy(2),
                    Text::default(),
                    font(12.),
                    TextColor(color(0xe89a70)),
                ));
                for index in 0..5 {
                    card.spawn((
                        Control(index),
                        Button,
                        Node {
                            width: percent(100),
                            min_height: px(36),
                            padding: UiRect::axes(px(10), px(7)),
                            border: UiRect::all(px(1)),
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        BackgroundColor(color(0x344943)),
                        BorderColor::all(color(0x686955)),
                    ))
                    .with_child((
                        ControlText(index),
                        Text::default(),
                        font(15.),
                        TextColor(color(CREAM)),
                    ));
                }
                card.spawn((
                    Copy(3),
                    Text::new("Tab / Up/Down  Select   Enter  Confirm   Esc  Back"),
                    font(11.),
                    TextColor(color(0xa7b394)),
                ));
            });
        });
}
#[allow(clippy::too_many_arguments)]
pub fn input(
    mut game: ResMut<Playground>,
    window: Single<(Entity, &Window), With<PrimaryWindow>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut keyboard: MessageReader<KeyboardInput>,
    interactions: Query<(&Control, &Interaction), Changed<Interaction>>,
    mut exit: MessageWriter<AppExit>,
) {
    game.input_blocked = game.menu.screen != Screen::Playing;
    if !window.1.focused {
        keyboard.clear();
        return;
    }
    let mut action = None;
    let items = controls(&game.menu);
    for (control, interaction) in &interactions {
        if let Some((_, a, enabled)) = items.get(control.0) {
            // Hover changes the fill only. Reflow under a stationary pointer
            // must not steal keyboard focus on a newly opened screen.
            if *interaction == Interaction::Pressed && *enabled {
                game.menu.selected = control.0;
                action = Some(*a);
            }
        }
    }
    for event in keyboard
        .read()
        .filter(|e| e.window == window.0 && e.state == ButtonState::Pressed)
    {
        if event.key_code == KeyCode::Escape && !event.repeat {
            match game.menu.screen {
                Screen::Playing => {
                    release(&mut game);
                    game.menu.screen = Screen::Paused;
                    game.menu.selected = 0;
                }
                Screen::Paused | Screen::ConfirmStop => action = Some(Action::Resume),
                Screen::Connecting => action = Some(Action::Cancel),
                Screen::Main => {}
                _ => action = Some(Action::Back),
            }
            break;
        }
        if game.menu.screen == Screen::Playing {
            continue;
        }
        if matches!(
            event.key_code,
            KeyCode::Tab | KeyCode::ArrowDown | KeyCode::ArrowUp
        ) && !event.repeat
        {
            let items = controls(&game.menu);
            let backwards = event.key_code == KeyCode::ArrowUp
                || (event.key_code == KeyCode::Tab
                    && (keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)));
            for _ in 0..items.len() {
                game.menu.selected = (game.menu.selected
                    + if backwards { items.len() - 1 } else { 1 })
                    % items.len();
                if items[game.menu.selected].2 {
                    break;
                }
            }
        } else if matches!(
            event.key_code,
            KeyCode::Enter | KeyCode::NumpadEnter | KeyCode::Space
        ) && !event.repeat
        {
            if let Some((_, a, true)) = controls(&game.menu).get(game.menu.selected) {
                action = Some(*a);
            }
        } else if game.menu.selected == 0 && matches!(game.menu.screen, Screen::Host | Screen::Join)
        {
            let shortcut = [
                KeyCode::ControlLeft,
                KeyCode::ControlRight,
                KeyCode::SuperLeft,
                KeyCode::SuperRight,
            ]
            .into_iter()
            .any(|k| keys.pressed(k));
            game.menu.editor.key(event, shortcut);
        }
    }
    if let Some(action) = action
        && act(&mut game, action)
    {
        exit.write(AppExit::Success);
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn present(
    game: Res<Playground>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut root: Single<
        &mut Node,
        (
            With<MenuRoot>,
            Without<Card>,
            Without<Brand>,
            Without<Copy>,
            Without<Control>,
        ),
    >,
    mut card: Single<
        &mut Node,
        (
            With<Card>,
            Without<MenuRoot>,
            Without<Brand>,
            Without<Copy>,
            Without<Control>,
        ),
    >,
    mut brand: Single<
        (&mut Node, &mut TextFont, &mut Text),
        (
            With<Brand>,
            Without<MenuRoot>,
            Without<Card>,
            Without<Copy>,
            Without<Control>,
            Without<ControlText>,
        ),
    >,
    mut copy: Query<
        (&Copy, &mut Text, &mut TextFont, &mut Node),
        (
            Without<MenuRoot>,
            Without<Card>,
            Without<Brand>,
            Without<Control>,
            Without<ControlText>,
        ),
    >,
    mut buttons: Query<
        (
            &Control,
            &Interaction,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
        ),
        (
            Without<MenuRoot>,
            Without<Card>,
            Without<Brand>,
            Without<Copy>,
        ),
    >,
    mut labels: Query<(&ControlText, &mut Text, &mut TextColor), (Without<Copy>, Without<Brand>)>,
) {
    use crate::artwork::{CREAM, CYAN, color};
    root.display = if game.menu.screen == Screen::Playing {
        Display::None
    } else {
        Display::Flex
    };
    let compact = window.width() < 850. || window.height() < 540.;
    brand.0.left = if compact { px(20) } else { percent(9) };
    brand.0.top = if compact { px(10) } else { percent(30) };
    brand.0.display = if compact {
        Display::None
    } else {
        Display::Flex
    };
    brand.1.font_size = 48.0.into();
    brand.2.0 = if game.ember() {
        "BURNHOP\nEMBER RELAY"
    } else {
        "BURNHOP\nFIELD RANGE"
    }
    .into();
    card.left = if compact {
        px((window.width() - 448.).max(16.) / 2.)
    } else {
        percent(53)
    };
    card.top = if compact {
        px(8)
    } else {
        px((window.height() - 400.) / 2.)
    };
    card.width = px(if compact {
        window.width().min(464.) - 16.
    } else {
        424.
    });
    card.padding = UiRect::all(px(if compact { 12. } else { 20. }));
    card.row_gap = px(if compact { 6. } else { 10. });
    let session = &game.menu;
    let items = controls(session);
    let (title, detail) = match session.screen {
        Screen::Main => (if compact { "BURNHOP" } else { "READY, PILOT?" }, "Boots on. Pick your next drop.".into()),
        Screen::Host => ("HOST GAME", format!("Bind IP:port / {}\n{}", match address(&session.editor.text) { Ok(a) if a.ip().is_loopback() => "LOCAL ONLY", Ok(_) => "LAN INTERFACE", Err(_) => "ENTER AN ADDRESS" }, "For friends, enter this Mac/PC's LAN IP. No internet setup.")),
        Screen::Join => ("JOIN GAME", "Server IP:port / numeric IPv4 or [IPv6]:port\nUse the address shared by your host.".into()),
        Screen::Connecting => ("JOINING THE MATCH", format!("Connecting to {}\nYou can cancel safely at any time.", session.address.map_or(String::new(), |a| a.to_string()))),
        Screen::Paused => (if game.online.is_some() { "MATCH MENU" } else { "PRACTICE PAUSED" }, if game.online.is_some() { format!("Match continues / your controls are released.\n{} / {}/8 players", session.address.map_or(String::new(), |a| a.to_string()), game.online.as_ref().map_or(0, |o| o.actors.iter().flatten().count())) } else { if game.ember() { "Ember Relay / offline paused.".into() } else { "Take a breather. The range is paused.".into() } }),
        Screen::ConfirmStop => ("STOP HOSTING?", "Everyone will disconnect and this match will end.\nYour server's port will be released.".into()),
        Screen::Error => (if session.retry_host { "UNABLE TO HOST" } else { "CONNECTION ENDED" }, "You can edit the address and try again.".into()),
        Screen::Playing => ("", String::new()),
    };
    let validation = if matches!(session.screen, Screen::Host | Screen::Join) {
        address(&session.editor.text).err().unwrap_or_default()
    } else {
        session.notice.clone()
    };
    for (field, mut text, mut font, mut node) in &mut copy {
        match field.0 {
            0 => {
                text.0 = title.into();
                font.font_size = if compact { 20. } else { 24. }.into();
            }
            1 => {
                text.0 = detail.clone();
                node.display = if compact && session.screen == Screen::Main {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            2 => {
                text.0 = validation.clone();
                node.display = if validation.is_empty() {
                    Display::None
                } else {
                    Display::Flex
                };
            }
            _ => {
                text.0 = if matches!(session.screen, Screen::Host | Screen::Join)
                    && session.selected == 0
                {
                    "Left/Right Home End  Edit / Ctrl/Cmd+A  Select all\nTab  Next   Esc  Back"
                } else {
                    "Tab / Up/Down  Select   Enter  Confirm   Esc  Back"
                }
                .into()
            }
        }
    }
    for (control, interaction, mut node, mut background, mut border) in &mut buttons {
        let Some((_, _, enabled)) = items.get(control.0) else {
            node.display = Display::None;
            continue;
        };
        node.display = Display::Flex;
        node.min_height = px(36.);
        node.padding = UiRect::axes(px(10), px(if compact { 5. } else { 7. }));
        background.0 = color(if !enabled {
            0x293632
        } else if *interaction == Interaction::Pressed {
            0x566552
        } else if *interaction == Interaction::Hovered {
            0x445d50
        } else {
            0x344943
        });
        *border = BorderColor::all(color(if *enabled && session.selected == control.0 {
            CYAN
        } else {
            0x59614d
        }));
    }
    for (control, mut text, mut tint) in &mut labels {
        if let Some((label, _, enabled)) = items.get(control.0) {
            text.0 = label.clone();
            tint.0 = color(if *enabled { CREAM } else { 0x819080 });
        }
    }
}

#[cfg(test)]
#[path = "menu_tests.rs"]
mod tests;
