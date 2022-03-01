use std::{
    cell::RefCell,
    time::{Duration, Instant},
};

use crossbeam::{
    channel::{self, Receiver, Sender, TryRecvError},
    sync::Parker,
};
use gba::Gba;

type GbaThreadCallback = Box<dyn 'static + Send + FnMut(&mut Gba, &mut GbaThreadState)>;
type GbaThreadCallbackOnce = Box<dyn 'static + Send + FnOnce(&mut Gba, &mut GbaThreadState)>;

fn gba_thread_fn(rx: Receiver<GbaMessage>) {
    let mut ctx = Context::default();
    ctx.state.target_fps = 60.0;

    log::trace!("waiting for GBA start");
    ctx.state.paused = true;
    wait_for_gba_unpause(&mut ctx, &rx);

    let spin_sleeper = spin_sleep::SpinSleeper::default();

    log::trace!("starting GBA thread loop");
    while !ctx.state.stopped {
        let frame_start_time = Instant::now();

        ctx.gba.frame();
        ctx.on_frame
            .iter_mut()
            .for_each(|cb| (cb)(&mut ctx.gba, &mut ctx.state));
        empty_gba_message_queue(&mut ctx, &rx);

        if ctx.state.paused {
            wait_for_gba_unpause(&mut ctx, &rx);
        } else {
            let frame_duration = frame_start_time.elapsed();
            ctx.state.frame_duration = frame_duration;
            let target_frame_duration = Duration::from_secs_f64(1.0 / ctx.state.target_fps);
            if frame_duration < target_frame_duration {
                spin_sleeper.sleep(target_frame_duration - frame_duration);
            }
        }
    }
    log::trace!("exited GBA thread loop");
}

fn empty_gba_message_queue(ctx: &mut Context, rx: &Receiver<GbaMessage>) {
    loop {
        match rx.try_recv() {
            Ok(msg) => process_gba_message(ctx, msg),
            Err(TryRecvError::Disconnected) => {
                log::trace!("no more GBA handles, shutting down");
                ctx.state.stop();
                break;
            }
            Err(TryRecvError::Empty) => break,
        }
    }
}

fn wait_for_gba_unpause(ctx: &mut Context, rx: &Receiver<GbaMessage>) {
    while ctx.state.paused && !ctx.state.stopped {
        match rx.recv() {
            Ok(msg) => process_gba_message(ctx, msg),
            Err(_) => {
                log::trace!("no more GBA handles, shutting down");
                ctx.state.stop();
                break;
            }
        }
    }
}

fn process_gba_message(ctx: &mut Context, msg: GbaMessage) {
    match msg {
        GbaMessage::Shutdown => {
            log::trace!("GBA thread shutdown requested");
            ctx.state.stopped = true;
        }
        GbaMessage::CallbackAfterFrame(cb) => (cb)(&mut ctx.gba, &mut ctx.state),
        GbaMessage::CallbackOnFrame(cb) => ctx.on_frame.push(cb),
    }
}

#[derive(Default)]
struct Context {
    gba: Gba,
    state: GbaThreadState,
    on_frame: Vec<GbaThreadCallback>,
}

#[derive(Default)]
pub struct GbaThreadState {
    pub paused: bool,
    stopped: bool,
    pub target_fps: f64,

    frame_duration: Duration,
}

impl GbaThreadState {
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn stopping(&self) -> bool {
        self.stopped
    }

    /// Returns the amount of time required to render the previous frame.
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }
}

/// A handle to a GBA instance running in its own thread.
pub struct GbaHandle {
    tx: Sender<GbaMessage>,
    parker: RefCell<Option<Parker>>,
}

impl GbaHandle {
    pub fn new() -> GbaHandle {
        let (tx, rx) = channel::unbounded();
        std::thread::spawn(move || gba_thread_fn(rx));
        let parker = RefCell::new(None);
        GbaHandle { tx, parker }
    }

    pub fn on_frame<F>(&self, cb: F)
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        if self
            .tx
            .send(GbaMessage::CallbackOnFrame(Box::new(cb)))
            .is_err()
        {
            log::warn!("called `on_frame` on disconnected GBA handle")
        }
    }

    pub fn after_frame<F>(&self, cb: F)
    where
        F: 'static + Send + FnOnce(&mut Gba, &mut GbaThreadState),
    {
        if self
            .tx
            .send(GbaMessage::CallbackAfterFrame(Box::new(cb)))
            .is_err()
        {
            log::warn!("called `after_frame` on disconnected GBA handle")
        }
    }

    pub fn after_frame_wait<F>(&self, cb: F)
    where
        F: 'static + Send + FnOnce(&mut Gba, &mut GbaThreadState),
    {
        let mut parker = self.parker.borrow_mut();
        if parker.is_none() {
            *parker = Some(Parker::new());
        }
        let parker = parker.as_mut().unwrap();
        let unparker = parker.unparker().clone();

        let msg = GbaMessage::CallbackAfterFrame(Box::new(move |gba, state| {
            (cb)(gba, state);
            unparker.unpark();
        }));

        if self.tx.send(msg).is_err() {
            log::warn!("called `after_frame_wait` on disconnected GBA handle")
        } else {
            parker.park();
        }
    }

    pub fn set_paused(&self, paused: bool) {
        self.after_frame(move |_, state| state.paused = paused);
    }
}

impl Clone for GbaHandle {
    fn clone(&self) -> Self {
        GbaHandle {
            tx: self.tx.clone(),
            parker: RefCell::new(None),
        }
    }
}

impl Drop for GbaHandle {
    fn drop(&mut self) {
        let _ = self.tx.send(GbaMessage::Shutdown);
    }
}

impl Default for GbaHandle {
    fn default() -> Self {
        Self::new()
    }
}

enum GbaMessage {
    CallbackAfterFrame(GbaThreadCallbackOnce),
    CallbackOnFrame(GbaThreadCallback),
    Shutdown,
}
