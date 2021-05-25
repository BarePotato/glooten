// use std::collections::HashMap;

// use anyhow::Result;

// use unicode_width::UnicodeWidthChar;

use std::{
    collections::HashMap,
    ffi::c_void,
    mem::{size_of, size_of_val},
    ptr::null,
};

use freetype::{face::LoadFlag, Library};

// use unicode_normalization::UnicodeNormalization;

use glutin::{
    dpi::LogicalSize,
    event::{Event as GlutinEvent, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as GlutinWindow, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

mod shader;

const WIN_SIZE: (f32, f32) = (800.0, 600.0);

#[derive(Debug)]
struct Vec2i {
    x: i32,
    y: i32,
}

#[derive(Debug)]
struct Character {
    tex_id: u32,
    size: Vec2i,
    bearing: Vec2i,
    advance: i64,
}

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("GL Font Things, Oof!")
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WIN_SIZE.0, WIN_SIZE.1));

    let windowed_context =
        make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

    gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

    let font_lib = Library::init().unwrap();
    let font_face = font_lib
        .new_face("/usr/share/fonts/nerd-fonts-complete/OTF/Fira Code Regular Nerd Font Complete.otf", 0)
        .unwrap();
    font_face.set_pixel_sizes(0, 24).unwrap();
    // font_face.load_char('A' as usize, LoadFlag::RENDER).unwrap();
    // let glyph = font_face.glyph();

    let mut characters: HashMap<char, Character> = HashMap::new();

    unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1) };

    for c in 0..128 {
        font_face.load_char(c, LoadFlag::RENDER).unwrap();

        let texture = u32::default();

        unsafe {
            gl::GenTextures(1, texture as *mut _);
            gl::BindTexture(gl::TEXTURE_2D, texture);
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RED as i32,
                font_face.glyph().bitmap().width(),
                font_face.glyph().bitmap().rows(),
                0,
                gl::RED,
                gl::UNSIGNED_BYTE,
                font_face.glyph().bitmap().buffer().as_ptr() as *const std::ffi::c_void,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
        };

        let character = Character {
            tex_id: texture,
            size: Vec2i { x: font_face.glyph().bitmap().width(), y: font_face.glyph().bitmap().rows() },
            bearing: Vec2i { x: font_face.glyph().bitmap_left(), y: font_face.glyph().bitmap_top() },
            advance: font_face.glyph().advance().x,
        };

        characters.insert(c as u8 as char, character);
    }

    let (shader_v, shader_f) = (include_str!("../shader/shader.v.glsl"), include_str!("../shader/shader.f.glsl"));

    let shader_program = shader::CharShaderProgram::new(shader_v, shader_f);

    unsafe {
        gl::Enable(gl::BLEND);
        gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    };

    let (vao, vbo) = (u32::default(), u32::default());
    unsafe {
        gl::GenVertexArrays(1, vao as *mut u32);
        gl::GenBuffers(1, vbo as *mut u32);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of::<f32>() as isize * 6 * 4, null(), gl::DYNAMIC_DRAW);
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(0, 4, gl::FLOAT, gl::FALSE, 4 * size_of::<f32>() as i32, 0 as *const c_void);
        gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        gl::BindVertexArray(0);
    };

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            GlutinEvent::LoopDestroyed => return,
            GlutinEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // WindowEvent::Focused(focused) => {}
                // WindowEvent::ReceivedCharacter(c) => {}
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

                draw_text(
                    shader_program.id,
                    "Some text",
                    50.0,
                    50.0,
                    1.0,
                    Color::rgb(255, 0, 0, 255).as_gl(),
                    vao,
                    vbo,
                    &characters,
                );

                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}

fn draw_text(
    shader_program: u32,
    text: &str,
    mut x: f32,
    y: f32,
    scale: f32,
    color: glColor,
    vao: u32,
    vbo: u32,
    characters: &HashMap<char, Character>,
) {
    unsafe {
        gl::UseProgram(shader_program);

        gl::Viewport(0, 0, WIN_SIZE.0 as gl::types::GLsizei, WIN_SIZE.1 as gl::types::GLsizei);
        gl::Uniform4f(
            gl::GetUniformLocation(shader_program, "projection".as_ptr() as *const i8),
            0.0,
            0.0,
            WIN_SIZE.0,
            WIN_SIZE.1,
        );

        gl::Uniform3f(
            gl::GetUniformLocation(shader_program, "textColor".as_ptr() as *const i8),
            color.r,
            color.g,
            color.b,
        );
        gl::ActiveTexture(gl::TEXTURE0);
        gl::BindVertexArray(vao);
    };

    for c in text.chars() {
        let character = characters.get(&c).unwrap();

        let xpos = x as f32 + character.bearing.x as f32 * scale;
        let ypos = y as f32 + (character.size.y - character.bearing.y) as f32 * scale;

        let w = character.size.x as f32 * scale;
        let h = character.size.y as f32 * scale;

        let verts: [[f32; 4]; 6] = [
            [xpos, ypos + h, 0.0, 0.0],
            [xpos, ypos, 0.0, 1.0],
            [xpos + w, ypos, 1.0, 1.0],
            [xpos, ypos + h, 0.0, 0.0],
            [xpos + w, ypos, 1.0, 1.0],
            [xpos + w, ypos + h, 1.0, 0.0],
        ];

        unsafe {
            gl::BindTexture(gl::TEXTURE_2D, character.tex_id);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, size_of_val(&verts) as isize, verts.as_ptr() as *const c_void);
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
            gl::DrawArrays(gl::TRIANGLES, 0, 6);
        };

        x += (character.advance >> 6) as f32 * scale;

        unsafe {
            gl::BindVertexArray(0);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        };
    }
}

fn clear_buffer(color: &Color) {
    let color = color.as_gl();

    unsafe {
        gl::ClearColor(color.r, color.g, color.b, color.a);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };
}

fn make_current_context(
    windowed_context: ContextWrapper<NotCurrent, GlutinWindow>,
) -> ContextWrapper<PossiblyCurrent, GlutinWindow> {
    unsafe { windowed_context.make_current().unwrap() }
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
