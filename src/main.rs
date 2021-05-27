use std::{
    ffi::c_void,
    mem::{size_of, size_of_val},
    ptr,
};

use glutin::{
    dpi::LogicalSize,
    event::{Event as GlutinEvent, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as GlutinWindow, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

use nalgebra_glm as glm;

pub(crate) mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

use gl::types::{GLsizei, GLuint};

mod shader;

const WIN_SIZE: (f32, f32) = (800.0, 600.0);

fn main() {
    let event_loop = EventLoop::new();
    let window_builder = WindowBuilder::new()
        .with_title("GL Font Things, Oof!")
        .with_resizable(false)
        .with_inner_size(LogicalSize::new(WIN_SIZE.0, WIN_SIZE.1));

    let windowed_context =
        make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

    gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

    // we were forcing 1 color, red
    // unsafe { gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1) };

    // unsafe {
    //     gl::Enable(gl::BLEND);
    //     gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
    // };

    // let projection = glm::ortho(0.0, WIN_SIZE.0, 0.0, WIN_SIZE.1, 0.0, -10_000.0);
    // let identity = glm::Mat4::identity();

    let vertices = [-0.5, -0.5, 0.0, 0.5, -0.5, 0.0, 0.0, 0.5, 0.0f32];

    let (mut vao, mut vbo) = (0, 0);
    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);
        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
        gl::BufferData(gl::ARRAY_BUFFER, size_of::<[f32; 9]>() as isize, vertices.as_ptr().cast(), gl::STATIC_DRAW);
    }

    let (shader_v, shader_f) = (include_str!("../shader/tri.v.glsl"), include_str!("../shader/tri.f.glsl"));
    let shader_program = shader::TriShaderProgram::new(shader_v, shader_f);

    unsafe {
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 3 * size_of::<f32>() as i32, 0 as *const _);
        gl::EnableVertexAttribArray(0);
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
                    gl::UseProgram(shader_program.id);

                    gl::Viewport(0, 0, WIN_SIZE.0 as i32, WIN_SIZE.1 as i32);

                    gl::DrawArrays(gl::TRIANGLES, 0, 3);
                }

                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
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
