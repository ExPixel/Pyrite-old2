use std::cell::RefCell;

use crossbeam::{
    channel::{self, Receiver, Sender, TryRecvError},
    sync::Parker,
};
use gba::Gba;

fn gba_thread_fn(rx: Receiver<GbaMessage>) {
    let mut gba = Gba::new();
    let mut state = GbaThreadState::default();

    log::trace!("waiting for GBA start");
    state.paused = true;
    wait_for_gba_unpause(&mut gba, &mut state, &rx);

    log::trace!("starting GBA thread loop");
    while !state.stopped {
        gba.frame();
        empty_gba_message_queue(&mut gba, &mut state, &rx);
        wait_for_gba_unpause(&mut gba, &mut state, &rx);
    }
    log::trace!("exited GBA thread loop");
}

fn empty_gba_message_queue(gba: &mut Gba, state: &mut GbaThreadState, rx: &Receiver<GbaMessage>) {
    loop {
        match rx.try_recv() {
            Ok(msg) => process_gba_message(msg, gba, state),
            Err(TryRecvError::Disconnected) => {
                log::trace!("no more GBA handles, shutting down");
                state.stop();
                break;
            }
            Err(TryRecvError::Empty) => break,
        }
    }
}

fn wait_for_gba_unpause(gba: &mut Gba, state: &mut GbaThreadState, rx: &Receiver<GbaMessage>) {
    while state.paused && !state.stopped {
        match rx.recv() {
            Ok(msg) => process_gba_message(msg, gba, state),
            Err(_) => {
                log::trace!("no more GBA handles, shutting down");
                state.stop();
                break;
            }
        }
    }
}

fn process_gba_message(msg: GbaMessage, gba: &mut Gba, state: &mut GbaThreadState) {
    match msg {
        GbaMessage::Shutdown => {
            log::trace!("GBA thread shutdown requested");
        }

        GbaMessage::CallbackImm(mut cb) => {
            (cb)(gba, state);
        }
    }
}

#[derive(Default)]
pub struct GbaThreadState {
    pub paused: bool,
    stopped: bool,
}

impl GbaThreadState {
    pub fn stop(&mut self) {
        self.stopped = true;
    }

    pub fn stopping(&self) -> bool {
        self.stopped
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

    pub fn after_frame<F>(&self, cb: F)
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        if self.tx.send(GbaMessage::CallbackImm(Box::new(cb))).is_err() {
            log::warn!("called `with` on disconnected GBA handle")
        }
    }

    pub fn after_frame_wait<F>(&self, mut cb: F)
    where
        F: 'static + Send + FnMut(&mut Gba, &mut GbaThreadState),
    {
        let mut parker = self.parker.borrow_mut();
        if parker.is_none() {
            *parker = Some(Parker::new());
        }
        let parker = parker.as_mut().unwrap();
        let unparker = parker.unparker().clone();

        self.after_frame(move |gba, state| {
            (cb)(gba, state);
            unparker.unpark();
        });

        parker.park();
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
    CallbackImm(Box<dyn 'static + Send + FnMut(&mut Gba, &mut GbaThreadState)>),
    Shutdown,
}
