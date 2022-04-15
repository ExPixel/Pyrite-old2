use egui::{
    plot::{Bar, BarChart, HLine, Plot},
    Color32, Grid, Ui,
};
use util::circular::CircularBuffer;

use crate::GbaData;

#[derive(Default)]
pub struct PerformancePane {
    frame_times: CircularBuffer<f64, 32>,
    frame_processing_times: CircularBuffer<f64, 32>,
}

impl PerformancePane {
    const GBA_MAX_FRAME_DUR: f64 = 1000.0 / 60.0;

    const GOOD_FRAME_COLOR: Color32 = Color32::from_rgb(0x37, 0xb2, 0x4d);
    const BAD_FRAME_COLOR: Color32 = Color32::from_rgb(0xf0, 0x3e, 0x3e);

    pub(crate) fn render(&mut self, ui: &mut Ui, data: &mut GbaData) {
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
