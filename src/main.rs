extern crate glutin;
extern crate gl;
extern crate cgmath;

mod graphics;

use graphics::*;
use glutin::*;
use std::time;
use std::f32::consts::*;

use cgmath::*;

fn draw_triangle(canvas: &mut Canvas) {
    canvas.draw_triangle(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(10f32, 10f32, 0f32, 1f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
}

fn draw_pgram(canvas: &mut Canvas) {
    canvas.draw_pgram(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(0f32, 10f32, 0f32, 1f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
}

fn draw_ppiped(canvas: &mut Canvas) {
    canvas.draw_ppiped(
        vec4(0f32, 0f32, 0f32, 1f32),
        vec4(10f32, 0f32, 0f32, 1f32),
        vec4(0f32, 10f32, 0f32, 1f32),
        vec4(0f32, 0f32, 10f32, 1f32),
        vec4(0f32, 0f32, 1f32, 1f32)
    );
}

// TODO: pass in the elasped time in secs as a double, to use
// for animation. Also pass in deltaT.
// TODO: Also, factor this out and make it a struct with an update
// method so that we can store whatever state we need.
// TODO: later, also pass in a struct containing the relevant audio data
fn update(delta_secs: f32, time_secs: f32) -> Canvas {
    let mut canvas = Canvas::new();

    let ca = vec4(0f32, 0f32, 0f32, 1f32);
    let cb = vec4(0f32, 1f32, 0f32, 1f32);
    let anim_factor = ((2f32 * PI * time_secs / 3.0f32).sin() + 1f32) / 2f32;
    let anim_color = ca + (1f32 - anim_factor) * cb;
    //canvas.set_background_color(anim_color);
    
    //draw_triangle(&mut canvas);
    draw_pgram(&mut canvas);
    //draw_ppiped(&mut canvas);
    
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

    let program_start = time::Instant::now();
    let mut keep_running = true;
    let mut previous_tick = program_start;
    let frame_period: f64 = 1.0 / 60.0; // in secs
    let frame_duration = time::Duration::from_millis(
        (frame_period * 1000.0) as u64);
    while keep_running {
        // sleep until the start of the next frame
        let current_time = time::Instant::now();
        let sleep_duration_opt = frame_duration.checked_sub(
            current_time.duration_since(previous_tick));
        if let Some(sleep_duration) = sleep_duration_opt {
            std::thread::sleep(sleep_duration);
        };
        previous_tick = current_time;

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
        let physical_size = display_size.to_physical(dpi);
        display.resize(physical_size);
        g_state.update_framebuffer_size(physical_size.width, physical_size.height);

        let program_duration = time::Instant::now().duration_since(
            program_start);
        let program_duration_secs = (program_duration.as_secs()  as f32) + 
            (program_duration.subsec_millis() as f32) / 1000.0;
        let canvas = update(
            frame_period as f32, program_duration_secs);
        g_state.draw_frame(&canvas);

        display.swap_buffers().unwrap();
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
