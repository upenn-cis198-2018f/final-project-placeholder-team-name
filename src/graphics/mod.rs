mod shaders;

use gl::types::*;
use std::ptr;
use std::ffi;
use cgmath::*;
use cgmath::prelude::*;
use std::mem::{size_of};
use std::f32::*;
use std::f32::consts::*;

const VERTEX_BUFFER_SIZE: usize = 1e8 as usize;
const INDEX_BUFFER_SIZE: usize = 1e8 as usize;

pub type Vec4 = Vector4<f32>;
pub type Vec3 = Vector3<f32>;
pub type Vec2 = Vector2<f32>;
pub type Mat4 = Matrix4<f32>;
pub type Pt3 = Point3<f32>;

/*
Used to store vertex data that will be transferred to the VBO.
We require that it's layout is like that of a C struct.
*/
#[repr(C)]
pub struct Vertex {
    position: Vec3,
    color: Vec4,
    normal: Vec3
}

impl Vertex {
    pub fn new(position: Vec3, color: Vec4, normal: Vec3) -> Vertex {
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
    light_pos_uniform: GLint,
    framebuffer_width: f64,
    framebuffer_height: f64
}

impl GraphicsState {

    pub fn new() -> GraphicsState {
        let mut state = GraphicsState {
            vao: 0,
            vbo: 0,
            index_buffer: 0,
            program: 0,
            mv_matrix_uniform: -1,
            proj_matrix_uniform: -1,
            light_pos_uniform: -1,
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
        gl::BufferData(gl::ARRAY_BUFFER, VERTEX_BUFFER_SIZE as isize,
            ptr::null(), gl::STATIC_DRAW);

        gl::GenBuffers(1, &mut self.index_buffer);
        gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.index_buffer);
        gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, INDEX_BUFFER_SIZE as isize,
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
        self.light_pos_uniform = gl::GetUniformLocation(
            self.program, b"light_world_pos\0".as_ptr() as *const _);
        assert!(self.mv_matrix_uniform != -1);
        assert!(self.proj_matrix_uniform != -1);
        assert!(self.light_pos_uniform != -1);

       let position_attrib = gl::GetAttribLocation(self.program, b"position\0".as_ptr() as *const _);
       let color_attrib = gl::GetAttribLocation(self.program, b"color\0".as_ptr() as *const _);
       let normal_attrib = gl::GetAttribLocation(self.program, b"normal\0".as_ptr() as *const _);
       assert!(position_attrib != -1);
       assert!(color_attrib != -1);
       assert!(normal_attrib != -1);

       gl::VertexAttribPointer(
           position_attrib as GLuint, 3, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
            0 as *const _);
       gl::VertexAttribPointer(
           color_attrib as GLuint, 4, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
           size_of::<Vec3>() as *const _);
       gl::VertexAttribPointer(
           normal_attrib as GLuint, 3, gl::FLOAT, gl::FALSE, size_of::<Vertex>() as i32,
           (size_of::<Vec3>() + size_of::<Vec4>()) as *const _);

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
            //gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE);

            let aspect_ratio = self.framebuffer_width / self.framebuffer_height;
            let proj_matrix: Mat4 = cgmath::perspective(
                Deg(30_f32), aspect_ratio as f32, 1_f32, 10000_f32);
            gl::UniformMatrix4fv(
                self.mv_matrix_uniform, 1, 0, canvas.mv_matrix.as_ptr());
            gl::UniformMatrix4fv(
                self.proj_matrix_uniform, 1, 0, proj_matrix.as_ptr());
            gl::Uniform3fv(
                self.light_pos_uniform, 1, canvas.light_position.as_ptr());

            let vertex_data = &canvas.vertex_data;
            let index_data = &canvas.index_data;
            let vertex_data_size = size_of::<Vertex>() * vertex_data.len();
            assert!(vertex_data_size < VERTEX_BUFFER_SIZE, "too much vertex data: {}", vertex_data_size);
            gl::BufferSubData(gl::ARRAY_BUFFER, 0, vertex_data_size as isize,
                              vertex_data.as_ptr() as *const _);
            let elem_data_size = size_of::<GLuint>() * index_data.len();
            assert!(elem_data_size < INDEX_BUFFER_SIZE, "too much index data: {}", elem_data_size);
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
    light_position: Vec3,
    mv_matrix: Mat4,
    vertex_data: Vec<Vertex>,
    index_data: Vec<GLuint>
}

impl Canvas {
    pub fn new() -> Canvas {
        let mut canvas = Canvas {
            background_color: Vec4::new(0f32, 0f32, 0f32, 1f32),
            light_position: 100f32 * Vec3::new(1f32, 1f32, -1f32), 
            mv_matrix: Mat4::zero(),
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

    pub fn set_light_position(&mut self, pos: Vec3) {
        self.light_position = pos;
    }
    
    // NB: The vertices must be given in CCW winding order for the
    // normals to be correctly set
    pub fn draw_triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: Vec4) {
        self.draw_half_pgram(a, b - a, c - a, color);
    }

    // Draw the triangle spanned by the given vectors
    pub fn draw_half_pgram(&mut self, pos: Vec3, u: Vec3, v: Vec3, color: Vec4) {
        let normal = u.cross(v);
        let mut vertices = vec![
            Vertex::new(pos, color, normal),
            Vertex::new(pos + u, color, normal),
            Vertex::new(pos + v, color, normal)
        ];
        let mut indices = vec![0, 1, 2];
        self.add_data(&mut vertices, &mut indices);
    }

    // Draw the parallelogram spanned by the given vectors
    pub fn draw_pgram(&mut self, pos: Vec3, u: Vec3, v: Vec3, color: Vec4) {
        let normal = u.cross(v);
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
    pub fn draw_ppiped(&mut self, pos: Vec3, u: Vec3, v: Vec3, w: Vec3, color: Vec4) {
        let positions = vec![
            pos, pos + u, pos + v, pos + u + v,
            pos + w, pos + w + u, pos + w + v, pos + w + u + v
        ];
        let faces = vec![
            [0, 2, 3, 1],
            [0, 1, 5, 4],
            [4, 5, 7, 6],
            [1, 3, 7, 5],
            [3, 2, 6, 7],
            [2, 0, 4, 6]
        ];
        for face in faces {
            let pos_a = positions[face[0]];
            let pos_b = positions[face[1]];
            let pos_c = positions[face[2]];
            let u = pos_b - pos_a;
            let v = pos_c - pos_a;
            let normal = u.cross(v);

            let mut vertices = Vec::new();
            for vert_index in face.into_iter() {
                let pos = positions[*vert_index];
                let vertex = Vertex::new(pos, color, normal);
                vertices.push(vertex);
            }
            let mut indices = vec![0, 1, 2, 0, 2, 3];
            self.add_data(&mut vertices, &mut indices);
        }
    }

    fn draw_tri_from_array(&mut self, pos_col_data: [(Vec3, Vec4); 3]) {
        let (a, c_a) = pos_col_data[0];
        let (b, c_b) = pos_col_data[1];
        let (c, c_c) = pos_col_data[2];
        let u = b - a;
        let v = c - a;
        let normal = u.cross(v);
        let mut vertices = vec![
            Vertex::new(a, c_a, normal),
            Vertex::new(b, c_b, normal),
            Vertex::new(c, c_c, normal)
        ];
        let mut indices = vec![0, 1, 2];
        self.add_data(&mut vertices, &mut indices);
    }

    /*
        Draws a surface by sampling a surface function on a grid of values.
        The function f takes index_x, index_y, float_x, and float_y, and returns (position, color),
        where float_x is the sample coordinate converted to the range [0, 1) for convenience.
        samples_x and samples_y are the number of samples in each axis.
    */
    pub fn draw_surface<F>(&mut self, samples_x: usize, samples_y: usize, f: F) 
        where F: Fn(usize, usize, f32, f32) -> (Vec3, Vec4) {
        // TODO - make convenience funcs to convert the sample num to some float range

        // create a grid of sampled values
        let mut positions: Vec<Vec<(Vec3, Vec4)>> = Vec::new();
        for x in 0..samples_x {
            let mut col_vec: Vec<(Vec3, Vec4)> = Vec::new();
            for y in 0..samples_y {
                let nx = (x as f32) / ((samples_x - 1) as f32);
                let ny = (y as f32) / ((samples_y - 1) as f32);
                let sample = f(x, y, nx, ny);
                col_vec.push(sample);    
            }
            positions.push(col_vec);
        }
        
        // use the samples to construct a mesh of area elements
        for x in 0..(samples_x - 1) {
            for y in 0..(samples_y - 1) {
                let s = positions[x][y];       
                let s_u = positions[x + 1][y];
                let s_v = positions[x][y + 1];
                let s_uv = positions[x + 1][y + 1];
                self.draw_tri_from_array([s, s_u, s_uv]);
                self.draw_tri_from_array([s, s_uv, s_v]);
            }
        }
    }

    pub fn draw_sphere(&mut self, center: Vec3, r: f32, color: Vec4) {
        self.draw_surface(100, 100, |sx, sy, nx, ny| {
            let angle_vert = nx * PI;
            let angle_hor = ny * 2f32 * PI;
            let y = angle_vert.cos() * r;
            let x = r * angle_vert.sin() * angle_hor.cos();
            let z = r * angle_vert.sin() * angle_hor.sin();
            let p = vec3(x, y, z);
            (p + center, color)
        });
    }

    pub fn draw_torus(&mut self, center: Vec3, r_major: f32, r_minor: f32, color: Vec4) {
        self.draw_surface(100, 100, |sx, sy, nx, ny| {
            let angle_major = nx * 2f32 * PI;
            let angle_minor = ny * 2f32 * PI;
            let major_pt: Vec3 = r_major * (
                angle_major.cos() * vec3(1f32, 0f32, 0f32) +
                angle_major.sin() * vec3(0f32, 1f32, 0f32));
            let minor_x_vec = -major_pt.normalize();
            let minor_y_vec = vec3(0f32, 0f32, 1f32);
            let minor_pt: Vec3 = r_minor * (
                angle_minor.cos() * minor_x_vec +
                angle_minor.sin() * minor_y_vec);
            (center + major_pt + minor_pt, color)
        });
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

