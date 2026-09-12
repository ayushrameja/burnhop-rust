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
        "Burnhop headless server on {} — 60 Hz, eight players, protocol {}",
        server.address(),
        burnhop_protocol::PROTOCOL_VERSION
    );
    let mut clock = ServerClock::default();
    let mut last = Instant::now();
    let mut last_report = last;
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
        if diagnostics && now.duration_since(last_report).as_secs_f64() >= 5. {
            let stats = server.input_stats();
            println!(
                "SERVER_METRICS tick={} players={} tick_us_p95={:.3} tick_us_max={:.3} dropped={} accepted={} applied={} missing={} late_redundant_copies={} queue_peak={} snapshot_max={} snapshot_bytes_enqueued={} app_bytes_received={}",
                server.state.tick,
                server.player_count(),
                server.tick_work_us.percentile(0.95),
                server.tick_work_us.max,
                clock.dropped_ticks,
                stats.accepted,
                stats.applied,
                stats.missing,
                stats.late,
                stats.peak,
                server.snapshot_max,
                server.application_sent,
                server.application_received
            );
            last_report = now;
        }
        server.flush();
        std::thread::sleep(Duration::from_millis(1));
    }
}
