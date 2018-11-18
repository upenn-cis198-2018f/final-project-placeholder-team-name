
use gl::types::*;
use std::ptr;
use std::ffi;

pub struct GraphicsState {
    program: GLuint
}

/*
   TODO:
   Cleanup in destructor.
   Use the gl-rs custom binding generator, the current one
   doesn't even load the VAO functions
*/
impl GraphicsState {

    pub fn new() -> GraphicsState {
        let mut state = GraphicsState { program: 0 };
        unsafe {
            state.setup_gl();
        }
        state
    }

    unsafe fn setup_gl(&mut self) {
        let vertex_data: [GLfloat; 6] = [0.0, 0.0, 0.5, 0.5, -0.5, 0.5];

        gl_log_version_info();

        // setup debugging
        gl::Enable(gl::DEBUG_OUTPUT);
        // TODO: this func isn't loaded
        //gl::DebugMessageCallback(handle_gl_debug_message, ptr::null());

        self.program = GraphicsState::create_program();

        // TODO: this func isn't loaded
        //let mut vao: GLuint = 0;
        //gl::CreateVertexArrays(1, &mut vao);
        //gl::BindVertexArray(vao);
    }

    unsafe fn create_program() -> GLuint {
        let vertex_shader_src = r#"
        #version 450 core

        void main() {
            gl_Position = vec4(0.0, 0.0, 0.5, 1.0);
        }
        \0"#;
        let fragment_shader_src = r#"
        #version 450 core

        out vec4 color;

        void main() {
            color = vec4(0.0, 1.0, 0.0, 1.0);
        }
        \0"#;
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &(vertex_shader_src.as_ptr() as *const _),
            ptr::null());
        gl::CompileShader(vertex_shader);
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &(fragment_shader_src.as_ptr() as *const _),
            ptr::null());
        gl::CompileShader(fragment_shader);
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        program
    }

    pub fn draw_frame(&self) {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            //gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::PointSize(40.0);
            gl::DrawArrays(gl::POINTS, 0, 1);
        }
    }
}

extern "system" fn handle_gl_debug_message(
    source: GLenum, msg_type: GLenum, id: GLuint, severity: GLenum,
                        length: GLsizei, message: *const GLchar,
                        user: *mut ffi::c_void) {
    println!("handling debug message!");
}

fn log_gl_errors() {
    unsafe {
        let error_code: GLenum = gl::GetError();
        let error_str: Option<&str> = match error_code {
            gl::NO_ERROR => None,
            gl::INVALID_ENUM => Some("INVALID_ENUM"),
            gl::INVALID_VALUE => Some("INVALID_VALUE"),
            gl::INVALID_OPERATION => Some("INVALID_OPERATION"),
            gl::INVALID_FRAMEBUFFER_OPERATION => Some("INVALID_FRAMEBUFFER_OPERATION"),
            gl::OUT_OF_MEMORY => Some("OUT_OF_MEMORY"),
            gl::STACK_UNDERFLOW => Some("STACK_UNDERFLOW"),
            gl::STACK_OVERFLOW => Some("STACK_OVERFLOW"),
            _ => Some("unknown error")
        };
        if let Some(s) = error_str {
            println!("gl error: {:?}", s);
        }
    }
}

/*
fn ptr_to_str(raw_ptr: *const u8) -> String {
    ffi::CStr::from_ptr(raw_ptr).to_str()
}
*/

fn get_gl_str(str_enum: GLenum) -> &'static str {
    unsafe {
        let gl_str: *const i8 = gl::GetString(str_enum) as *const i8;
        let c_str = ffi::CStr::from_ptr(gl_str);
        c_str.to_str().unwrap()
    }
}

fn gl_log_version_info() {
    unsafe {
        let vendor = get_gl_str(gl::VENDOR);
        let renderer = get_gl_str(gl::RENDERER);
        let version = get_gl_str(gl::VERSION);
        let glsl_version = get_gl_str(gl::SHADING_LANGUAGE_VERSION);
        println!("Vender: {:?}\nRenderer: {:?}\nversion: {:?}\nGLSL version: {:?}",
                 vendor, renderer, version, glsl_version);
    }
}

