//! A single session-owned headless server. No child executable or renderer dependency.
use crate::{Server, ServerClock};
use std::{
    net::SocketAddr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub struct OwnedServer {
    address: SocketAddr,
    stop: Arc<AtomicBool>,
    failure: Arc<Mutex<Option<String>>>,
    worker: Option<JoinHandle<()>>,
}
impl OwnedServer {
    /// Bind before reporting success. Failed bind/spawn drops every created resource.
    pub fn start(address: SocketAddr) -> Result<Self, String> {
        let mut server = Server::bind(address)?;
        let address = server.address();
        let stop = Arc::new(AtomicBool::new(false));
        let failure = Arc::new(Mutex::new(None));
        let worker_stop = stop.clone();
        let worker_failure = failure.clone();
        let worker = thread::Builder::new()
            .name("burnhop-host".into())
            .spawn(move || {
                let mut clock = ServerClock::default();
                let mut last = Instant::now();
                while !worker_stop.load(Ordering::Acquire) {
                    let now = Instant::now();
                    let elapsed = now.duration_since(last);
                    last = now;
                    if let Err(error) = server.poll(elapsed) {
                        *worker_failure.lock().unwrap_or_else(|p| p.into_inner()) = Some(error);
                        break;
                    }
                    // Exactly the standalone server's bounded clock and shared simulation.
                    for _ in 0..clock.advance(elapsed.as_secs_f64()) {
                        server.tick();
                        if server.state.tick.is_multiple_of(2) {
                            server.snapshot();
                        }
                    }
                    server.flush();
                    thread::park_timeout(Duration::from_millis(1));
                }
                // Server::drop sends transport disconnects and releases the socket.
            })
            .map_err(|e| format!("Cannot start server worker: {e}"))?;
        Ok(Self {
            address,
            stop,
            failure,
            worker: Some(worker),
        })
    }
    pub fn address(&self) -> SocketAddr {
        self.address
    }
    pub fn failure(&self) -> Option<String> {
        self.failure
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .clone()
            .or_else(|| {
                self.worker
                    .as_ref()
                    .filter(|w| w.is_finished())
                    .map(|_| "Server worker stopped unexpectedly".into())
            })
    }
}
impl Drop for OwnedServer {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.thread().unpark();
            // Polling is nonblocking, catch-up is capped at five ticks. Joining
            // acknowledges socket release before a new session may bind this port.
            if worker.join().is_err() {
                eprintln!("Owned server worker panicked; its socket was released");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::UdpSocket;
    #[test]
    fn failed_start_does_not_own_occupied_port_and_retry_releases_it() {
        let occupied = UdpSocket::bind("127.0.0.1:0").unwrap();
        let address = occupied.local_addr().unwrap();
        assert!(OwnedServer::start(address).is_err());
        assert!(UdpSocket::bind(address).is_err());
        drop(occupied);
        for _ in 0..8 {
            let host = OwnedServer::start(address).unwrap();
            assert_eq!(host.address(), address);
            assert!(UdpSocket::bind(address).is_err());
            drop(host);
            drop(UdpSocket::bind(address).unwrap());
        }
    }
}
