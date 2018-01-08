use gfx;

use gfx::{Bind, Bundle, CommandBuffer, Encoder, Factory, IntoIndexBuffer, Rect, Resources, Slice};
use gfx::handle::{Buffer, RenderTargetView};
use gfx::traits::FactoryExt;
use imgui::{DrawList, ImDrawIdx, ImDrawVert, ImGui, Ui};

pub type RendererResult<T> = Result<T, RendererError>;

#[derive(Clone, Debug)]
pub enum RendererError {
    Update(gfx::UpdateError<usize>),
    Buffer(gfx::buffer::CreationError),
    Pipeline(gfx::PipelineStateError<String>),
    Combined(gfx::CombinedError),
}

impl From<gfx::UpdateError<usize>> for RendererError {
    fn from(e: gfx::UpdateError<usize>) -> RendererError { RendererError::Update(e) }
}

impl From<gfx::buffer::CreationError> for RendererError {
    fn from(e: gfx::buffer::CreationError) -> RendererError { RendererError::Buffer(e) }
}

impl From<gfx::PipelineStateError<String>> for RendererError {
    fn from(e: gfx::PipelineStateError<String>) -> RendererError { RendererError::Pipeline(e) }
}

impl From<gfx::CombinedError> for RendererError {
    fn from(e: gfx::CombinedError) -> RendererError { RendererError::Combined(e) }
}

