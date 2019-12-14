use crate::support::{System, make_shaders};
use glium::{Surface};

pub struct Mesh<T : Copy, I : Copy + glium::index::Index> {
    pub index_buffer : glium::IndexBuffer<I>,
    pub vertex_buffer : glium::VertexBuffer<T>,
    pub program: glium::Program,
}

pub trait MeshTrait {
    fn draw(&self, display : &glium::Display);
}

impl <T :Copy ,I : Copy + glium::index::Index> MeshTrait for Mesh<T,I> {
    fn draw(&self, display : &glium::Display) {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

impl<T :Copy ,I : Copy + glium::index::Index> Mesh<T,I> {
    pub fn new(system : &System, vertex_buffer : glium::VertexBuffer<T>, index_buffer : glium::IndexBuffer<I>) -> Self {

        let program = make_shaders(&system.display);
        Self {
            vertex_buffer,
            index_buffer,
            program
        }
    }

    pub fn draw(&self, display : &glium::Display) {
        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ]
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
        target.finish().unwrap();
    }
}

