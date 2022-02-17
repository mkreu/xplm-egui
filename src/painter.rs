#![allow(unsafe_code)]

use std::{collections::HashMap, borrow::Borrow};

use egui::{
    emath::Rect,
    epaint::{Mesh, Vertex},
};
use glow::{HasContext, NativeTexture};
use memoffset::offset_of;
use xplm::debugln;

use crate::{misc_util::{
    as_u8_slice, compile_shader, link_program, srgbtexture2d, texture_from_raw_id,
}, check_gl_error};

pub use glow::Context;

const VERT_SRC: &str = include_str!("shader.vert");
const FRAG_SRC: &str = include_str!("shader.frag");

/// OpenGL painter
///
/// This struct must be destroyed with [`Painter::destroy`] before dropping, to ensure OpenGL
/// objects have been properly deleted and are not leaked.
pub struct Painter {
    program: glow::Program,
    u_screen_size: glow::UniformLocation,
    u_sampler: glow::UniformLocation,
    egui_texture: Option<NativeTexture>,
    egui_texture_version: Option<u64>,
    vertex_array: glow::NativeVertexArray,
    vertex_buffer: glow::Buffer,
    element_array_buffer: glow::Buffer,

    /// Index is the same as in [`egui::TextureId::User`].
    user_textures: HashMap<u64, glow::Texture>,

    #[cfg(feature = "epi")]
    next_native_tex_id: u64, // TODO: 128-bit texture space?

    /// Stores outdated OpenGL textures that are yet to be deleted
    textures_to_destroy: Vec<glow::Texture>,

    /// Used to make sure we are destroyed correctly.
    destroyed: bool,
}

