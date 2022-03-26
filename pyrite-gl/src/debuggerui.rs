use debugger::Debugger;
use egui_glow::EguiGlow;
use glutin::{
    event::{ElementState, ModifiersState, VirtualKeyCode, WindowEvent},
    PossiblyCurrent, WindowedContext,
};
use pyrite::GbaHandle;

use crate::pyrite_window::PyriteWindow;

pub struct DebuggerWindow {
    context: Option<WindowedContext<PossiblyCurrent>>,
    gl: glow::Context,
    modifiers: ModifiersState,
    wants_close: bool,
    gui: EguiGlow,
    repaint: bool,
    debugger_state: Debugger,
    gba: GbaHandle,
}

impl DebuggerWindow {
    pub fn new(
        gba: GbaHandle,
        context: WindowedContext<PossiblyCurrent>,
    ) -> anyhow::Result<DebuggerWindow> {
        let gl = unsafe {
            glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _)
        };
        let gui = EguiGlow::new(context.window(), &gl);

        Ok(DebuggerWindow {
            context: Some(context),
            gl,
            modifiers: ModifiersState::default(),
            wants_close: false,
            gui,
            repaint: false,
            debugger_state: Debugger::default(),
            gba,
        })
    }

    fn on_keyboard_input(&mut self, input: glutin::event::KeyboardInput) {
        if let Some(VirtualKeyCode::Escape) = input.virtual_keycode {
            if input.state == ElementState::Pressed {
                self.wants_close = true
            }
        }
    }

    pub fn wants_close(&self) -> bool {
        self.wants_close
    }
}

impl Drop for DebuggerWindow {
    fn drop(&mut self) {
        self.debugger_state.destroy(&self.gba);
        if self.try_swap_context() {
            self.gui.destroy(&self.gl);
            log::debug!("destroyed debugger UI");
        } else {
            log::debug!("failed to swap to debugger window context for cleanup");
        }
    }
}

impl PyriteWindow for DebuggerWindow {
    fn on_window_event(&mut self, event: WindowEvent) {
        if self.gui.on_event(&event) {
            return;
        }

        match event {
            WindowEvent::KeyboardInput { input, .. } => self.on_keyboard_input(input),
            WindowEvent::Resized(..) => self.repaint = true,
            _ => (),
        }
    }

    fn render(&mut self) {
        let window = self.context.as_ref().expect("no context").window();
        if self.repaint {
            self.gui.paint(window, &self.gl);
            self.repaint = false;
        }
    }

    fn update(&mut self) -> bool {
        let window = self.context.as_ref().expect("no context").window();
        self.repaint = self.gui.run(window, |gui_context| {
            self.debugger_state.render(gui_context, &self.gba);
        });
        self.repaint
    }

    fn context_mut_opt(&mut self) -> &mut Option<glutin::WindowedContext<glutin::PossiblyCurrent>> {
        &mut self.context
    }

    fn context_opt(&self) -> &Option<glutin::WindowedContext<glutin::PossiblyCurrent>> {
        &self.context
    }

    fn modifiers_mut(&mut self) -> &mut ModifiersState {
        &mut self.modifiers
    }
}
