
use gl::types::*;
use std::ptr;
use std::ffi;

pub struct GraphicsState {
    program: GLuint
}

/*
   TODO:
   Cleanup OpenGL state in destructor.
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

        let mut vao: GLuint = 0;
        gl::GenVertexArrays(1, &mut vao);
        gl::BindVertexArray(vao);

        self.program = GraphicsState::create_program();
        
        log_gl_errors("setup_gl");
    }

    unsafe fn create_program() -> GLuint {
        let vertex_shader_src: &[u8] = b"
        #version 400

        void main() {
            gl_Position = vec4(0.0, 0.0, 0.5, 1.0);
        }
        \0";
        let fragment_shader_src: &[u8] = b"
        #version 400

        out vec4 frag_color;

        void main() {
            frag_color = vec4(0.0, 1.0, 0.0, 1.0);
        }
        \0";
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
        gl::ValidateProgram(program);

        log_shader_info_logs("vertex shader log", vertex_shader);
        log_shader_info_logs("frag shader log", fragment_shader);
        log_program_info_logs("program log", program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        log_gl_errors("create program");

        program
    }

    pub fn draw_frame(&self) {
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            gl::PointSize(40.0);
            gl::DrawArrays(gl::POINTS, 0, 1);
            //gl::DrawArrays(gl::TRIANGLES, 0, 3);
            log_gl_errors("draw frame");
        }
    }
}

fn log_program_info_logs(msg: &str, program: GLuint) {
    unsafe {
        let mut log_len: GLint = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
        if log_len > 0 {
            let mut log_bytes: Vec<i8> = vec![0, log_len as i8];
            let mut bytes_written: GLsizei = 0;
            gl::GetProgramInfoLog(program, log_len as GLsizei,
                                  &mut bytes_written, log_bytes.as_mut_ptr());
            let c_str = ffi::CStr::from_ptr(log_bytes.as_ptr());
            println!("{}:\n{}", msg, c_str.to_str().unwrap());
        }
    }
}

fn log_shader_info_logs(msg: &str, shader: GLuint) {
    unsafe {
        let mut log_len: GLint = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_len);
        if log_len > 0 {
            let mut log_bytes: Vec<i8> = vec![0, log_len as i8];
            let mut bytes_written: GLsizei = 0;
            gl::GetShaderInfoLog(shader, log_len as GLsizei,
                                  &mut bytes_written, log_bytes.as_mut_ptr());
            let c_str = ffi::CStr::from_ptr(log_bytes.as_ptr());
            println!("{}:\n{}", msg, c_str.to_str().unwrap());
        }
    }
}
fn log_gl_errors(msg: &str) {
    if let Some(err) = get_gl_error() {
        println!("GL error: {:?}, {:?}", msg, err);
        assert!(false);
    }
}

fn get_gl_error() -> Option<String> {
    unsafe {
        let error_code: GLenum = gl::GetError();
        let error_str = match error_code {
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
        match error_str {
            Some(e) => Some(String::from(e)),
            None => None
        }
    }
}

fn get_gl_str(str_enum: GLenum) -> &'static str {
    unsafe {
        let gl_str: *const i8 = gl::GetString(str_enum) as *const i8;
        let c_str = ffi::CStr::from_ptr(gl_str);
        c_str.to_str().unwrap()
    }
}

fn gl_log_version_info() {
    let vendor = get_gl_str(gl::VENDOR);
    let renderer = get_gl_str(gl::RENDERER);
    let version = get_gl_str(gl::VERSION);
    let glsl_version = get_gl_str(gl::SHADING_LANGUAGE_VERSION);
    println!("Vender: {:?}\nRenderer: {:?}\nversion: {:?}\nGLSL version: {:?}",
             vendor, renderer, version, glsl_version);
}

