use std::{
    collections::HashMap,
    mem::{size_of, size_of_val},
    ptr::null,
};

use glutin::{
    dpi::{LogicalSize, PhysicalSize},
    event::{Event as GlutinEvent, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as GlutinWindow, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

use freetype::{face::LoadFlag, Library};

use nalgebra_glm as glm;

pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod shader;
use shader::CharShaderProgram;

const WIN_SIZE: (f32, f32) = (800.0, 600.0);

type CharMap = HashMap<char, Char>;

#[derive(Debug)]
struct Char {
    tex_id: u32,
    size: glm::IVec2,
    bearing: glm::IVec2,
    advance: i64,
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("GL Font Things, Oof!")
        .with_resizable(true)
        // .with_transparent(true)
        .with_inner_size(LogicalSize::new(WIN_SIZE.0, WIN_SIZE.1));

    let windowed_context =
        make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

    gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

    let lib = Library::init().unwrap();
    let face =
        lib.new_face("/usr/share/fonts/nerd-fonts-complete/OTF/Fira Code Regular Nerd Font Complete.otf", 0).unwrap();
    // face.set_char_size(40 * 64, 0, 50, 0).unwrap();
    face.set_pixel_sizes(0, 16).unwrap();

    let mut chars: HashMap<char, Char> = HashMap::new();

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

        gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1)
    };

    for c in 0..128u8 {
        face.load_char(c as usize, LoadFlag::RENDER).unwrap();
        let glyph = face.glyph();
        let bitmap = glyph.bitmap();

        let mut tex_id = 0u32;
        unsafe {
            gl::GenTextures(1, &mut tex_id);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as gl::types::GLint,
                bitmap.width(),
                bitmap.rows(),
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                bitmap.buffer().as_ptr().cast(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);

            chars.insert(
                c as char,
                Char {
                    tex_id,
                    size: glm::IVec2::new(bitmap.width(), bitmap.rows()),
                    bearing: glm::IVec2::new(glyph.bitmap_left(), glyph.bitmap_top()),
                    advance: glyph.advance().x,
                },
            );
        }
    }

    unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) };

    // let identity = glm::Mat4::identity();

    let (mut vao, mut vbo, mut _ebo) = (0, 0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of::<f32>() as isize * 6 * 4, null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * size_of::<f32>() as i32, 0 as *const _);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    }

    let (shader_v, shader_f) = (include_str!("../shader/char.v.glsl"), include_str!("../shader/char.f.glsl"));
    let shader_program = shader::CharShaderProgram::new(shader_v, shader_f, vao, vbo);

    let proj = glm::ortho(0.0, WIN_SIZE.0, 0.0, WIN_SIZE.1, 10.0, -10.0);

    unsafe {
        gl::UseProgram(shader_program.id);
        gl::UniformMatrix4fv(shader_program.projection, 1, gl::FALSE, glm::value_ptr(&proj).as_ptr());
    };

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            GlutinEvent::LoopDestroyed => return,
            GlutinEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput { input, .. } if input.virtual_keycode.is_some() => {
                    match input.virtual_keycode.unwrap() {
                        VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                        _ => {}
                    }
                }
                _ => {}
            },
            GlutinEvent::RedrawRequested(_) => {
                clear_buffer(&Color::eight);

                unsafe {
                    gl::Viewport(0, 0, WIN_SIZE.0 as i32, WIN_SIZE.1 as i32);

                    let why = 4.0;

                    draw_text(
                        &shader_program,
                        &chars,
                        "The cow goes meh.",
                        0.0,
                        0.0,
                        1.0,
                        &Color::rgb(200, 200, 200, 255),
                    );

                    draw_text(
                        &shader_program,
                        &chars,
                        "Thanks g0rg! LUL",
                        0.0,
                        16.0,
                        1.0,
                        &Color::rgb(200, 30, 30, 255),
                    );

                    draw_text(
                        &shader_program,
                        &chars,
                        "The answer is in #rust",
                        0.0,
                        16.0 * 2.0,
                        1.0,
                        &Color::rgb(20, 90, 130, 255),
                    );
                    draw_text(
                        &shader_program,
                        &chars,
                        "Missed a gl::UseProgram(id)",
                        20.0,
                        16.0 * 3.0,
                        1.0,
                        &Color::rgb(20, 90, 130, 255),
                    );
                }

                windowed_context.swap_buffers().unwrap();
                // windowed_context.window().request_redraw();
            }
            _ => {}
        }
    });
}

fn draw_text(program: &CharShaderProgram, chars: &CharMap, text: &str, mut x: f32, y: f32, scale: f32, color: &Color) {
    let color = color.as_gl();

    unsafe {
        gl::UseProgram(program.id);

        let kuler = gl::GetUniformLocation(program.id, b"kuler\0".as_ptr().cast());
        gl::Uniform3f(kuler, color.r, color.g, color.b);
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(program.vao);
    }

    for c in text.chars() {
        let c = chars.get(&c).unwrap();

        let posx = (x + c.bearing.x as f32) * scale;
        let posy = (y - (c.size.y - c.bearing.y) as f32) * scale;

        let w = c.size.x as f32 * scale;
        let h = c.size.y as f32 * scale;

        let verts = [
            [posx, posy + h, 0.0, 0.0f32],
            [posx, posy, 0.0, 1.0],
            [posx + w, posy, 1.0, 1.0],
            [posx, posy + h, 0.0, 0.0],
            [posx + w, posy, 1.0, 1.0],
            [posx + w, posy + h, 1.0, 0.0],
        ];

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, c.tex_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, program.vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, size_of_val(&verts) as isize, verts.as_ptr() as *const _);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);

            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        }

        x += (c.advance >> 6) as f32 * scale;
    }

    unsafe {
        gl::BindVertexArray(0);
        gl::BindTexture(gl::TEXTURE_2D, 0);
    }
}

fn make_current_context(
    windowed_context: ContextWrapper<NotCurrent, GlutinWindow>,
) -> ContextWrapper<PossiblyCurrent, GlutinWindow> {
    unsafe { windowed_context.make_current().unwrap() }
}

fn clear_buffer(color: &Color) {
    let color = color.as_gl();

    unsafe {
        gl::ClearColor(color.r, color.g, color.b, color.a);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };
}

#[allow(dead_code, non_camel_case_types)]
enum Color {
    rgb(u8, u8, u8, u8),
    gl(f32, f32, f32, f32),
    eight,
}

impl Color {
    fn as_gl(&self) -> glColor {
        let (r, g, b, a) = match self {
            Color::rgb(r, g, b, a) => (*r as f32 / 255.0, *g as f32 / 255.0, *b as f32 / 255.0, *a as f32 / 255.0),
            Color::gl(r, g, b, a) => (*r, *g, *b, *a),
            Color::eight => {
                let e: f32 = 8.0 / 255.0;
                (e, e, e, 1.0)
            }
        };

        glColor { r, g, b, a }
    }
}

#[allow(non_camel_case_types)]
struct glColor {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}
