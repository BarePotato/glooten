// use std::collections::HashMap;

// use anyhow::Result;

// use unicode_width::UnicodeWidthChar;

use glutin::{
    event::{Event as GlutinEvent, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window as GlutinWindow, WindowBuilder},
    ContextBuilder, ContextWrapper, NotCurrent, PossiblyCurrent,
};

mod gl {
    include!(concat!(env!("OUT_DIR"), "/gl_bindings.rs"));
}

fn main() {
    let event_loop = EventLoop::new();

    let window_builder = WindowBuilder::new().with_title("GL Font Things, Oof!");

    let windowed_context =
        make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

    gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

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
                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
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
