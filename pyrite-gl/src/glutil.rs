#![allow(dead_code)]

use std::rc::Rc;

use glow::{Context as GlContext, HasContext, NativeUniformLocation, PixelUnpackData};

pub fn uniform_1_i32(gl: &GlContext, location: NativeUniformLocation, x: i32) {
    unsafe { gl.uniform_1_i32(Some(&location), x) }
}

pub fn clear(gl: &GlContext, r: f32, g: f32, b: f32) {
    unsafe {
        gl.clear_color(r, g, b, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
    }
}

pub fn draw_arrays(gl: &GlContext, mode: DrawMode, first: i32, count: i32) {
    unsafe { gl.draw_arrays(mode.gl_value(), first, count) }
}

pub fn log_gl_error(gl: &GlContext) {
    let err = unsafe { gl.get_error() };
    if err != glow::NO_ERROR {
        log::error!("OpenGL error: 0x{err:X}");
    }
}

pub fn assert_gl_error(gl: &GlContext, tag: &dyn std::fmt::Display) {
    let err = unsafe { gl.get_error() };
    if err != glow::NO_ERROR {
        log::error!("OpenGL error: 0x{err:X} ({tag})");
        std::process::exit(1);
    }
}

pub struct VertexArray {
    native: <GlContext as HasContext>::VertexArray,
    gl: Rc<GlContext>,
}

impl VertexArray {
    pub fn new(gl: &Rc<GlContext>) -> Result<Self, String> {
        unsafe {
            let nva = gl.create_vertex_array()?;
            Ok(VertexArray {
                native: nva,
                gl: Rc::clone(gl),
            })
        }
    }

    pub fn with<F>(&self, f: F)
    where
        F: FnOnce(VertexArrayEditor),
    {
        self.bind();
        (f)(VertexArrayEditor {
            native: self.native,
            gl: &self.gl,
        });
    }

    pub fn bind(&self) {
        unsafe { self.gl.bind_vertex_array(Some(self.native)) };
    }
}

pub struct VertexArrayEditor<'g> {
    native: <GlContext as HasContext>::VertexArray,
    gl: &'g GlContext,
}

impl<'g> VertexArrayEditor<'g> {
    pub fn enable_attrib(&self, index: u32) {
        unsafe { self.gl.enable_vertex_attrib_array(index) };
    }

    pub fn vertex_attrib_pointer_f32(
        &self,
        index: u32,
        size: i32,
        data_type: AttribPtrType,
        normalized: bool,
        stride: i32,
        offset: i32,
    ) {
        unsafe {
            self.gl.vertex_attrib_pointer_f32(
                index,
                size,
                data_type.gl_value(),
                normalized,
                stride,
                offset,
            )
        };
    }
}

pub struct Program {
    native: <GlContext as HasContext>::Program,
    gl: Rc<GlContext>,
}

impl Program {
    fn new(mut unlinked: UnlinkedProgram) -> Result<Self, String> {
        unsafe {
            unlinked.gl.link_program(unlinked.native);
            if !unlinked.gl.get_program_link_status(unlinked.native) {
                return Err(unlinked.gl.get_program_info_log(unlinked.native));
            }
        }
        unlinked.requires_delete = false;
        Ok(Program {
            native: unlinked.native,
            gl: Rc::clone(&unlinked.gl),
        })
    }

    pub fn bind(&self) {
        unsafe { self.gl.use_program(Some(self.native)) };
    }

    pub fn get_attrib_location(&self, name: &str) -> Option<u32> {
        unsafe { self.gl.get_attrib_location(self.native, name) }
    }

    pub fn uniform_location(&self, name: &str) -> Option<NativeUniformLocation> {
        unsafe { self.gl.get_uniform_location(self.native, name) }
    }
}

pub struct UnlinkedProgram {
    native: <GlContext as HasContext>::Program,
    gl: Rc<GlContext>,
    requires_delete: bool,
}

impl UnlinkedProgram {
    pub fn new(gl: &Rc<GlContext>, shaders: &[Shader]) -> Result<Self, String> {
        unsafe {
            let np = gl.create_program()?;
            for shader in shaders {
                gl.attach_shader(np, shader.native);
            }

            Ok(UnlinkedProgram {
                native: np,
                gl: Rc::clone(gl),
                requires_delete: true,
            })
        }
    }

    pub fn bind_frag_data_location(&self, color_number: u32, name: &str) {
        unsafe {
            self.gl
                .bind_frag_data_location(self.native, color_number, name)
        };
    }

    pub fn link(self) -> Result<Program, String> {
        Program::new(self)
    }
}

impl Drop for UnlinkedProgram {
    fn drop(&mut self) {
        if self.requires_delete {
            unsafe { self.gl.delete_program(self.native) };
        }
    }
}

