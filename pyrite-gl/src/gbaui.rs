use std::{
    rc::Rc,
    sync::{
        atomic::{self, AtomicBool, AtomicU16, Ordering},
        Arc,
    },
};

use anyhow::Context as _;
use gba::{Button, ButtonSet};
use glutin::{
    event::{ElementState, ModifiersState, VirtualKeyCode, WindowEvent},
    PossiblyCurrent, WindowedContext,
};
use parking_lot::Mutex;
use pyrite::{config::Config, GbaHandle};

use crate::{
    glutil::{
        self, AttribPtrType, Buffer, BufferTarget, BufferUsage, DrawMode, InternalTextureFormat,
        PixelDataType, Program, Shader, ShaderType, Texture, TextureFormat, UnlinkedProgram,
        VertexArray,
    },
    pyrite_window::PyriteWindow,
};

pub struct GbaWindow {
    context: Option<WindowedContext<PossiblyCurrent>>,
    gl: Rc<glow::Context>,

    gba: GbaHandle,
    buttons: ButtonSet,
    buttons_u16: Arc<AtomicU16>,

    screen_ready: Arc<AtomicBool>,
    screen: Arc<Mutex<[u16; 240 * 160]>>,
    screen_texture: Texture,
    screen_vbo: Buffer,
    screen_vao: VertexArray,
    screen_shader: Program,

    modifiers: ModifiersState,
    wants_exit: bool,
    wants_debugger: bool,

    config: Arc<Config>,
}

impl GbaWindow {
    pub fn new(
        config: Arc<Config>,
        context: WindowedContext<PossiblyCurrent>,
        gba: GbaHandle,
    ) -> anyhow::Result<GbaWindow> {
        let gl = Rc::new(unsafe {
            glow::Context::from_loader_function(|s| context.get_proc_address(s) as *const _)
        });

        let buffer_ready = Arc::new(AtomicBool::new(true));
        let buffer = Arc::new(Mutex::new([0; 240 * 160]));

        let gba_buffer_ready = Arc::clone(&buffer_ready);
        let gba_buffer = Arc::clone(&buffer);

        let buttons_u16 = Arc::new(AtomicU16::new(ButtonSet::default().into()));
        let buttons_u16_gba = buttons_u16.clone();

        gba.on_frame(move |gba, state| {
            let buttons = buttons_u16_gba.load(atomic::Ordering::Acquire);
            gba.set_buttons(ButtonSet::from(buttons));

            if !state.paused {
                let mut screen = gba_buffer.lock();
                screen.copy_from_slice(gba.video().screen());
                drop(screen);
                gba_buffer_ready.store(true, Ordering::Release);
            }
        });

        let screen_texture = Texture::builder()
            .width(240)
            .height(160)
            .internal_format(InternalTextureFormat::Rgb)
            .format(TextureFormat::Bgra)
            .build::<[u16]>(&gl, PixelDataType::UnsignedShort1555Rev, None)
            .map_err(anyhow::Error::msg)
            .context("error while creating screen texture")?;

        // Device Coordinates (left, right ,top, bottom):
        const DL: f32 = -1.0;
        const DR: f32 = 1.0;
        const DT: f32 = 1.0;
        const DB: f32 = -1.0;

        // Texture Coordinates (left, right, top, bottom):
        const TL: f32 = 0.0;
        const TR: f32 = 1.0;
        const TT: f32 = 0.0;
        const TB: f32 = 1.0;

        let vertices: &[f32] = &[
            DL, DT, TL, TT, // left, top
            DR, DT, TR, TT, // right, top
            DL, DB, TL, TB, // left, bottom
            DL, DB, TL, TB, // left, bottom
            DR, DB, TR, TB, // right, bottom
            DR, DT, TR, TT, // right, top
        ];

        let screen_vbo = Buffer::from_slice(
            &gl,
            BufferTarget::ArrayBuffer,
            vertices,
            BufferUsage::StaticDraw,
        )
        .map_err(anyhow::Error::msg)
        .context("error creating screen vertex buffer object")?;

        let vertex_shader =
            Shader::new(&gl, ShaderType::Vertex, include_str!("../shaders/gba.vert"))
                .map_err(anyhow::Error::msg)
                .context("error creating/compiling vertex shader")?;
        let fragment_shader = Shader::new(
            &gl,
            ShaderType::Fragment,
            include_str!("../shaders/gba.frag"),
        )
        .map_err(anyhow::Error::msg)
        .context("error creating/compiling fragmenet shader")?;
        let screen_shader = UnlinkedProgram::new(&gl, &[vertex_shader, fragment_shader])
            .map_err(anyhow::Error::msg)
            .context("error creating shader program")?;
        screen_shader.bind_frag_data_location(0, "out_color");
        let screen_shader = screen_shader
            .link()
            .map_err(anyhow::Error::msg)
            .context("error linking shader program")?;

        screen_shader.bind();
        glutil::uniform_1_i32(
            &gl,
            screen_shader
                .uniform_location("tex")
                .expect("no tex uniform"),
            0,
        );

        let screen_vao = VertexArray::new(&gl)
            .map_err(anyhow::Error::msg)
            .context("error creating vertex array")?;
        screen_vao.with(|vao| {
            let sz_float = std::mem::size_of::<f32>() as i32;
            let pos = screen_shader
                .get_attrib_location("in_position")
                .expect("no in_position attribute");
            let tex = screen_shader
                .get_attrib_location("in_texcoord")
                .expect("no in_texcoord attribute");
            vao.vertex_attrib_pointer_f32(pos, 2, AttribPtrType::Float, false, 4 * sz_float, 0);
            vao.vertex_attrib_pointer_f32(
                tex,
                2,
                AttribPtrType::Float,
                false,
                4 * sz_float,
                2 * sz_float,
            );
            vao.enable_attrib(pos);
            vao.enable_attrib(tex);
        });

        Ok(GbaWindow {
            context: Some(context),
            gl,

            gba,
            buttons: ButtonSet::default(),
            buttons_u16,

            screen_ready: buffer_ready,
            screen: buffer,
            screen_texture,
            screen_vbo,
            screen_vao,
            screen_shader,

            modifiers: ModifiersState::default(),
            wants_exit: false,
            wants_debugger: false,

            config,
        })
    }

