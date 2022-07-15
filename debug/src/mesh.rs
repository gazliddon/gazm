use crate::app::system::System;
use imgui_glium_renderer::glium;
use glium::{ Surface, uniform, program };

pub struct Mesh<T: Copy, I: Copy + glium::index::Index> {
    pub index_buffer: glium::IndexBuffer<I>,
    pub vertex_buffer: glium::VertexBuffer<T>,
    pub program: glium::Program,
}

pub trait MeshTrait {
    fn draw(&self, m: cgmath::Matrix4<f32>, display: &mut glium::Frame);
}

impl<T: Copy, I: Copy + glium::index::Index> MeshTrait for Mesh<T, I> {
    fn draw(&self, m: cgmath::Matrix4<f32>, frame: &mut glium::Frame) {
        let matrix: [[f32; 4]; 4] = m.into();
        let uniforms = uniform! { matrix: matrix };
        frame
            .draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
    }
}

impl<T: Copy, I: Copy + glium::index::Index> Mesh<T, I> {
    pub fn new(
        system: &System,
        vertex_buffer: glium::VertexBuffer<T>,
        index_buffer: glium::IndexBuffer<I>,
    ) -> Self {
        let program = make_shaders(&system.display);
        Self {
            vertex_buffer,
            index_buffer,
            program,
        }
    }
}

fn make_shaders(display: &glium::Display) -> glium::Program {
    program!(display,
    140 => {
        vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec3 color;
                out vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

        fragment: "
                #version 140
                in vec3 vColor;
                out vec4 f_color;
                void main() {
                    f_color = vec4(vColor, 1.0);
                }
            "
    },

    110 => {
        vertex: "
                #version 110
                uniform mat4 matrix;
                attribute vec2 position;
                attribute vec3 color;
                varying vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

        fragment: "
                #version 110
                varying vec3 vColor;
                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
    },

    100 => {
        vertex: "
                #version 100
                uniform lowp mat4 matrix;
                attribute lowp vec2 position;
                attribute lowp vec3 color;
                varying lowp vec3 vColor;
                void main() {
                    gl_Position = vec4(position, 0.0, 1.0) * matrix;
                    vColor = color;
                }
            ",

        fragment: "
                #version 100
                varying lowp vec3 vColor;
                void main() {
                    gl_FragColor = vec4(vColor, 1.0);
                }
            ",
    },
    )
    .unwrap()
}