pub struct Shader {
    native: <GlContext as HasContext>::Shader,
    gl: Rc<GlContext>,
}

impl Shader {
    pub fn new(gl: &Rc<GlContext>, shader_type: ShaderType, source: &str) -> Result<Self, String> {
        unsafe {
            let ns = gl.create_shader(shader_type.gl_value())?;
            gl.shader_source(ns, source);
            gl.compile_shader(ns);

            if !gl.get_shader_compile_status(ns) {
                return Err(gl.get_shader_info_log(ns));
            }

            Ok(Shader {
                native: ns,
                gl: Rc::clone(gl),
            })
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { self.gl.delete_shader(self.native) };
    }
}

pub struct Buffer {
    native: <GlContext as HasContext>::Buffer,
    gl: Rc<GlContext>,
    target: BufferTarget,
}

impl Buffer {
    pub fn from_slice<T>(
        gl: &Rc<GlContext>,
        target: BufferTarget,
        data: &[T],
        usage: BufferUsage,
    ) -> Result<Self, String> {
        let as_u8_slice = unsafe {
            std::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                std::mem::size_of::<T>() * data.len(),
            )
        };
        Self::new(gl, target, Some(as_u8_slice), usage)
    }

    pub fn new(
        gl: &Rc<GlContext>,
        target: BufferTarget,
        data: Option<&[u8]>,
        usage: BufferUsage,
    ) -> Result<Self, String> {
        unsafe {
            let nb = gl.create_buffer()?;
            gl.bind_buffer(target.gl_value(), Some(nb));
            if let Some(data) = data {
                gl.buffer_data_u8_slice(target.gl_value(), data, usage.gl_value());
            }

            Ok(Buffer {
                native: nb,
                gl: Rc::clone(gl),
                target,
            })
        }
    }

    pub fn bind(&self) {
        unsafe {
            self.gl
                .bind_buffer(self.target.gl_value(), Some(self.native))
        };
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { self.gl.delete_buffer(self.native) };
    }
}

pub struct Texture {
    native: <GlContext as HasContext>::Texture,
    gl: Rc<GlContext>,
    target: u32,
    width: u32,
    height: u32,
}

impl Texture {
    pub fn builder() -> TextureBuilder {
        TextureBuilder::new()
    }

    pub fn bind(&mut self, unit: u32) {
        unsafe {
            self.gl.active_texture(Self::active_texture_unit(unit));
            self.gl.bind_texture(self.target, Some(self.native));
        }
    }

    pub fn update<P: ?Sized + PixelData>(
        &self,
        rect: Option<Rectangle>,
        format: TextureFormat,
        data_type: PixelDataType,
        pixels: &P,
    ) {
        if self.target != glow::TEXTURE_2D {
            panic!("only 2D textures supported")
        }

        let rect = rect.unwrap_or_else(|| Rectangle::new(0, 0, self.width, self.height));
        let x = rect.x.clamp(0, self.width) as i32;
        let y = rect.y.clamp(0, self.height) as i32;
        let w = rect.w.clamp(0, self.width) as i32;
        let h = rect.h.clamp(0, self.height) as i32;

        unsafe {
            self.gl.bind_texture(self.target, Some(self.native));
            self.gl.tex_sub_image_2d(
                self.target,
                0,
                x,
                y,
                w,
                h,
                format.gl_value(),
                data_type.gl_value(),
                PixelUnpackData::Slice(pixels.as_u8_slice()),
            );
        }
    }

    fn active_texture_unit(unit: u32) -> u32 {
        const TEXTURE_UNITS: [u32; 8] = [
            glow::TEXTURE0,
            glow::TEXTURE1,
            glow::TEXTURE2,
            glow::TEXTURE3,
            glow::TEXTURE4,
            glow::TEXTURE5,
            glow::TEXTURE6,
            glow::TEXTURE7,
        ];

        if let Some(&tex) = TEXTURE_UNITS.get(unit as usize) {
            tex
        } else {
            panic!("unsupported texture unit {unit}");
        }
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { self.gl.delete_texture(self.native) };
    }
}

pub struct TextureBuilder {
    target: u32,
    internal_format: Option<InternalTextureFormat>,
    width: Option<i32>,
    height: Option<i32>,
    border: i32,
    format: Option<TextureFormat>,
}

impl TextureBuilder {
    fn new() -> TextureBuilder {
        TextureBuilder {
            target: glow::TEXTURE_2D,
            internal_format: None,
            width: None,
            height: None,
            border: 0,
            format: None,
        }
    }

