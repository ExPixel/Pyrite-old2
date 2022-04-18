use egui::{ComboBox, Grid, Ui};
use gba::memory::io;
use std::collections::HashMap;

use crate::GbaData;

#[derive(Default)]
pub struct IoRegistersPane {
    register: IoRegisters,
    value: Option<u32>,
    track_value: bool,
    ioreg_display: HashMap<IoRegisters, String>,
}

impl IoRegistersPane {
    pub(crate) fn render(&mut self, ui: &mut Ui, data: &mut GbaData) {
        if let Some(value) = data.ioreg.take() {
            self.value = Some(value);
        }

        let mut request_new_reg = false;
        ComboBox::from_label("IO Register")
            .selected_text(&self.ioreg_display[&self.register])
            .show_ui(ui, |ui| {
                for &register in IoRegisters::ALL {
                    let response = ui.selectable_value(
                        &mut self.register,
                        register,
                        &self.ioreg_display[&register],
                    );
                    request_new_reg |= response.changed();
                }
            });

        if request_new_reg {
            self.value = None;
        }

        if let Some(value) = self.value {
            let render_reg = self.register.render_function();
            (render_reg)(value, ui);

            if self.track_value {
                data.requests.ioreg = Some(self.register.address());
            }
        } else {
            ui.label("Waiting for value...");
            data.requests.ioreg = Some(self.register.address());
        }
    }

    pub(crate) fn init(&mut self) {
        self.track_value = true;
        self.ioreg_display
            .extend(IoRegisters::ALL.iter().map(|&reg| {
                let addr = reg.address();
                let short_name = reg.short_name();
                let long_name = reg.long_name();
                let text = format!("[0x{addr:08X}] {short_name} - {long_name}",);
                (reg, text)
            }));
    }
}

macro_rules! io_registers {
    (
        $( [$Address:expr, $Width:expr, $EnumName:ident, $ShortName:expr, $LongName:expr, $RenderFn:expr] ),+ $(,)?
    ) => {
        #[derive(Copy, Clone, Hash, PartialEq, Eq)]
        pub enum IoRegisters {
            $($EnumName),+
        }

        impl IoRegisters {
            const ALL: &'static [IoRegisters] = &[
                $(Self::$EnumName),+
            ];

            fn long_name(&self) -> &'static str {
                match self {
                    $( Self::$EnumName => $LongName ),+
                }
            }

            fn short_name(&self) -> &'static str {
                match self {
                    $( Self::$EnumName => $ShortName ),+
                }
            }

            fn address(&self) -> u32 {
                match self {
                    $( Self::$EnumName => $Address ),+
                }
            }

            fn render_function(&self) -> fn(u32, &mut Ui) {
                match self {
                    $( Self::$EnumName => $RenderFn ),+
                }
            }
        }
    };
}

io_registers! {
    [io::DISPCNT, 2, Dispcnt, "DISPCNT", "LCD Control", render_dispcnt],
    [io::DISPSTAT, 2, Dispstat, "DISPSTAT", "LCD Status", render_dispstat],
    [io::VCOUNT, 2, Vcount, "VCOUNT", "Vertical Counter", render_u16],
    [io::BG0CNT, 2, Bg0cnt, "BG0CNT", "BG0 Control", render_bgcnt],
    [io::BG1CNT, 2, Bg1cnt, "BG1CNT", "BG1 Control", render_bgcnt],
    [io::BG2CNT, 2, Bg2cnt, "BG2CNT", "BG2 Control", render_bgcnt],
    [io::BG3CNT, 2, Bg3cnt, "BG3CNT", "BG3 Control", render_bgcnt],
}

impl Default for IoRegisters {
    fn default() -> Self {
        IoRegisters::Dispcnt
    }
}

#[allow(dead_code)]
fn render_u16(value: u32, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let value = value as u16;
        ui.label("Value");
        ui.label(format!("{value:04X}"));
    });
}

#[allow(dead_code)]
fn render_u32(value: u32, ui: &mut Ui) {
    ui.horizontal(|ui| {
        let lo = value as u16;
        let hi = (value >> 16) as u16;

        ui.label("Value");
        ui.label(format!("{hi:04X} {lo:04X}"));
    });
}

