use std::{
    cell::RefCell,
    sync::atomic::{AtomicU64, Ordering},
    time::{Duration, Instant},
};

use crossbeam::{
    channel::{self, Receiver, Sender, TryRecvError},
    sync::Parker,
};
use gba::Gba;

type GbaThreadCallback = Box<dyn 'static + Send + FnMut(&mut Gba, &mut GbaThreadState, GbaEvent)>;
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
        ctx.state.frame_duration = frame_start_time.elapsed();
        ctx.state.frame_count += 1;

        ctx.on_event(GbaEvent::FRAME_READY);

        empty_gba_message_queue(&mut ctx, &rx);

        if ctx.state.paused {
            ctx.on_event(GbaEvent::PAUSED);
            wait_for_gba_unpause(&mut ctx, &rx);
            ctx.on_event(GbaEvent::UNPAUSED);
        }

        let frame_duration = frame_start_time.elapsed();
        ctx.state.frame_processing_duration = frame_duration;
        let target_frame_duration = Duration::from_secs_f64(1.0 / ctx.state.target_fps);
        if frame_duration < target_frame_duration {
            spin_sleeper.sleep(target_frame_duration - frame_duration);
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
        GbaMessage::WithGba(cb) => (cb)(&mut ctx.gba, &mut ctx.state),
        GbaMessage::Listen(id, cb, e) => ctx.on_event.push((id, cb, e)),
        GbaMessage::RemoveOnFrameCallback(rm_id) => ctx.on_event.retain(|&(id, ..)| id != rm_id),
    }
}

#[derive(Default)]
struct Context {
    gba: Gba,
    state: GbaThreadState,

    on_event: Vec<(CallbackId, GbaThreadCallback, GbaEvent)>,
}

impl Context {
    fn on_event(&mut self, event: GbaEvent) {
        let mut idx = 0;
        while idx < self.on_event.len() {
            let (_, ref mut cb, mask) = self.on_event[idx];
            if mask.contains(event) {
                self.state.remove_callback = false;
                (cb)(&mut self.gba, &mut self.state, event);
                if std::mem::take(&mut self.state.remove_callback) {
                    let _ = self.on_event.remove(idx);
                    continue;
                }
            }
            idx += 1;
        }
    }
}

#[derive(Default)]
pub struct GbaThreadState {
    pub paused: bool,
    stopped: bool,
    pub target_fps: f64,

    frame_count: u64,

    frame_duration: Duration,

    /// Duration including drawing the frame and running all of the callbacks.
    frame_processing_duration: Duration,

    /// When set to true, the currently executing callback will be marked for deletion.
    remove_callback: bool,
}

impl GbaThreadState {
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn stopping(&self) -> bool {
        self.stopped
    }

    /// Returns the amount of time required to render the current frame.
    pub fn frame_duration(&self) -> Duration {
        self.frame_duration
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Returns the amount of time required to render the previous frame
    /// and run all of the callbacks.
    pub fn frame_processing_duration(&self) -> Duration {
        self.frame_processing_duration
    }

    // Remove the current callback.
    pub fn remove_callback(&mut self) {
        self.remove_callback = true;
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

    pub fn shutdown(&self) {
        let _ = self.tx.send(GbaMessage::Shutdown);
    }

    fn on_events_internal(&self, events: GbaEvent, cb: GbaThreadCallback) -> CallbackId {
        let id = CallbackId::next_id();
        let result = self.tx.send(GbaMessage::Listen(id, cb, events));
        if result.is_err() {
            log::warn!("called `on_events` on disconnected GBA handle");
            CallbackId(0)
        } else {
            id
        }
    }

    pub fn on_events<F>(&self, events: GbaEvent, cb: F) -> CallbackId
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState, GbaEvent),
    {
        self.on_events_internal(events, Box::new(cb))
    }

    fn on_event_discard<F>(&self, event: GbaEvent, mut cb: F) -> CallbackId
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        self.on_events(event, move |gba, state, _| (cb)(gba, state))
    }

    pub fn on_frame<F>(&self, cb: F) -> CallbackId
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        self.on_event_discard(GbaEvent::FRAME_READY, cb)
    }

    pub fn on_pause_changed<F>(&self, cb: F) -> CallbackId
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        self.on_event_discard(GbaEvent::PAUSED | GbaEvent::UNPAUSED, cb)
    }

    pub fn remove_on_frame(&self, id: CallbackId) {
        if self.tx.send(GbaMessage::RemoveOnFrameCallback(id)).is_err() {
            log::warn!("called `remove_on_frame` on disconnected GBA handle");
        }
    }

    pub fn after_frame<F>(&self, cb: F)
    where
        F: 'static + Send + FnOnce(&mut Gba, &mut GbaThreadState),
    {
        if self.tx.send(GbaMessage::WithGba(Box::new(cb))).is_err() {
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

        let msg = GbaMessage::WithGba(Box::new(move |gba, state| {
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

impl Default for GbaHandle {
    fn default() -> Self {
        Self::new()
    }
}

enum GbaMessage {
    WithGba(GbaThreadCallbackOnce),
    Listen(CallbackId, GbaThreadCallback, GbaEvent),
    RemoveOnFrameCallback(CallbackId),
    Shutdown,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct CallbackId(u64);

impl CallbackId {
    fn next_id() -> Self {
        static NEXT_CALLBACK_ID: AtomicU64 = AtomicU64::new(1);
        let id = NEXT_CALLBACK_ID.fetch_add(1, Ordering::Relaxed);
        CallbackId(id)
    }
}

bitflags::bitflags! {
    pub struct GbaEvent: u32 {
        const FRAME_READY = 0x1;
        const PAUSED = 0x2;
        const UNPAUSED = 0x04;
    }
}
