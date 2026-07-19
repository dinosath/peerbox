use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

pub struct WindowHandle {
    pub width: u32,
    pub height: u32,
    pub title: String,
}

pub enum Event {
    WindowResized { width: u32, height: u32 },
    WindowClosed,
    KeyPressed { key: String },
}

pub struct EventLoop {
    running: Arc<AtomicBool>,
    notify: Arc<Notify>,
}

impl EventLoop {
    pub fn new() -> Self {
        Self {
            running: Arc::new(AtomicBool::new(true)),
            notify: Arc::new(Notify::new()),
        }
    }

    pub fn request_stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        self.notify.notify_one();
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for EventLoop {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_event_loop<F, Fut>(on_tick: F) -> anyhow::Result<()>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<()>>,
{
    let event_loop = EventLoop::new();
    while event_loop.is_running() {
        on_tick().await?;
        tokio::select! {
            _ = event_loop.notify.notified() => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_loop_new_is_running() {
        let el = EventLoop::new();
        assert!(el.is_running());
    }

    #[test]
    fn test_event_loop_stop() {
        let el = EventLoop::new();
        el.request_stop();
        assert!(!el.is_running());
    }

    #[test]
    fn test_window_handle_creation() {
        let w = WindowHandle {
            width: 800,
            height: 600,
            title: "Peerbox".to_string(),
        };
        assert_eq!(w.width, 800);
        assert_eq!(w.height, 600);
        assert_eq!(w.title, "Peerbox");
    }
}
