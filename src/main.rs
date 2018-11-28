extern crate glutin;
extern crate gl;
extern crate cgmath;

mod graphics;
mod visualizer;

use visualizer::*;
use graphics::*;
use glutin::*;
use std::time;

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("music visualizer")
        .with_dimensions(dpi::LogicalSize::new(1000.0, 700.0));
    let context = ContextBuilder::new()
        .with_vsync(true)
        .with_gl(GlRequest::Specific(glutin::Api::OpenGl, (4, 1)))
        .with_gl_profile(GlProfile::Core);

    // attempt to create a window
    let display_result = GlWindow::new(window, context, &events_loop);
    let mut display_opt = match display_result {
        Ok(display) => Some(display),
        Err(err) => {
            println!("{:?}", err);
            None
        }
    };

    // if we have a window, setup OpenGL
    let mut g_state = GraphicsState::new();
    if let Some(ref display) = display_opt {
        unsafe {
            display.make_current().unwrap_or_else(|err| {
                println!("Context creation error:\n{:?}", err);
            });
        }
        gl::load_with(
            |symbol| display.get_proc_address(symbol) as *const _);
        g_state.setup_opengl();
    }

    let program_start = time::Instant::now();
    let mut keep_running = true;
    let mut previous_tick = program_start;
    let frame_period: f64 = 1.0 / 60.0; // in secs
    let frame_duration = time::Duration::from_millis(
        (frame_period * 1000.0) as u64);
    let mut visualizer = Visualizer::new();
    while keep_running {
        // sleep until the start of the next frame
        let current_time = time::Instant::now();
        let sleep_duration_opt = frame_duration.checked_sub(
            current_time.duration_since(previous_tick));
        if let Some(sleep_duration) = sleep_duration_opt {
            std::thread::sleep(sleep_duration);
        };
        previous_tick = current_time;

        // if we have a window, poll for events and resize to fit the window
        if let Some(ref mut display) = display_opt {
            events_loop.poll_events(|event| {
                let keep_open = handle_event(display, event);
                if !keep_open && keep_running {
                    keep_running = false
                };
            });
            // This hack is required to fix a bug on OS Mojave
            // It resizes the window to its current size.
            // https://github.com/tomaka/glutin/issues/1069
            let dpi = display.get_hidpi_factor();
            if let Some(display_size) = display.get_inner_size() {
                let physical_size = display_size.to_physical(dpi);
                display.resize(physical_size);
                g_state.update_framebuffer_size(physical_size.width, physical_size.height);
            }
        }

        let program_duration = time::Instant::now().duration_since(
            program_start);
        let program_duration_secs = (program_duration.as_secs()  as f32) + 
            (program_duration.subsec_millis() as f32) / 1000.0;
        let canvas = visualizer.update(
            frame_period as f32, program_duration_secs);

        // if we have a window, render the canvas to it
        if let Some(ref display) = display_opt {
            g_state.draw_frame(&canvas);
            display.swap_buffers().unwrap_or_else(|err| {
                println!("Error on swap buffers:\n{:?}", err);
            });
        }
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
