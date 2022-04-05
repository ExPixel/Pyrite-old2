use egui::{
    plot::{Bar, BarChart, HLine, Line, Plot, Value, Values},
    Color32, Context, Grid, Ui, Visuals,
};
use gba::{Command, Gba, GbaAudioSampler};
use parking_lot::Mutex;
use pyrite::{CallbackId, GbaHandle, GbaThreadState};
use std::{sync::Arc, time::Duration};
use util::circular::CircularBuffer;

#[derive(Default)]
pub struct Debugger {
    current_pane: Pane,
    performance_pane: PerformancePane,
    audio_pane: AudioPane,

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
                ui.selectable_value(&mut self.current_pane, Pane::Audio, "Audio");
            });

            match self.current_pane {
                Pane::Performance => self.performance_pane.render(ui, &mut self.gba_data),
                Pane::Audio => self.audio_pane.render(ui, &mut self.gba_data),
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
}

impl Default for Pane {
    fn default() -> Self {
        Self::Performance
    }
}

struct AudioPane {
    sampler: GbaAudioSampler,
    samples_l: CircularBuffer<f32, { Self::BUFFER_SIZE }>,
    samples_r: CircularBuffer<f32, { Self::BUFFER_SIZE }>,
    commands_buffer_sizes: CircularBuffer<u32, { Self::FRAMES }>,
}

impl AudioPane {
    const RENDER_SAMPLES: u32 = 1024;
    const FRAMES: usize = 16;
    const BUFFER_SIZE: usize = Self::FRAMES * Self::RENDER_SAMPLES as usize;

    fn render(&mut self, ui: &mut Ui, data: &mut GbaData) {
        self.get_data(data);
        Grid::new("GBA Frame Durations Grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                ui.label("Output");
                self.render_samples_plot(ui);
                ui.end_row();

                ui.label("Commands");
                self.render_commands_buffer_plot(ui);
                ui.end_row();
            });
    }

    fn render_commands_buffer_plot(&mut self, ui: &mut Ui) {
        let mut bars = Vec::with_capacity(self.commands_buffer_sizes.len());
        for (idx, &size) in self.commands_buffer_sizes.iter().enumerate() {
            bars.push(Bar::new(idx as f64, size as f64));
        }

        let chart = BarChart::new(bars).name("Command Buffer Size Chart");
        Plot::new("Command Buffer Sizes")
            .allow_drag(false)
            .allow_zoom(false)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
            });
    }

    fn render_samples_plot(&mut self, ui: &mut Ui) {
        let samples_l = self
            .samples_l
            .iter()
            .enumerate()
            .map(|(idx, &s)| Value::new(idx as f64, s));
        let samples_r = self
            .samples_r
            .iter()
            .enumerate()
            .map(|(idx, &s)| Value::new(idx as f64, s));

        let line_l = Line::new(Values::from_values_iter(samples_l)).color(rgb(0x228be6));
        let line_r = Line::new(Values::from_values_iter(samples_r)).color(rgb(0xae3ec9));

        Plot::new("Samples")
            .show_axes([false, false])
            .allow_drag(true)
            .allow_zoom(true)
            .include_y(1.0)
            .include_y(-1.0)
            .show(ui, |plot_ui| {
                plot_ui.line(line_l);
                plot_ui.line(line_r);
            });
    }

    fn get_data(&mut self, data: &mut GbaData) {
        if std::mem::take(&mut data.has_audio_commands) {
            self.commands_buffer_sizes
                .push(data.audio_commands.len() as u32);
            for _ in 0..Self::RENDER_SAMPLES {
                while self.sampler.needs_commands() {
                    if let Some(command) = data.audio_commands.pop() {
                        self.sampler.command(command);
                    } else {
                        break;
                    }
                }

                let (l, r) = self.sampler.frame();
                self.samples_l.push(l);
                self.samples_r.push(r);
            }
        }
        data.requests.audio_data = true;
    }
}

impl Default for AudioPane {
    fn default() -> Self {
        AudioPane {
            sampler: GbaAudioSampler::new(Self::RENDER_SAMPLES),
            samples_l: Default::default(),
            samples_r: Default::default(),
            commands_buffer_sizes: Default::default(),
        }
    }
}

#[derive(Default)]
struct PerformancePane {
    frame_times: CircularBuffer<f64, 32>,
    frame_processing_times: CircularBuffer<f64, 32>,
}

impl PerformancePane {
    const GBA_MAX_FRAME_DUR: f64 = 1000.0 / 60.0;

