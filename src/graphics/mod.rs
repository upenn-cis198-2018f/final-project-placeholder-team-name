
use gl::types::*;
use std::ptr;
use std::ffi;
use cgmath::*;
use cgmath::prelude::*;
use std::mem::{size_of};

mod shaders;

type Vec4 = Vector4<f32>;
type Vec3 = Vector3<f32>;
type Mat4 = Matrix4<f32>;
type Pt3 = Point3<f32>;

/*
Used to store vertex data that will be transferred to the VBO.
We require that it's layout is like that of a C struct.
*/
#[repr(C)]
struct Vertex {
    position: Vec4,
    color: Vec4,
    normal: Vec3
}

impl Vertex {
    fn new(position: Vec4, color: Vec4, normal: Vec3) -> Vertex {
        Vertex {
            position, color, normal
        }
    }
}

const MAX_NUM_VERTICES: u32 = 1e6 as u32;

pub struct GraphicsState {
    vao: GLuint,
    vbo: GLuint,
    program: GLuint,
    mv_matrix_uniform: GLint,
    proj_matrix_uniform: GLint
}

/*
   TODO:
   Cleanup OpenGL state in destructor.
*/
impl GraphicsState {

    pub fn new() -> GraphicsState {
        let mut state = GraphicsState {
            vao: 0,
            vbo: 0,
            program: 0,
            mv_matrix_uniform: -1,
            proj_matrix_uniform: -1
        };
        unsafe {
            state.setup_gl();
        }
        state
    }

    unsafe fn setup_gl(&mut self) {
        let vertex_data: [GLfloat; 6] = [0.0, 0.0, 0.5, 0.5, -0.5, 0.5];

        gl_log_version_info();

        gl::GenVertexArrays(1, &mut self.vao);
        gl::BindVertexArray(self.vao);

        gl::GenBuffers(1, &mut self.vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        let buffer_size = (size_of::<Vertex>() as u32) * MAX_NUM_VERTICES;
        gl::BufferData(gl::ARRAY_BUFFER, buffer_size as isize,
            ptr::null(), gl::STATIC_DRAW);

        self.setup_program();
        
        log_gl_errors("setup_gl");
    }

    unsafe fn setup_program(&mut self) {
        let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
        gl::ShaderSource(vertex_shader, 1, &(shaders::VERTEX_SHADER_SRC.as_ptr() as *const _),
            ptr::null());
        gl::CompileShader(vertex_shader);
        let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
        gl::ShaderSource(fragment_shader, 1, &(shaders::FRAGMENT_SHADER_SRC.as_ptr() as *const _),
            ptr::null());
        gl::CompileShader(fragment_shader);
        self.program = gl::CreateProgram();
        gl::AttachShader(self.program, vertex_shader);
        gl::AttachShader(self.program, fragment_shader);
        gl::LinkProgram(self.program);
        gl::ValidateProgram(self.program);

        log_shader_info_logs("vertex shader log", vertex_shader);
        log_shader_info_logs("frag shader log", fragment_shader);
        log_program_info_logs("program log", self.program);

        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        log_gl_errors("create program");

        // setup uniforms and attributes from the program
        self.mv_matrix_uniform = gl::GetUniformLocation(
            self.program, b"mv_matrix\0".as_ptr() as *const _);
        self.proj_matrix_uniform = gl::GetUniformLocation(
            self.program, b"proj_matrix\0".as_ptr() as *const _);
        assert!(self.mv_matrix_uniform != -1);
        assert!(self.proj_matrix_uniform != -1);

       let position_attrib = gl::GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
       let color_attrib = gl::GetAttribLocation(self.program, b"color\0".as_ptr() as *const _);
       let normal_attrib = gl::GetAttribLocation(self.program, b"normal\0".as_ptr() as *const _);
       assert!(position_attrib != -1);
       assert!(color_attrib != -1);
       assert!(normal_attrib != -1);

       gl::VertexAttribPointer(
           position_attrib as GLuint, 4, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
            0 as *const _);
       gl::VertexAttribPointer(
           color_attrib as GLuint, 4, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
           size_of::<Vector4<f32>>() as *const _);
       gl::VertexAttribPointer(
           normal_attrib as GLuint, 4, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
           (2 * size_of::<Vector4<f32>>()) as *const _);

       gl::EnableVertexAttribArray(position_attrib as GLuint);
       gl::EnableVertexAttribArray(color_attrib as GLuint);
       gl::EnableVertexAttribArray(normal_attrib as GLuint);
    }

    pub fn draw_frame(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.program);
            
            // set the uniform values
            let mv_matrix: Mat4 = Mat4::look_at_dir(
                Pt3::new(0_f32, 0_f32, 3_f32),
                Vec3::new(0_f32, 0_f32, -1_f32),
                Vec3::new(0_f32, 1_f32, 0_f32)
            );
            let proj_matrix: Mat4 = cgmath::perspective(
                Deg(30_f32), 2_f32, 1_f32, 100_f32);
            gl::UniformMatrix4fv(
                self.mv_matrix_uniform, 1, 0, mv_matrix.as_ptr());
            gl::UniformMatrix4fv(
                self.proj_matrix_uniform, 1, 0, proj_matrix.as_ptr());

            // buffer some sample vertex data, TODO
            let z_pos = -50_f32;
            let vertex_data: [Vertex; 3] = [
                Vertex::new(
                    Vec4::new(0_f32, 0_f32, z_pos, 1_f32),
                    Vec4::new(1_f32, 0_f32, 0_f32, 1_f32),
                    Vec3::new(1_f32, 0_f32, 0_f32)
                    ),
                Vertex::new(
                    Vec4::new(10_f32, 0_f32, z_pos, 1_f32),
                    Vec4::new(0_f32, 1_f32, 0_f32, 1_f32),
                    Vec3::new(1_f32, 0_f32, 0_f32)
                    ),
                Vertex::new(
                    Vec4::new(0_f32, 10_f32, z_pos, 1_f32),
                    Vec4::new(0_f32, 0_f32, 1_f32, 1_f32),
                    Vec3::new(1_f32, 0_f32, 0_f32)
                    )
            ];
            let data_size = size_of::<Vertex>() * vertex_data.len();
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, data_size as isize,
                              vertex_data.as_ptr() as *const _);

            //gl::PointSize(40.0);
            //gl::DrawArrays(gl::POINTS, 0, 1);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
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

