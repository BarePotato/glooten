use anyhow::Result;

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

// Carries the pigeons
struct Display {
    window: Window,
    rasterizer: Rasterizer,
    renderer: Renderer,
}

impl Display {
    fn new(event_loop: &EventLoop<UserEvent>) -> Self {
        let window = Window::new(event_loop).unwrap();
        let rasterizer = Rasterizer {};
        let renderer = Renderer {};

        Display { window, rasterizer, renderer }
    }
}

// Just a box for dots
struct Window {
    windowed_context: ContextWrapper<PossiblyCurrent, GlutinWindow>,
}

impl Window {
    fn new(event_loop: &EventLoop<UserEvent>) -> Result<Self> {
        let window_builder = WindowBuilder::new().with_title("TeeWeeMayBee");

        let windowed_context =
            make_current_context(ContextBuilder::new().build_windowed(window_builder, &event_loop).unwrap());

        gl::load_with(|c_ptr| windowed_context.get_proc_address(c_ptr) as *const _);

        Ok(Self { windowed_context })
    }
}

// Font/Glyph cache
struct Rasterizer {}

// Does the thing
struct Renderer {}

// Pigeons, maybe
enum UserEvent {}

fn main() {
    let event_loop = EventLoop::<UserEvent>::with_user_event();

    let display = Display::new(&event_loop);

    event_loop.run(move |event, _window_target, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            GlutinEvent::LoopDestroyed => return,
            GlutinEvent::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(physical_size) => display.window.windowed_context.resize(physical_size),
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                // WindowEvent::Focused(focused) => {}
                // WindowEvent::ReceivedCharacter(c) => {}
                WindowEvent::KeyboardInput { input, .. } => match input.virtual_keycode.unwrap() {
                    VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                    _ => {}
                },
                _ => {}
            },
            GlutinEvent::RedrawRequested(_) => {
                clear_buffer();
                display.window.windowed_context.swap_buffers().unwrap();
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