fn render_dispcnt(value: u32, ui: &mut Ui) {
    use io::{LCDControl, ObjCharVramMapping};

    let value = value as u16;
    let dispcnt = LCDControl::new(value);

    Grid::new("DISPCNT values").show(ui, |ui| {
        ui.label("Value");
        ui.label(format!("{value:04X}"));
        ui.end_row();

        ui.label("Mode");
        ui.label(dispcnt.bg_mode().to_string());
        ui.end_row();

        ui.label("CGB Mode");
        ui.label(dispcnt.cgb_mode().to_string());
        ui.end_row();

        ui.label("Frame Select");
        ui.label(dispcnt.frame().to_string());
        ui.end_row();

        ui.label("H-Blank Interval Free");
        ui.label(dispcnt.hblank_interval_free().to_string());
        ui.end_row();

        ui.label("OBJ Character VRAM Mapping");
        ui.label(
            if dispcnt.obj_char_vram_mapping() == ObjCharVramMapping::OneDimensional {
                "One Dimensional"
            } else {
                "Two Dimensional"
            },
        );
        ui.end_row();

        ui.label("Forced Blank");
        ui.label(dispcnt.forced_blank().to_string());
        ui.end_row();

        for bg in 0..3 {
            ui.label(format!("Display BG{bg}"));
            ui.label(dispcnt.display_bg(bg).to_string());
            ui.end_row();
        }

        ui.label("Display OBJ");
        ui.label(dispcnt.display_obj().to_string());
        ui.end_row();

        ui.label("Window 0 Display");
        ui.label(dispcnt.win0_display().to_string());
        ui.end_row();

        ui.label("Window 1 Display");
        ui.label(dispcnt.win1_display().to_string());
        ui.end_row();

        ui.label("OBJ Window Display");
        ui.label(dispcnt.obj_window_display().to_string());
        ui.end_row();
    });
}

fn render_dispstat(value: u32, ui: &mut Ui) {
    use gba::memory::io::LCDStatus;

    let value = value as u16;
    let dispstat = LCDStatus::new(value);

    Grid::new("DISPSTAT values").show(ui, |ui| {
        ui.label("Value");
        ui.label(format!("{value:04X}"));
        ui.end_row();

        ui.label("V-Blank Flag");
        ui.label(dispstat.vblank().to_string());
        ui.end_row();

        ui.label("H-Blank Flag");
        ui.label(dispstat.hblank().to_string());
        ui.end_row();

        ui.label("V-Counter Flag");
        ui.label(dispstat.vcounter_match().to_string());
        ui.end_row();

        ui.label("H-Blank IRQ");
        ui.label(enabled(dispstat.hblank_irq_enable()));
        ui.end_row();

        ui.label("V-Blank IRQ");
        ui.label(enabled(dispstat.vblank_irq_enable()));
        ui.end_row();

        ui.label("V-Counter IRQ");
        ui.label(enabled(dispstat.vcounter_irq_enable()));
        ui.end_row();

        ui.label("V-Counter Setting");
        ui.label(dispstat.vcount_setting().to_string());
        ui.end_row();
    });
}

fn render_bgcnt(value: u32, ui: &mut Ui) {
    use gba::memory::io::BgControl;

    let value = value as u16;
    let bgcnt = BgControl::new(value);

    Grid::new("BGCNT values").show(ui, |ui| {
        ui.label("Value");
        ui.label(format!("{value:04X}"));
        ui.end_row();

        ui.label("Priority");
        ui.label(bgcnt.priority().to_string());
        ui.end_row();

        ui.label("Character Base");
        ui.label(format!("+0x{:04X}", bgcnt.character_base()));
        ui.end_row();

        ui.label("Mosaic");
        ui.label(enabled(bgcnt.mosaic()));
        ui.end_row();

        ui.label("Palettes");
        ui.label(if bgcnt.palette_256() {
            "256/1"
        } else {
            "16/16"
        });
        ui.end_row();

        ui.label("Screen Base");
        ui.label(format!("+0x{:04X}", bgcnt.screen_base()));
        ui.end_row();

        let size = bgcnt.screen_size();
        ui.label("Screen Size");
        ui.label(format!(
            "{}x{} / {}x{} (rotscale)",
            size.width(false),
            size.height(false),
            size.width(true),
            size.height(false)
        ));
        ui.end_row();
    });
}

fn enabled(b: bool) -> &'static str {
    if b {
        "Enabled"
    } else {
        "Disabled"
    }
}
