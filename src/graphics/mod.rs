mod shaders;

use gl::types::*;
use std::ptr;
use std::ffi;
use cgmath::*;
use cgmath::prelude::*;
use std::mem::{size_of};
use std::f32::*;

const MAX_NUM_TRIANGLES: u32 = 1e6 as u32;
const MAX_NUM_VERTICES: u32 = 1e6 as u32;

pub type Vec4 = Vector4<f32>;
pub type Vec3 = Vector3<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Pt3 = Point3<f32>;

/*
   TODO

   Configure OpenGL properly. Clear the depth buffer and stencil
   buffer on every frame. Have an option to draw lines to debug.
*/

/*
Used to store vertex data that will be transferred to the VBO.
We require that it's layout is like that of a C struct.
*/
#[repr(C)]
pub struct Vertex {
    position: Vec4,
    color: Vec4,
    normal: Vec3
}

impl Vertex {
    pub fn new(position: Vec4, color: Vec4, normal: Vec3) -> Vertex {
        Vertex {
            position, color, normal
        }
    }
}

pub struct GraphicsState {
    vao: GLuint,
    vbo: GLuint,
    index_buffer: GLuint,
    program: GLuint,
    mv_matrix_uniform: GLint,
    proj_matrix_uniform: GLint,
    framebuffer_width: f64,
    framebuffer_height: f64
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
            index_buffer: 0,
            program: 0,
            mv_matrix_uniform: -1,
            proj_matrix_uniform: -1,
            framebuffer_width: 1000.0,
            framebuffer_height: 600.0
        };
        unsafe {
            state.setup_gl();
        }
        state
    }

    unsafe fn setup_gl(&mut self) {
        gl_log_version_info();

        gl::GenVertexArrays(1, &mut self.vao);
        gl::BindVertexArray(self.vao);

        gl::GenBuffers(1, &mut self.vbo);
        gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
        let buffer_size = (size_of::<Vertex>() as u32) * MAX_NUM_VERTICES;
        gl::BufferData(gl::ARRAY_BUFFER, buffer_size as isize,
            ptr::null(), gl::STATIC_DRAW);

        gl::GenBuffers(1, &mut self.index_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
        let elem_buffer_size = (size_of::<GLuint>() as u32) * MAX_NUM_TRIANGLES;
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, elem_buffer_size as isize,
            ptr::null(), gl::STATIC_DRAW);

        self.setup_program();

        gl::Enable(gl::DEPTH_TEST);
        
        log_gl_errors("setup_gl");
    }

    pub fn update_framebuffer_size(&mut self, w: f64, h: f64) {
        self.framebuffer_width = w;
        self.framebuffer_height = h;
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

       log_gl_errors("setup attributes");
    }

    pub fn draw_frame(&self, canvas: &Canvas) {
        unsafe {
            let bg_col = canvas.background_color;
            gl::ClearColor(bg_col.x, bg_col.y, bg_col.z, bg_col.z);
            gl::ClearDepth(1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT |
                      gl::DEPTH_BUFFER_BIT | gl::STENCIL_BUFFER_BIT);

            gl::UseProgram(self.program);

            // TODO: for debug only
            gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            // TODO: don't hard-code the proj. calc aspect ratio
            let aspect_ratio = self.framebuffer_width / self.framebuffer_height;
            let proj_matrix: Mat4 = cgmath::perspective(
                Deg(30_f32), aspect_ratio as f32, 1_f32, 100_f32);
            gl::UniformMatrix4fv(
                self.mv_matrix_uniform, 1, 0, canvas.mv_matrix.as_ptr());
            gl::UniformMatrix4fv(
                self.proj_matrix_uniform, 1, 0, proj_matrix.as_ptr());

            let vertex_data = &canvas.vertex_data;
            let index_data = &canvas.index_data;
            let vertex_data_size = size_of::<Vertex>() * vertex_data.len();
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertex_data_size as isize,
                              vertex_data.as_ptr() as *const _);
            let elem_data_size = size_of::<GLuint>() * index_data.len();
            gl::BufferSubData(gl::ELEMENT_ARRAY_BUFFER, 0, elem_data_size as isize,
                              index_data.as_ptr() as *const _);

            gl::DrawElements(gl::TRIANGLES, index_data.len() as i32, gl::UNSIGNED_INT,
                             ptr::null() as *const _);
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

fn cross4(a: Vec4, b: Vec4) -> Vec3 {
    a.truncate().cross(b.truncate())
}

fn faces_to_indices(faces: Vec<[u32; 4]>) -> Vec<u32> {
    faces.into_iter().map(|face| {
        vec![face[0], face[1], face[2], face[0], face[2], face[3]]
    }).flatten().collect()
}

pub struct Canvas {
    background_color: Vec4,
    mv_matrix: Mat4,
    vertex_data: Vec<Vertex>,
    index_data: Vec<GLuint>
}

impl Canvas {
    pub fn new() -> Canvas {
        let mut canvas = Canvas {
            background_color: Vector4::new(0f32, 0f32, 0f32, 1f32),
            mv_matrix: Matrix4::zero(),
            vertex_data: Vec::new(),
            index_data: Vec::new()
        };
        canvas.set_camera(Vec3::new(0f32, 0f32, 20f32),
            Vec3::zero(), Vec3::new(0f32, 1f32, 0f32));
        canvas
    }

    pub fn set_background_color(&mut self, color: Vec4) {
        self.background_color = color;
    }

    pub fn set_camera(&mut self, eye: Vec3, target: Vec3, up: Vec3) {
        self.mv_matrix = Matrix4::look_at_dir(Pt3::from_vec(eye), target - eye, up);
    }
    
    pub fn draw_triangle(&mut self, a: Vec4, b: Vec4, c: Vec4, color: Vec4) {
        self.draw_half_pgram(a, b - a, c - a, color);
    }

    // Draw the triangle spanned by the given vectors
    pub fn draw_half_pgram(&mut self, pos: Vec4, u: Vec4, v: Vec4, color: Vec4) {
        let normal = cross4(u, v);
        let mut vertices = vec![
            Vertex::new(pos, color, normal),
            Vertex::new(pos + u, color, normal),
            Vertex::new(pos + v, color, normal)
        ];
        let mut indices = vec![0, 1, 2];
        self.add_data(&mut vertices, &mut indices);
    }

    // Draw the parallelogram spanned by the given vectors
    pub fn draw_pgram(&mut self, pos: Vec4, u: Vec4, v: Vec4, color: Vec4) {
        let normal = cross4(u, v);
        let mut vertices = vec![
            Vertex::new(pos, color, normal),
            Vertex::new(pos + u, color, normal),
            Vertex::new(pos + v, color, normal),
            Vertex::new(pos + u + v, color, normal)
        ];
        let mut indices = vec![0, 1, 3, 0, 3, 2];
        self.add_data(&mut vertices, &mut indices);
    }

    // Draw the parallelopiped spanned by the given vectors
    pub fn draw_ppiped(&mut self, pos: Vec4, u: Vec4, v: Vec4, w: Vec4, color: Vec4) {
        let mut positions = vec![
            pos, pos + u, pos + v, pos + u + v,
            pos + w, pos + w + u, pos + w + v, pos + w + u + v
        ];
        let center = ((pos + u + v + w) + pos) / 2f32;
        let mut vertices = positions.into_iter().map(|pos| {
            let normal = (pos - center).truncate().normalize();
            Vertex::new(pos, color, normal)
        }).collect();
        let mut faces = vec![
            [0, 1, 3, 2],
            [0, 1, 5, 4],
            [4, 5, 7, 6],
            [1, 3, 7, 5],
            [3, 2, 6, 7],
            [0, 2, 4, 6]
        ];
        let mut indices = faces_to_indices(faces);
        self.add_data(&mut vertices, &mut indices);
    }

    pub fn draw_surface<F>(&mut self, samples_x: u32, samples_y: u32, f: F) 
        where F: Fn(u32, u32) -> Vertex {
        // TODO - sample the surface at a regular interval and 
        // draw surface area elements as you go.
        // TODO - also make convenience funcs to convert the sample num to some range
    }

    // the indices must be relative to the start of the list of given vertices
    fn add_data(&mut self, vertices: &mut Vec<Vertex>, indices: &mut Vec<GLuint>) {
        let base_index = self.vertex_data.len() as u32;
        for index in indices.iter_mut() {
            *index += base_index;
        }
        self.vertex_data.append(vertices);
        self.index_data.append(indices);
    }
}

