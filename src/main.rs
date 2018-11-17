#[macro_use]
extern crate glium;

/**
  TODO
  Matt: Glium doesn't seem to work on my mac. Instead try to use
  an OpenGL or Vulkan lib (maybe gl-rs) and do things the old way.
  */

mod graphics;

use glium::*;
use glium::Surface;
use glium::glutin::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn main() {
    let mut events_loop = EventsLoop::new();
    let window = WindowBuilder::new()
        .with_title("music visualizer")
        .with_dimensions(dpi::LogicalSize::new(1000.0, 700.0));
    let context = ContextBuilder::new()
        .with_vsync(true);
        //.with_gl(GlRequest::Specific(glutin::Api::OpenGl, (3, 3)))
        //.with_gl_profile(GlProfile::Core);
    let mut display = Display::new(window, context, &events_loop).unwrap();

    println!("OpenGL version: {:?}\nGLSL support: {:?}\nframebuffer size: {:?}",
             display.get_opengl_version(),
             display.get_supported_glsl_version(),
             display.get_framebuffer_dimensions());

    let v1 = Vertex { position: [0.0, 0.0] };
    let v2 = Vertex { position: [0.5, 0.5] };
    let v3 = Vertex { position: [-0.5, 0.5] };
    let vertices = vec![v1, v2, v3];
    let vertex_buffer = VertexBuffer::new(&display, &vertices).unwrap();
    let indices = index::NoIndices(index::PrimitiveType::TrianglesList);

    let vertex_shader_src = r#"
        #version 140

        in vec2 position;

        void main() {
            gl_Position = vec4(position, 0.0, 1.0);
        }
    "#;

    let fragment_shader_src = r#"
        #version 140

        out vec4 color;

        void main() {
            color = vec4(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    let program = Program::from_source(&display, vertex_shader_src,
                                       fragment_shader_src, None).unwrap();

    assert_no_gl_error!(display);

    let mut keep_running = true;
    while keep_running {
        events_loop.poll_events(|event| {
            let keep_open = handle_event(&mut display, event);
            if !keep_open && keep_running {
                keep_running = false
            };
        });

        let mut target = display.draw();
        target.clear_color(1.0, 0.0, 0.0, 1.0);
        target.draw(&vertex_buffer, &indices, &program, &uniforms::EmptyUniforms,
                    &Default::default()).unwrap();
        target.finish().unwrap();
        assert_no_gl_error!(display);

        //std::thread::sleep(std::time::Duration::from_millis(17));
    }
}

fn handle_event(display: &mut Display, event: Event) -> bool {
    match event {
        Event::WindowEvent{event: win_event, ..} => {
            match win_event {
                WindowEvent::CloseRequested => return false,
                WindowEvent::Resized(logical_size) => {
                    // when the window resizes, we must resize the context
                    //let dpi = display.get_hidpi_factor();
                    //display.resize(logical_size.to_physical(dpi));
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
