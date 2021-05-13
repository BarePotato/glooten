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

const EIGHT: f32 = 8.0 / 255.0;

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
                clear_buffer();
                windowed_context.swap_buffers().unwrap();
            }
            _ => {}
        }
    });
}

fn clear_buffer() {
    unsafe {
        gl::ClearColor(EIGHT, EIGHT, EIGHT, 1.0);
        gl::Clear(gl::COLOR_BUFFER_BIT);
    };
}

fn make_current_context(
    windowed_context: ContextWrapper<NotCurrent, GlutinWindow>,
) -> ContextWrapper<PossiblyCurrent, GlutinWindow> {
    unsafe { windowed_context.make_current().unwrap() }
}
