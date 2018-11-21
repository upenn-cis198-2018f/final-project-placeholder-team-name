extern crate glutin;
extern crate gl;
extern crate cgmath;

mod graphics;

use graphics::*;
use glutin::*;

use cgmath::*;

// TODO: pass in the elasped time in secs as a double, to use
// for animation. Also pass in deltaT.
// TODO: Also, factor this out and make it a struct with an update
// method so that we can store whatever state we need.
// TODO: later, also pass in a struct containing the relevant audio data
fn update() -> Canvas {
    let mut canvas = Canvas::new();
    canvas.set_background_color(Vec4::new(0f32, 0f32, 0f32, 1f32));
    /*
    canvas.draw_triangle(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(10f32, 10f32, 0f32, 1f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
    */
    /*
    canvas.draw_pgram(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(0f32, 20f32, 0f32, 1f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
    */
    canvas.draw_ppiped(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(0f32, 10f32, 0f32, 1f32),
        vec4(0f32, 0f32, 10f32, 1f32),
        vec4(0f32, 0f32, 1f32, 1f32)
    );
    canvas
}

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("music visualizer")
        .with_dimensions(dpi::LogicalSize::new(1000.0, 700.0));
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 1)))
        .with_gl_profile(GlProfile::Core);
    let mut display = GlWindow::new(window, context, &events_loop).unwrap();

    unsafe {
        display.make_current().unwrap();
    }

    gl::load_with(
        |symbol| display.get_proc_address(symbol) as *const _);

    let mut g_state = GraphicsState::new();

    let mut keep_running = true;
    while keep_running {
        events_loop.poll_events(|event| {
            let keep_open = handle_event(&mut display, event);
            if !keep_open && keep_running {
                keep_running = false
            };
        });
        // This hack is required to fix a bug on OS Mojave
        // It resizes the window to its current size.
        // https://github.com/tomaka/glutin/issues/1069
        let dpi = display.get_hidpi_factor();
        let display_size = display.get_inner_size().unwrap();
        display.resize(display_size.to_physical(dpi));

        let canvas = update();
        g_state.draw_frame(&canvas);

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
