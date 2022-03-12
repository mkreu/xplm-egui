#![allow(unsafe_code)]
use glow::{HasContext, NativeTexture};
use xplm::debugln;

pub(crate) fn srgbtexture2d(gl: &glow::Context, data: &[u8], w: usize, h: usize) -> NativeTexture {
    assert_eq!(data.len(), w * h * 4);
    assert!(w >= 1);
    assert!(h >= 1);
    unsafe {
        //let tex = xplm::draw::generate_texture_number();
        let tex = gl.create_texture().expect("failed to create a texture");
        check_gl_error(gl, "after creating texture via xplm");
        gl.bind_texture(glow::TEXTURE_2D, Some(tex));
        //xplm::draw::bind_texture(tex, 0);
        check_gl_error(gl, "after binding texture via xplm");

        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MAG_FILTER,
            glow::LINEAR as i32,
        );
        check_gl_error(gl, "after tex parameter");
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_MIN_FILTER,
            glow::LINEAR as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_S,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_parameter_i32(
            glow::TEXTURE_2D,
            glow::TEXTURE_WRAP_T,
            glow::CLAMP_TO_EDGE as i32,
        );
        gl.tex_storage_2d(glow::TEXTURE_2D, 1, glow::SRGB8_ALPHA8, w as i32, h as i32);
        check_gl_error(gl, "after tex storage");
        gl.tex_sub_image_2d(
            glow::TEXTURE_2D,
            0,
            0,
            0,
            w as i32,
            h as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(data),
        );
        check_gl_error(gl, "after tex sub image");
        gl.bind_texture(glow::TEXTURE_2D, None);
        tex
    }
}

pub(crate) unsafe fn as_u8_slice<T>(s: &[T]) -> &[u8] {
    std::slice::from_raw_parts(s.as_ptr().cast::<u8>(), s.len() * std::mem::size_of::<T>())
}

pub fn check_gl_error(gl: &glow::Context, description: impl std::fmt::Display) {
    loop {
        let err = unsafe { gl.get_error() };
        match err {
            glow::NO_ERROR => break,
            _ => debugln!("gl error {description}: {err:x}"),
        }
    }
}

pub(crate) unsafe fn compile_shader(
    gl: &glow::Context,
    shader_type: u32,
    source: &str,
) -> Result<glow::Shader, String> {
    let shader = gl.create_shader(shader_type)?;

    gl.shader_source(shader, source);

    gl.compile_shader(shader);

    if gl.get_shader_compile_status(shader) {
        Ok(shader)
    } else {
        Err(gl.get_shader_info_log(shader))
    }
}

pub(crate) unsafe fn link_program<'a, T: IntoIterator<Item = &'a glow::Shader>>(
    gl: &glow::Context,
    shaders: T,
) -> Result<glow::Program, String> {
    let program = gl.create_program()?;

    for shader in shaders {
        gl.attach_shader(program, *shader);
    }

    gl.link_program(program);

    if gl.get_program_link_status(program) {
        Ok(program)
    } else {
        Err(gl.get_program_info_log(program))
    }
}
