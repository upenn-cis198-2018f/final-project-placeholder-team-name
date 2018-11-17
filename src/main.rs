extern crate glutin;
extern crate gl;

mod graphics;

use glutin::*;

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("music visualizer")
        .with_dimensions(dpi::LogicalSize::new(1000.0, 700.0));
    // TODO see past project for proper setup on a mac
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        .with_gl_profile(GlProfile::Core);
    let mut display = GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        display.make_current().unwrap();
    }

    gl::load_with(
        |symbol| display.get_proc_address(symbol) as *const _);

    let mut keep_running = true;
    while keep_running {
        events_loop.poll_events(|event| {
            let keep_open = handle_event(&mut display, event);
            if !keep_open && keep_running {
                keep_running = false
            };
        });
        // This hack is required to fix a bug on OS Mojave
        display.resize(dpi::PhysicalSize::new(1000.0, 700.0));

        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        display.swap_buffers().unwrap();
        //std::thread::sleep(std::time::Duration::from_millis(17));
    }
}

fn handle_event(display: &mut GlWindow, event: Event) -> bool {
    match event {
        Event::WindowEvent{event: win_event, ..} => {
            match win_event {
                WindowEvent::CloseRequested => return false,
                WindowEvent::Resized(logical_size) => {
                    // when the window resizes, we must resize the context
                    let dpi = display.get_hidpi_factor();
                    display.resize(logical_size.to_physical(dpi));
                },
                _ => {
                    // TODO: forward mouse and key to ImGui backend
                }
            }
        },
        _ => ()
    };
    true
}