    fn on_keyboard_input(&mut self, input: glutin::event::KeyboardInput) {
        let pressed = input.state == ElementState::Pressed;
        match input.virtual_keycode {
            Some(VirtualKeyCode::Escape) if pressed => self.wants_exit = true,
            Some(VirtualKeyCode::P) if pressed && self.modifiers.ctrl() => self
                .gba
                .after_frame(|_, state| state.paused = !state.paused),
            Some(VirtualKeyCode::R) if pressed && self.modifiers.ctrl() => {
                let boot_from_bios = self.config.gba.boot_from_bios.unwrap_or(true);
                self.gba
                    .after_frame(move |gba, _| gba.reset(boot_from_bios))
            }
            Some(VirtualKeyCode::D) if pressed && self.modifiers.ctrl() => {
                self.wants_debugger = true
            }

            Some(keycode) => {
                if let Some(button) = keycode_to_button(keycode) {
                    self.buttons.set_pressed(button, pressed);
                    self.buttons_u16
                        .store(self.buttons.into(), Ordering::Relaxed);
                }
            }

            _ => {}
        }
    }

    pub fn wants_exit(&mut self) -> bool {
        std::mem::replace(&mut self.wants_exit, false)
    }

    pub fn wants_debugger(&mut self) -> bool {
        std::mem::replace(&mut self.wants_debugger, false)
    }
}

impl PyriteWindow for GbaWindow {
    fn on_window_event(&mut self, event: WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            self.on_keyboard_input(input)
        }
    }

    fn render(&mut self) {
        glutil::clear(&self.gl, 0.2, 0.5, 0.5);

        if self
            .screen_ready
            .compare_exchange(true, false, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            let screen = self.screen.lock();
            self.screen_texture.update(
                None,
                TextureFormat::Rgba,
                PixelDataType::UnsignedShort1555Rev,
                &screen[..],
            );
        }

        self.screen_vbo.bind();
        self.screen_vao.bind();
        self.screen_shader.bind();
        self.screen_texture.bind(0);
        glutil::draw_arrays(&self.gl, DrawMode::Triangles, 0, 6);
        self.request_redraw();
    }

    fn context_mut_opt(&mut self) -> &mut Option<WindowedContext<PossiblyCurrent>> {
        &mut self.context
    }

    fn context_opt(&self) -> &Option<WindowedContext<PossiblyCurrent>> {
        &self.context
    }

    fn modifiers_mut(&mut self) -> &mut ModifiersState {
        &mut self.modifiers
    }
}

fn keycode_to_button(keycode: VirtualKeyCode) -> Option<Button> {
    match keycode {
        VirtualKeyCode::Z => Some(Button::A),
        VirtualKeyCode::X => Some(Button::B),
        VirtualKeyCode::Left => Some(Button::Left),
        VirtualKeyCode::Right => Some(Button::Right),
        VirtualKeyCode::Up => Some(Button::Up),
        VirtualKeyCode::Down => Some(Button::Down),
        VirtualKeyCode::A => Some(Button::L),
        VirtualKeyCode::S => Some(Button::R),
        VirtualKeyCode::Return => Some(Button::Start),
        VirtualKeyCode::Back => Some(Button::Select),
        _ => None,
    }
}
