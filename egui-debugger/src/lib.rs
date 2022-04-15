mod audio;
mod ioregs;
mod performance;

use egui::{Color32, Context, Visuals};
use gba::{Command, Gba};
use parking_lot::Mutex;
use pyrite::{CallbackId, GbaHandle, GbaThreadState};
use std::{sync::Arc, time::Duration};

#[derive(Default)]
pub struct Debugger {
    current_pane: Pane,
    performance_pane: performance::PerformancePane,
    audio_pane: audio::AudioPane,
    ioregs_pane: ioregs::IoRegistersPane,

    has_initialized: bool,

    gba_data: GbaData,
    gba_data_buffer: Arc<Mutex<GbaData>>,
    frame_callback: Option<CallbackId>,
}

impl Debugger {
    pub fn render(&mut self, ctx: &Context, gba: &GbaHandle) {
        if !self.has_initialized {
            self.init(ctx, gba);
            self.has_initialized = true;
        } else {
            self.fetch_updated_data();

            if self.gba_data.updated {
                ctx.request_repaint();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_pane, Pane::Performance, "Performance");
                ui.selectable_value(&mut self.current_pane, Pane::IoRegisters, "IO Registers");
                ui.selectable_value(&mut self.current_pane, Pane::Audio, "Audio");
            });

            match self.current_pane {
                Pane::Performance => self.performance_pane.render(ui, &mut self.gba_data),
                Pane::Audio => self.audio_pane.render(ui, &mut self.gba_data),
                Pane::IoRegisters => self.ioregs_pane.render(ui, &mut self.gba_data),
            }
        });
    }

    fn fetch_updated_data(&mut self) {
        let mut locked = self.gba_data_buffer.lock();
        self.gba_data.copy_data(&mut *locked);
    }

    #[cold]
    pub fn init(&mut self, ctx: &Context, gba: &GbaHandle) {
        let mut visuals = Visuals::dark();
        visuals.override_text_color = Some(Color32::from_rgb(0xe9, 0xec, 0xef));
        ctx.set_visuals(visuals);

        let gba_data_buffer = Arc::clone(&self.gba_data_buffer);
        let callback_id = gba.on_frame(move |gba, state| {
            let mut locked = gba_data_buffer.lock();
            pull_data_from_gba(&mut *locked, gba, state);
        });
        self.frame_callback = Some(callback_id);

        self.ioregs_pane.init();
    }

    pub fn destroy(&mut self, gba: &GbaHandle) {
        if let Some(cb) = self.frame_callback.take() {
            gba.remove_on_frame(cb);
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum Pane {
    Performance,
    Audio,
    IoRegisters,
}

impl Default for Pane {
    fn default() -> Self {
        Self::Performance
    }
}

#[derive(Default)]
struct GbaData {
    frame_duration: Option<Duration>,
    frame_processing_duration: Option<Duration>,

    audio_commands: Vec<Command>,
    has_audio_commands: bool,

    ioreg: Option<u32>,

    updated: bool,
    requests: GbaDataRequests,
}

impl GbaData {
    fn copy_data(&mut self, source: &mut GbaData) {
        if !source.updated {
            self.updated = false;
            return;
        }
        self.updated = true;
        source.updated = false;

        self.frame_duration = source.frame_duration.take();
        self.frame_processing_duration = source.frame_processing_duration.take();

        if self.requests.audio_data {
            self.audio_commands.clear();
            self.audio_commands
                .extend(source.audio_commands.iter().rev().copied());
            self.has_audio_commands = source.has_audio_commands;
        } else {
            self.has_audio_commands = false;
        }

        self.ioreg = source.ioreg.take();

        source.requests = std::mem::take(&mut self.requests);
    }
}

#[derive(Default)]
struct GbaDataRequests {
    frame_duration: bool,
    audio_data: bool,
    ioreg: Option<u32>,
}

fn pull_data_from_gba(data: &mut GbaData, gba: &mut Gba, state: &mut GbaThreadState) {
    if data.requests.frame_duration {
        data.frame_duration = Some(state.frame_duration());
        data.frame_processing_duration = Some(state.frame_processing_duration());
    }

    if data.requests.audio_data {
        data.audio_commands.clear();
        data.audio_commands.extend(gba.audio().commands());
        data.has_audio_commands = true;
    } else {
        data.has_audio_commands = false;
    }

    if let Some(address) = data.requests.ioreg {
        data.ioreg = Some(gba.memory_mut().view32(address));
    }

    data.updated = true;
}

const fn rgb(col: u32) -> Color32 {
    Color32::from_rgb((col >> 16) as u8, (col >> 8) as u8, col as u8)
}
