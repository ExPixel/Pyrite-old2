use egui::{
    plot::{Bar, BarChart, Line, Plot, Value, Values},
    Grid, Ui,
};
use gba::GbaAudioSampler;
use util::circular::CircularBuffer;

use crate::{rgb, GbaData};

pub struct AudioPane {
    sampler: GbaAudioSampler,
    samples_l: CircularBuffer<f32, { Self::BUFFER_SIZE }>,
    samples_r: CircularBuffer<f32, { Self::BUFFER_SIZE }>,
    commands_buffer_sizes: CircularBuffer<u32, { Self::FRAMES }>,
}

impl AudioPane {
    const RENDER_SAMPLES: u32 = 1024;
    const FRAMES: usize = 16;
    const BUFFER_SIZE: usize = Self::FRAMES * Self::RENDER_SAMPLES as usize;

    pub(crate) fn render(&mut self, ui: &mut Ui, data: &mut GbaData) {
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
            .allow_zoom(true)
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
