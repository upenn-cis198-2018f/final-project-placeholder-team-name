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
        vec3(0f32, 0f32, 0f32),
        vec3(10f32, 0f32, 0f32),
        vec3(10f32, 10f32, 0f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
}

fn draw_pgram(canvas: &mut Canvas) {
    canvas.draw_pgram(
        vec3(0f32, 0f32, 0f32),
        vec3(5f32, 0f32, 0f32),
        vec3(0f32, 5f32, 0f32),
        vec4(0f32, 1f32, 0f32, 1f32)
    );
    canvas.draw_triangle(
        vec3(0f32, 0f32, 0f32),
        vec3(5f32, 0f32, 0f32),
        vec3(5f32, 5f32, 0f32),
        vec4(1f32, 0f32, 0f32, 1f32)
        );
    canvas.draw_triangle(
        vec3(-5f32, 0f32, 0f32),
        vec3(0f32, 0f32, 0f32),
        vec3(0f32, 5f32, 0f32),
        vec4(0f32, 0f32, 1f32, 1f32)
        );
}

fn draw_ppiped(canvas: &mut Canvas) {
    let x = 20f32;
    canvas.draw_ppiped(
        vec3(0f32, 0f32, 0f32),
        vec3(x, 0f32, 0f32),
        vec3(0f32, x, 0f32),
        vec3(0f32, 0f32, x),
        vec4(0f32, 0f32, 1f32, 1f32)
    );
}

fn draw_ground(canvas: &mut Canvas) {
    let half_w = 50f32;
    let h = 10f32;
    canvas.draw_ppiped(
        vec3(-half_w, -h, -half_w),
        vec3(half_w * 2f32, 0f32, 0f32),
        vec3(0f32, h, 0f32),
        vec3(0f32, 0f32, half_w * 2f32),
        vec4(0f32, 0.5f32, 0f32, 1f32)
    );
}

fn draw_surf(canvas: &mut Canvas) {
    canvas.draw_surface(100, 100, |sx, sy, nx, ny| {
        let x = 100f32 * nx - 50f32;
        let z = 100f32 * ny - 50f32;
        let y = 5f32 * (10f32 * 2f32 * PI *  x / 100f32).sin();
        let p = vec3(x, y, -z);
        let c = vec4(1f32, 0f32, 0f32, 1f32);
        (p, c)
    });
}

fn draw_sphere(canvas: &mut Canvas) {
    let r = 10f32;
    canvas.draw_surface(10, 10, |sx, sy, nx, ny| {
        let angle_vert = nx * PI;
        let angle_hor = ny * 2f32 * PI;
        let y = angle_vert.cos() * r;
        let x = r * angle_vert.sin() * angle_hor.cos();
        let z = r * angle_vert.sin() * angle_hor.sin();
        let p = vec3(x, y, z);
        let c = vec4(1f32, 0f32, 0f32, 1f32);
        (p, c)
    });
}

fn map(val: f32, cur_min: f32, cur_max: f32) -> f32 {
    (val - cur_min) / (cur_max - cur_min)
}

fn lerp(factor: f32, min: f32, max: f32) -> f32 {
    min + factor * (max - min)
}

// TODO: Also, factor this out and make it a struct with an update
// method so that we can store whatever state we need.
// TODO: later, also pass in a struct containing the relevant audio data
fn update(delta_secs: f32, time_secs: f32) -> Canvas {
    let mut canvas = Canvas::new();
    let anim_factor = map((2f32 * PI * time_secs / 5.0f32).sin(), -1f32, 1f32);

    //canvas.set_camera(vec3(0f32, 0f32, 50f32),
        //vec3(0f32, 0f32, 0f32), vec3(0f32, 1f32, 0f32));
    canvas.set_camera(vec3(0f32, 100f32, 100f32),
        vec3(0f32, 0f32, 0f32), vec3(0f32, 1f32, 0f32));
    //let cx = lerp(anim_factor, 0f32, -100f32);
    //let l_pos = vec3(0f32, 20f32, cx);
    let l_pos = 100f32 * vec3(1f32, 1f32, -1f32);
    canvas.set_light_position(l_pos);

    //draw_triangle(&mut canvas);
    //draw_pgram(&mut canvas);
    //draw_ppiped(&mut canvas);
    //draw_ground(&mut canvas);
    //draw_surf(&mut canvas);
    draw_sphere(&mut canvas);
    
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
