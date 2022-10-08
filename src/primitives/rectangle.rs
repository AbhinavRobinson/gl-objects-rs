use crate::object::BufferData;
use crate::object::OpenGLObjectTrait;
use crate::object::MVP;
use gfx_maths::Mat4;
use gfx_maths::Vec3;
use glow::HasContext;
use glow::NativeProgram;

#[derive(Debug)]
pub struct Rectangle {
    pub width: u32,
    pub height: u32,
    pub program: Option<Box<NativeProgram>>,
    buffers: Option<BufferData>,
    source: String,
    pub matrix: MVP,
}

impl Rectangle {
    pub fn new(width: u32, height: u32, source: &str) -> Self {
        Self {
            height,
            width,
            program: None,
            buffers: None,
            source: source.to_string(),
            matrix: MVP::new(800, 600),
        }
    }
}

impl Rectangle {
    pub fn is_in_bounding_box(&self, x: i32, y: i32) -> bool {
        let data = self.matrix.view * self.matrix.model;
        if x >= data.x as i32
            && x as f32 <= data.x as f32 + self.width as f32
            && y >= data.y as i32
            && y as f32 <= data.y as f32 + self.height as f32
        {
            return true;
        }
        false
    }
}

impl OpenGLObjectTrait for Rectangle {
    fn attach(&mut self, gl: &glow::Context) {
        unsafe {
            let program = gl.create_program().expect("Cannot create program");

            Self::setup_shaders(gl, &program, self.source.clone());
            gl.use_program(Some(program));

            let vertices = [
                0f32,
                0f32,
                0f32,
                self.height as f32,
                self.width as f32,
                self.height as f32,
                self.width as f32,
                0f32,
            ];

            self.buffers = Some(Self::setup_buffers(
                gl,
                &vertices,
                &vec![0u32, 1, 2, 2, 3],
                2,
                8,
            ));

            let matrix =
                self.matrix.projection * self.matrix.view * Mat4::translate(self.matrix.model);

            let proj_matrix = gl.get_uniform_location(program, "u_proj_matrix");

            gl.uniform_matrix_4_f32_slice(proj_matrix.as_ref(), false, &matrix.values);

            self.program = Some(Box::new(program));
        }
    }

    fn render(&mut self, gl: &glow::Context) {
        unsafe {
            gl.draw_elements(glow::TRIANGLES, 6, glow::UNSIGNED_INT, 0);
        }
    }

    fn detach(&mut self, gl: &glow::Context) {
        if self.program.is_some() {
            unsafe {
                gl.delete_program(**self.program.as_ref().unwrap());
            }
        };
    }

    fn move_model(&mut self, movement_x: f32, movement_y: f32, movement_z: f32) {
        self.matrix.model += Vec3::new(movement_x, movement_y, movement_z);
    }

    fn set_model(&mut self, movement_x: f32, movement_y: f32, movement_z: f32) {
        self.matrix.model = Vec3::new(movement_x, movement_y, movement_z);
    }

    fn window_resize(&mut self, _draw_size: [f32; 2], size: [f32; 2]) {
        {
            self.matrix.projection =
                Mat4::orthographic_opengl(0.0, size[0] as f32, size[1] as f32, 0.0, -1.0, 1.0);
        }
    }
}