impl Painter {
    /// Create painter.
    ///
    /// Set `pp_fb_extent` to the framebuffer size to enable `sRGB` support on OpenGL ES and WebGL.
    ///
    /// Set `shader_prefix` if you want to turn on shader workaround e.g. `"#define APPLY_BRIGHTENING_GAMMA\n"`
    /// (see <https://github.com/emilk/egui/issues/794>).
    ///
    /// # Errors
    /// will return `Err` below cases
    /// * failed to compile shader
    /// * failed to create postprocess on webgl with `sRGB` support
    /// * failed to create buffer
    pub fn new(
        gl: &glow::Context,
        //pp_fb_extent: Option<[i32; 2]>,
    ) -> Result<Painter, String> {
        unsafe {
            let vert = compile_shader(gl, glow::VERTEX_SHADER, VERT_SRC)?;
            let frag = compile_shader(gl, glow::FRAGMENT_SHADER, FRAG_SRC)?;
            let program = link_program(gl, [vert, frag].iter())?;
            gl.detach_shader(program, vert);
            gl.detach_shader(program, frag);
            gl.delete_shader(vert);
            gl.delete_shader(frag);
            let u_screen_size = gl.get_uniform_location(program, "u_screen_size").unwrap();
            let u_sampler = gl.get_uniform_location(program, "u_sampler").unwrap();
            let vertex_buffer = gl.create_buffer()?;
            let element_array_buffer = gl.create_buffer()?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            let a_pos_loc = gl.get_attrib_location(program, "a_pos").unwrap();
            let a_tc_loc = gl.get_attrib_location(program, "a_tc").unwrap();
            let a_srgba_loc = gl.get_attrib_location(program, "a_srgba").unwrap();
            let mut vertex_array = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(vertex_array));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vertex_buffer));
            let stride = std::mem::size_of::<Vertex>() as i32;
            gl.vertex_attrib_pointer_f32(
                a_pos_loc,
                2,
                glow::FLOAT,
                false,
                stride,
                offset_of!(Vertex, pos) as i32,
            );
            gl.vertex_attrib_pointer_f32(
                a_tc_loc,
                2,
                glow::FLOAT,
                false,
                stride,
                offset_of!(Vertex, uv) as i32,
            );
            gl.vertex_attrib_pointer_f32(
                a_srgba_loc,
                4,
                glow::UNSIGNED_BYTE,
                false,
                stride,
                offset_of!(Vertex, color) as i32,
            );
            gl.enable_vertex_attrib_array(a_pos_loc);
            gl.enable_vertex_attrib_array(a_tc_loc);
            gl.enable_vertex_attrib_array(a_srgba_loc);

            check_gl_error(gl, "while setting up painter");

            Ok(Painter {
                program,
                u_screen_size,
                u_sampler,
                egui_texture: None,
                egui_texture_version: None,
                vertex_array,
                vertex_buffer,
                element_array_buffer,
                user_textures: Default::default(),
                textures_to_destroy: Vec::new(),
                destroyed: false,
            })
        }
    }

    pub fn upload_egui_texture(&mut self, gl: &glow::Context, font_image: &egui::FontImage) {
        self.assert_not_destroyed();

        if self.egui_texture_version == Some(font_image.version) {
            return; // No change
        }
        let gamma = 1.0;
        let pixels: Vec<u8> = font_image
            .srgba_pixels(gamma)
            .flat_map(|a| Vec::from(a.to_array()))
            .collect();

        if let Some(old_tex) = std::mem::replace(
            &mut self.egui_texture,
            Some(srgbtexture2d(
                gl,
                &pixels,
                font_image.width,
                font_image.height,
            )),
        ) {
            unsafe {
                //gl.delete_texture(texture_from_raw_id(old_tex));
                gl.delete_texture(old_tex);
            }
        }
        self.egui_texture_version = Some(font_image.version);
    }

    unsafe fn prepare_painting(
        &mut self,
        window: xplm::geometry::Rect<i32>,
        gl: &glow::Context,
        pixels_per_point: f32,
    ) -> (u32, u32) {
        xplm::draw::set_state(&xplm::draw::GraphicsState {
            fog: false,
            lighting: false,
            alpha_testing: true,
            alpha_blending: true,
            depth_testing: false,
            depth_writing: false,
            textures: 1, // TODO is this right?
        });
        check_gl_error(gl, "while setting xplm graphic options");

        gl.enable(glow::SCISSOR_TEST);
        check_gl_error(gl, "while enabling scissor test");
        // egui outputs mesh in both winding orders
        gl.disable(glow::CULL_FACE);
        check_gl_error(gl, "while disabling face culling");

        gl.blend_func_separate(
            // egui outputs colors with premultiplied alpha:
            glow::ONE,
            glow::ONE_MINUS_SRC_ALPHA,
            // Less important, but this is technically the correct alpha blend function
            // when you want to make use of the framebuffer alpha (for screenshots, compositing, etc).
            glow::ONE_MINUS_DST_ALPHA,
            glow::ONE,
        );
        check_gl_error(gl, "while setting blend functions");
        let width_in_pixels = window.right() - window.left();
        let height_in_pixels = window.top() - window.bottom();

        let width_in_points = width_in_pixels as f32 / pixels_per_point;
        let height_in_points = height_in_pixels as f32 / pixels_per_point;

        gl.viewport(window.left(), window.bottom(), width_in_pixels, height_in_pixels);
        check_gl_error(gl, "while setting viewport");
        gl.use_program(Some(self.program));
        check_gl_error(gl, "while binding program");

        gl.uniform_2_f32(Some(&self.u_screen_size), width_in_points, height_in_points);
        check_gl_error(gl, "while setting screen_size uniform");
        gl.uniform_1_i32(Some(&self.u_sampler), 0);
        check_gl_error(gl, "while setting sampler uniform");
        if let Some(tex) = self.egui_texture {
            //xplm::draw::bind_texture(0, tex);
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));
            check_gl_error(gl, "while binding texture uniform");
        }
        gl.bind_vertex_array(Some(self.vertex_array));
        check_gl_error(gl, "while binding vao");

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.element_array_buffer));
        check_gl_error(gl, "while binding element buffer");

        (width_in_pixels as u32, height_in_pixels as u32)
    }

    unsafe fn cleanup_painting(
        &mut self,
        [left, bottom, right, top]: [i32; 4],
        gl: &glow::Context,
    ) {
        gl.disable(glow::SCISSOR_TEST);
        // egui outputs mesh in both winding orders
        gl.enable(glow::CULL_FACE);

        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

        gl.viewport(left, bottom, right - left, top - bottom);
        gl.use_program(None);

        gl.bind_vertex_array(None);

        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

        check_gl_error(gl, "while cleaning up after painting");
    }

    pub fn paint_meshes(
        &mut self,
        gl: &glow::Context,
        inner_size: xplm::geometry::Rect<i32>,
        viewport: [i32; 4],
        pixels_per_point: f32,
        clipped_meshes: &[egui::ClippedMesh],
    ) {
        self.assert_not_destroyed();

        let size_in_pixels = unsafe { self.prepare_painting(inner_size, gl, pixels_per_point) };
        debugln!("{inner_size:?},{size_in_pixels:?}");
        for egui::ClippedMesh(clip_rect, mesh) in clipped_meshes {
            debugln!("{clip_rect:?}");

            self.paint_mesh(gl, size_in_pixels, &inner_size, &mesh);
        }
        check_gl_error(gl, "while painting");
        unsafe {
            self.cleanup_painting(viewport, gl);
        }
    }

    fn paint_mesh(
        &mut self,
        gl: &glow::Context,
        size_in_pixels: (u32, u32),
        clip_rect: &xplm::geometry::Rect<i32>,
        mesh: &Mesh,
    ) {
        debug_assert!(mesh.is_valid());
        //TODO support user textures
        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                as_u8_slice(mesh.vertices.as_slice()),
                glow::STREAM_DRAW,
            );

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.element_array_buffer));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                as_u8_slice(mesh.indices.as_slice()),
                glow::STREAM_DRAW,
            );
        }
        // Transform clip rect to physical pixels:
        let clip_min_x = clip_rect.left();
        let clip_min_y = clip_rect.bottom();
        let clip_max_x = clip_rect.right();
        let clip_max_y = clip_rect.top();

        unsafe {
            gl.scissor(
                clip_min_x,
                clip_min_y,
                clip_max_x - clip_min_x,
                clip_max_y - clip_min_y,
            );
            gl.draw_elements(
                glow::TRIANGLES,
                mesh.indices.len() as i32,
                glow::UNSIGNED_INT,
                0,
            );
        }
    }

    pub fn free_texture(&mut self, tex_id: u64) {
        self.user_textures.remove(&tex_id);
    }

    unsafe fn destroy_gl(&self, gl: &glow::Context) {
        gl.delete_program(self.program);
        if let Some(tex) = self.egui_texture {
            //gl.delete_texture(texture_from_raw_id(tex));
            gl.delete_texture(tex);
        }
        for tex in self.user_textures.values() {
            gl.delete_texture(*tex);
        }
        gl.delete_buffer(self.vertex_buffer);
        gl.delete_buffer(self.element_array_buffer);
        for t in &self.textures_to_destroy {
            gl.delete_texture(*t);
        }
    }

    /// This function must be called before Painter is dropped, as Painter has some OpenGL objects
    /// that should be deleted.

    pub fn destroy(&mut self, gl: &glow::Context) {
        if !self.destroyed {
            unsafe {
                self.destroy_gl(gl);
            }
            self.destroyed = true;
        }
    }

    fn assert_not_destroyed(&self) {
        assert!(!self.destroyed, "the egui glow has already been destroyed!");
    }
}

impl Drop for Painter {
    fn drop(&mut self) {
        if !self.destroyed {
            eprintln!(
                "You forgot to call destroy() on the egui glow painter. Resources will leak!"
            );
        }
    }
}
