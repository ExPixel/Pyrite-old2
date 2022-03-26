use glutin::{
    event::{ModifiersState, WindowEvent},
    event_loop::ControlFlow,
    window::{Window, WindowId},
    PossiblyCurrent, WindowedContext,
};

pub trait PyriteWindow {
    fn process_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            *self.modifiers_mut() = modifiers;
        }
        self.on_window_event(event)
    }

    fn on_window_event(&mut self, _event: WindowEvent) {}

    fn gl_render(&mut self, _flow: &mut ControlFlow) {
        if !self.try_swap_context() {
            return;
        }
        self.render();
        self.present();
    }

    fn render(&mut self);

    /// Returns true if the window should rerender.
    fn update(&mut self) -> bool {
        true
    }

    fn modifiers_mut(&mut self) -> &mut ModifiersState;

    fn try_swap_context(&mut self) -> bool {
        if self.context().is_current() {
            return true;
        }

        let mut success = true;
        if let Some(mut context) = self.context_mut_opt().take() {
            context = match unsafe { context.make_current() } {
                Ok(new_context) => new_context,
                Err((new_context, err)) => {
                    success = false;
                    log::error!("error while switching context: {err:?}");
                    new_context
                }
            };
            *self.context_mut_opt() = Some(context);
        }
        success
    }

    fn context_mut_opt(&mut self) -> &mut Option<WindowedContext<PossiblyCurrent>>;
    fn context_opt(&self) -> &Option<WindowedContext<PossiblyCurrent>>;
    fn context(&self) -> &WindowedContext<PossiblyCurrent> {
        self.context_opt().as_ref().unwrap()
    }

    fn present(&mut self) {
        if let Err(err) = self.context_opt().as_ref().unwrap().swap_buffers() {
            log::error!("error occurred while swapping context buffers: {err:?}");
        }
    }

    fn window(&self) -> &Window {
        self.context().window()
    }

    fn window_id(&self) -> WindowId {
        self.window().id()
    }

    fn request_redraw(&self) {
        self.window().request_redraw()
    }
}