    const GOOD_FRAME_COLOR: Color32 = Color32::from_rgb(0x37, 0xb2, 0x4d);
    const BAD_FRAME_COLOR: Color32 = Color32::from_rgb(0xf0, 0x3e, 0x3e);

    fn render(&mut self, ui: &mut Ui, data: &mut GbaData) {
        if let Some(duration) = data.frame_duration.take() {
            self.frame_times.push(duration.as_secs_f64() * 1000.0);
        }

        if let Some(duration) = data.frame_processing_duration.take() {
            self.frame_processing_times
                .push(duration.as_secs_f64() * 1000.0);
        }

        self.render_frame_times(ui);
        data.requests.frame_duration = true;
    }

    fn render_frame_times_text(&mut self, ui: &mut Ui) {
        let average_dur =
            self.frame_times.iter().copied().sum::<f64>() / self.frame_times.len() as f64;
        ui.label("Average GBA Frame Duration");
        ui.label(format!("{average_dur:0.2}ms"));
        ui.end_row();

        let average_fps = 1000.0 / average_dur;
        ui.label("Average GBA FPS");
        ui.label(format!("{average_fps:0.2}"));
        ui.end_row();

        let average_perf = (Self::GBA_MAX_FRAME_DUR / average_dur) * 100.0;
        ui.label("Average GBA Performance");
        ui.label(format!("{average_perf:0.2}%"));
        ui.end_row();
    }

    fn render_processing_times_plot(&mut self, ui: &mut Ui) {
        const PROCESSING_COLOR: Color32 = Color32::from_rgb(0x1c, 0x7e, 0xd6);
        let mut bars = Vec::with_capacity(self.frame_processing_times.len());
        for (idx, &t) in self.frame_processing_times.iter().skip(1).enumerate() {
            bars.push(Bar::new(idx as f64, t).fill(PROCESSING_COLOR));
        }

        if !self.frame_processing_times.is_empty() {
            bars.push(Bar::new(bars.len() as f64, 0.0));
        }

        let chart = BarChart::new(bars).name("GBA Frame Processing Duration");
        Plot::new("GBA Processing Durations")
            .show_x(false)
            .show_axes([false, false])
            .allow_drag(false)
            .allow_zoom(false)
            .include_y(Self::GBA_MAX_FRAME_DUR)
            .show(ui, |plot_ui| {
                plot_ui.bar_chart(chart);
            });
    }

    fn render_frame_times_plot(&mut self, ui: &mut Ui) {
        let bar_color = |t: f64| -> Color32 {
            if t > 4.0 {
                Self::BAD_FRAME_COLOR
            } else {
                Self::GOOD_FRAME_COLOR
            }
        };

        let bars = self
            .frame_times
            .iter()
            .enumerate()
            .map(|(idx, &t)| Bar::new(idx as f64, t).fill(bar_color(t)))
            .collect();
        let chart = BarChart::new(bars).name("GBA Frame Processing Duration");
        Plot::new("GBA Frame Durations")
            .show_x(false)
            .show_axes([false, false])
            .allow_drag(false)
            .allow_zoom(false)
            .include_y(Self::GBA_MAX_FRAME_DUR)
            .show(ui, |plot_ui| {
                plot_ui.hline(HLine::new(4.0).color(Self::BAD_FRAME_COLOR));
                plot_ui.bar_chart(chart);
            });
    }

    fn render_frame_times(&mut self, ui: &mut Ui) {
        Grid::new("GBA Frame Durations Grid")
            .num_columns(2)
            .striped(true)
            .show(ui, |ui| {
                self.render_frame_times_text(ui);

                ui.label("GBA Frame Durations");
                self.render_frame_times_plot(ui);
                ui.end_row();

                ui.label("GBA Processing Durations");
                self.render_processing_times_plot(ui);
                ui.end_row();
            });
    }
}

#[derive(Default)]
struct GbaData {
    frame_duration: Option<Duration>,
    frame_processing_duration: Option<Duration>,

    audio_commands: Vec<Command>,
    has_audio_commands: bool,

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

        source.requests = std::mem::take(&mut self.requests);
    }
}

#[derive(Default)]
struct GbaDataRequests {
    frame_duration: bool,
    audio_data: bool,
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

    data.updated = true;
}

const fn rgb(col: u32) -> Color32 {
    Color32::from_rgb((col >> 16) as u8, (col >> 8) as u8, col as u8)
}