gfx_defines!{
    pipeline pipe {
        vertex_buffer: gfx::VertexBuffer<ImDrawVert> = (),
        matrix: gfx::Global<[[f32; 4]; 4]> = "matrix",
        tex: gfx::TextureSampler<[f32; 4]> = "tex",
        out: gfx::BlendTarget<gfx::format::Srgba8> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
        scissor: gfx::Scissor = (),
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Shaders {
    GlSl400, // OpenGL 4.0+
    GlSl130, // OpenGL 3.0+
    GlSl110, // OpenGL 2.0+
    GlSlEs300, // OpenGL ES 3.0+
    GlSlEs100, // OpenGL ES 2.0+
}

impl Shaders {
    fn get_program_code(self) -> (&'static [u8], &'static [u8]) {
        use self::Shaders::*;
        match self {
            GlSl400 => (
                include_bytes!("shader/glsl_400.vert"),
                include_bytes!("shader/glsl_400.frag"),
            ),
            GlSl130 => (
                include_bytes!("shader/glsl_130.vert"),
                include_bytes!("shader/glsl_130.frag"),
            ),
            GlSl110 => (
                include_bytes!("shader/glsl_110.vert"),
                include_bytes!("shader/glsl_110.frag"),
            ),
            GlSlEs300 => (
                include_bytes!("shader/glsles_300.vert"),
                include_bytes!("shader/glsles_300.frag"),
            ),
            GlSlEs100 => (
                include_bytes!("shader/glsles_100.vert"),
                include_bytes!("shader/glsles_100.frag"),
            ),
        }
    }
}

pub struct Renderer<R: Resources> {
    bundle: Bundle<R, pipe::Data<R>>,
    index_buffer: Buffer<R, u16>,
}

impl<R: Resources> Renderer<R> {
    pub fn init<F: Factory<R>>(
        imgui: &mut ImGui,
        factory: &mut F,
        shaders: Shaders,
        out: RenderTargetView<R, gfx::format::Srgba8>,
    ) -> RendererResult<Renderer<R>> {
        let (vs_code, ps_code) = shaders.get_program_code();
        let pso = factory.create_pipeline_simple(
            vs_code,
            ps_code,
            pipe::new(),
        )?;
        let vertex_buffer = factory.create_buffer::<ImDrawVert>(
            256,
            gfx::buffer::Role::Vertex,
            gfx::memory::Usage::Dynamic,
            Bind::empty(),
        )?;
        let index_buffer = factory.create_buffer::<ImDrawIdx>(
            256,
            gfx::buffer::Role::Index,
            gfx::memory::Usage::Dynamic,
            Bind::empty(),
        )?;
        let (_, texture) = imgui.prepare_texture(|handle| {
            factory.create_texture_immutable_u8::<gfx::format::Srgba8>(
                gfx::texture::Kind::D2(
                    handle.width as u16,
                    handle.height as u16,
                    gfx::texture::AaMode::Single,
                ),
                &[handle.pixels],
            )
        })?;
        // TODO: set texture id in imgui
        let sampler = factory.create_sampler_linear();
        let data = pipe::Data {
            vertex_buffer: vertex_buffer,
            matrix: [
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, 0.0, 0.0],
                [0.0, 0.0, -1.0, 0.0],
                [-1.0, 1.0, 0.0, 1.0],
            ],
            tex: (texture, sampler),
            out: out,
            scissor: Rect {
                x: 0,
                y: 0,
                w: 0,
                h: 0,
            },
        };
        let slice = Slice {
            start: 0,
            end: 0,
            base_vertex: 0,
            instances: None,
            buffer: index_buffer.clone().into_index_buffer(factory),
        };
        Ok(Renderer {
            bundle: Bundle::new(slice, pso, data),
            index_buffer: index_buffer,
        })
    }
    pub fn update_render_target(&mut self, out: RenderTargetView<R, gfx::format::Srgba8>) {
        self.bundle.data.out = out;
    }
    pub fn render<'a, F: Factory<R>, C: CommandBuffer<R>>(
        &mut self,
        ui: Ui<'a>,
        factory: &mut F,
        encoder: &mut Encoder<R, C>,
    ) -> RendererResult<()> {
        let (width, height) = ui.imgui().display_size();

        if width == 0.0 || height == 0.0 {
            return Ok(());
        }
        self.bundle.data.matrix = [
            [2.0 / width as f32, 0.0, 0.0, 0.0],
            [0.0, -2.0 / height as f32, 0.0, 0.0],
            [0.0, 0.0, -1.0, 0.0],
            [-1.0, 1.0, 0.0, 1.0],
        ];

        ui.render(|ui, draw_list| {
            self.render_draw_list(ui, factory, encoder, &draw_list)
        })
    }
    fn render_draw_list<'a, F: Factory<R>, C: CommandBuffer<R>>(
        &mut self,
        ui: &'a Ui<'a>,
        factory: &mut F,
        encoder: &mut Encoder<R, C>,
        draw_list: &DrawList<'a>,
    ) -> RendererResult<()> {
        let (scale_width, scale_height) = ui.imgui().display_framebuffer_scale();

        self.bundle.slice.start = 0;
        for cmd in draw_list.cmd_buffer {
            // TODO: check cmd.texture_id

            self.upload_vertex_buffer(
                factory,
                encoder,
                draw_list.vtx_buffer,
            )?;
            self.upload_index_buffer(
                factory,
                encoder,
                draw_list.idx_buffer,
            )?;

            self.bundle.slice.end = self.bundle.slice.start + cmd.elem_count;
            self.bundle.data.scissor = Rect {
                x: (cmd.clip_rect.x * scale_width) as u16,
                y: (cmd.clip_rect.y * scale_height) as u16,
                w: ((cmd.clip_rect.z - cmd.clip_rect.x).abs() * scale_width) as u16,
                h: ((cmd.clip_rect.w - cmd.clip_rect.y).abs() * scale_height) as u16,
            };
            self.bundle.encode(encoder);
            self.bundle.slice.start = self.bundle.slice.end;
        }
        Ok(())
    }
    fn upload_vertex_buffer<F: Factory<R>, C: CommandBuffer<R>>(
        &mut self,
        factory: &mut F,
        encoder: &mut Encoder<R, C>,
        vtx_buffer: &[ImDrawVert],
    ) -> RendererResult<()> {
        if self.bundle.data.vertex_buffer.len() < vtx_buffer.len() {
            self.bundle.data.vertex_buffer = factory.create_buffer::<ImDrawVert>(
                vtx_buffer.len(),
                gfx::buffer::Role::Vertex,
                gfx::memory::Usage::Dynamic,
                Bind::empty(),
            )?;
        }
        Ok(encoder.update_buffer(
            &self.bundle.data.vertex_buffer,
            vtx_buffer,
            0,
        )?)
    }
    fn upload_index_buffer<F: Factory<R>, C: CommandBuffer<R>>(
        &mut self,
        factory: &mut F,
        encoder: &mut Encoder<R, C>,
        idx_buffer: &[ImDrawIdx],
    ) -> RendererResult<()> {
        if self.index_buffer.len() < idx_buffer.len() {
            self.index_buffer = factory.create_buffer::<ImDrawIdx>(
                idx_buffer.len(),
                gfx::buffer::Role::Index,
                gfx::memory::Usage::Dynamic,
                Bind::empty(),
            )?;
            self.bundle.slice.buffer = self.index_buffer.clone().into_index_buffer(factory);
        }
        Ok(encoder.update_buffer(&self.index_buffer, idx_buffer, 0)?)
    }
}
