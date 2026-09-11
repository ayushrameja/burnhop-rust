use burnhop_server::{Server, ServerClock};
use std::time::{Duration, Instant};
fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut address = None;
    let mut diagnostics = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--bind" => {
                i += 1;
                address = Some(
                    args.get(i)
                        .ok_or("--bind needs an IP:port")?
                        .parse()
                        .map_err(|_| "invalid bind IP:port")?,
                );
            }
            "--diagnostics" => diagnostics = true,
            _ => return Err("Usage: burnhop-server --bind 127.0.0.1:5000 [--diagnostics]".into()),
        }
        i += 1;
    }
    let mut server =
        Server::bind(address.ok_or("Specify --bind 127.0.0.1:5000 (or an explicit LAN address)")?)?;
    server.diagnostics = diagnostics;
    println!(
        "Burnhop headless server on {} — 60 Hz, two players, protocol {}",
        server.address(),
        burnhop_protocol::PROTOCOL_VERSION
    );
    let mut clock = ServerClock::default();
    let mut last = Instant::now();
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(last);
        last = now;
        server.poll(elapsed)?;
        let dropped = clock.dropped_ticks;
        for _ in 0..clock.advance(elapsed.as_secs_f64()) {
            server.tick();
            if server.state.tick.is_multiple_of(2) {
                server.snapshot();
            }
        }
        if dropped != clock.dropped_ticks {
            eprintln!(
                "SERVER overload: dropped {} wall-clock ticks",
                clock.dropped_ticks - dropped
            );
        }
        server.flush();
        std::thread::sleep(Duration::from_millis(1));
    }
}