    pub fn build<P: ?Sized + PixelData>(
        self,
        gl: &Rc<GlContext>,
        data_type: PixelDataType,
        data: Option<&P>,
    ) -> Result<Texture, String> {
        if self.target != glow::TEXTURE_2D {
            panic!("only 2D textures supported")
        }

        unsafe {
            let nt = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(nt));
            gl.tex_image_2d(
                self.target,
                0,
                self.internal_format.expect("no internal format").gl_value() as _,
                self.width.expect("no width"),
                self.height.expect("no height"),
                self.border,
                self.format.expect("no format").gl_value() as _,
                data_type.gl_value(),
                data.map(|d| d.as_u8_slice()),
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as _,
            );
            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as _,
            );
            Ok(Texture {
                native: nt,
                gl: Rc::clone(gl),
                target: self.target,
                width: self.width.unwrap() as u32,
                height: self.height.unwrap() as u32,
            })
        }
    }

    pub fn internal_format(mut self, internal_format: InternalTextureFormat) -> Self {
        self.internal_format = Some(internal_format);
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = Some(width.try_into().expect("out of range"));
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = Some(height.try_into().expect("out of range"));
        self
    }

    pub fn border(mut self, border: u32) -> Self {
        self.border = border.try_into().expect("out of range");
        self
    }

    pub fn format(mut self, format: TextureFormat) -> Self {
        self.format = Some(format);
        self
    }
}

impl PixelData for [u8] {
    fn as_u8_slice(&self) -> &[u8] {
        self
    }
}

impl PixelData for [u16] {
    fn as_u8_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 2) }
    }
}

#[derive(Copy, Clone)]
pub enum InternalTextureFormat {
    Rgb,
    Rgba,
    Rgb8,
    Rgba8,
}

impl InternalTextureFormat {
    fn gl_value(self) -> u32 {
        use InternalTextureFormat::*;
        match self {
            Rgb => glow::RGB,
            Rgba => glow::RGBA,
            Rgb8 => glow::RGB8,
            Rgba8 => glow::RGBA8,
        }
    }
}

#[derive(Copy, Clone)]
pub enum TextureFormat {
    Red,
    Rg,
    Rgb,
    Bgr,
    Rgba,
    Bgra,
}

impl TextureFormat {
    fn gl_value(self) -> u32 {
        use TextureFormat::*;
        match self {
            Red => glow::RED,
            Rg => glow::RG,
            Rgb => glow::RGB,
            Bgr => glow::BGR,
            Rgba => glow::RGBA,
            Bgra => glow::BGRA,
        }
    }
}

#[derive(Copy, Clone)]
pub enum PixelDataType {
    UnsignedByte,
    UnsignedShort5551,
    UnsignedShort1555Rev,
}

impl PixelDataType {
    fn gl_value(self) -> u32 {
        use PixelDataType::*;

        match self {
            UnsignedByte => glow::UNSIGNED_BYTE,
            UnsignedShort5551 => glow::UNSIGNED_SHORT_5_5_5_1,
            UnsignedShort1555Rev => glow::UNSIGNED_SHORT_1_5_5_5_REV,
        }
    }
}

pub trait PixelData {
    fn as_u8_slice(&self) -> &[u8];
}

#[derive(Copy, Clone)]
pub enum BufferTarget {
    ArrayBuffer,
}

impl BufferTarget {
    fn gl_value(self) -> u32 {
        use BufferTarget::*;

        match self {
            ArrayBuffer => glow::ARRAY_BUFFER,
        }
    }
}

#[derive(Copy, Clone)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
    StreamDraw,
    StaticRead,
}

impl BufferUsage {
    fn gl_value(self) -> u32 {
        use BufferUsage::*;

        match self {
            StaticDraw => glow::STATIC_DRAW,
            DynamicDraw => glow::DYNAMIC_DRAW,
            StreamDraw => glow::STREAM_DRAW,
            StaticRead => glow::STATIC_READ,
        }
    }
}

#[derive(Copy, Clone)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl ShaderType {
    fn gl_value(self) -> u32 {
        use ShaderType::*;

        match self {
            Vertex => glow::VERTEX_SHADER,
            Fragment => glow::FRAGMENT_SHADER,
        }
    }
}

#[derive(Copy, Clone)]
pub enum AttribPtrType {
    Float,
}

impl AttribPtrType {
    fn gl_value(self) -> u32 {
        use AttribPtrType::*;

        match self {
            Float => glow::FLOAT,
        }
    }
}

#[derive(Copy, Clone)]
pub enum DrawMode {
    Triangles,
}

impl DrawMode {
    fn gl_value(self) -> u32 {
        use DrawMode::*;

        match self {
            Triangles => glow::TRIANGLES,
        }
    }
}

#[derive(Copy, Clone, Default, PartialEq, Eq)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

impl Rectangle {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        Rectangle { x, y, w, h }
    }
}
