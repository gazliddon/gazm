use crate::support::{System, make_shaders};
use glium::{Surface};

pub struct Mesh<T : Copy, I : Copy + glium::index::Index> {
    pub index_buffer : glium::IndexBuffer<I>,
    pub vertex_buffer : glium::VertexBuffer<T>,
    pub program: glium::Program,
}

pub trait MeshTrait {
    // fn draw(&self, display : &mut glium::Frame);
    fn draw(&self, m : cgmath::Matrix4<f32>, display : &mut glium::Frame);
}

impl <T :Copy ,I : Copy + glium::index::Index> MeshTrait for Mesh<T,I> {
    fn draw(&self, m : cgmath::Matrix4<f32> , frame : &mut glium::Frame) {

        let matrix: [[f32; 4]; 4] = m.into();

        // building the uniforms
        let uniforms = uniform! { matrix: matrix };

        frame.draw(&self.vertex_buffer, &self.index_buffer, &self.program, &uniforms, &Default::default()).unwrap();
    }
}

impl<T :Copy ,I : Copy + glium::index::Index> Mesh<T,I> {
    pub fn new(system : &System, vertex_buffer : glium::VertexBuffer<T>, index_buffer : glium::IndexBuffer<I>) -> Self {

        let program = make_shaders(&system.display);
        Self {
            vertex_buffer,
            index_buffer,
            program,
        }
    }
}



