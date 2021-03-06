#![allow(unused)]
use crate::openxr_module::OpenXR;
use crate::parser::*;
use crate::SCALE;

use glium::texture::{DepthFormat, DepthTexture2d, MipmapsOption, UncompressedFloatFormat};
use glium::{vertex::VertexBufferAny, Display, DrawParameters, Frame, Program, Surface, Texture2d};
use nalgebra::{Matrix4, Translation3, UnitQuaternion};
use std::collections::HashMap;

use std::ffi::{c_void, CString};
use std::os::raw::*;
use std::rc::Rc;
use x11::{glx, xlib};

pub mod backend;
pub mod camera;
pub mod shaders;

pub struct Window {
    pub context: Rc<glium::backend::Context>,
    pub xr: OpenXR,
    pub shaders: HashMap<String, Program>,
    pub models: HashMap<String, VertexBufferAny>,
    pub textures: HashMap<String, Texture2d>,
    pub depth_textures: Option<(DepthTexture2d, DepthTexture2d)>,
}

impl Window {
    pub fn new() -> Self {
        let mut backend = backend::Backend::new();
        let xr = OpenXR::new(&mut backend);
        let context =
            unsafe { glium::backend::Context::new(backend, false, Default::default()) }.unwrap();

        Self {
            context,
            xr,
            depth_textures: None,
            shaders: HashMap::new(),
            models: HashMap::new(),
            textures: HashMap::new(),
        }
    }
    pub fn create_depth_textures(&mut self) {
        let depth_texture_left = DepthTexture2d::empty_with_format(
            &self.context,
            DepthFormat::F32,
            MipmapsOption::EmptyMipmaps,
            self.xr.swapchains.resolution_left.0,
            self.xr.swapchains.resolution_left.1,
        )
        .unwrap();
        let depth_texture_right = DepthTexture2d::empty_with_format(
            &self.context,
            DepthFormat::F32,
            MipmapsOption::EmptyMipmaps,
            self.xr.swapchains.resolution_right.0,
            self.xr.swapchains.resolution_right.1,
        )
        .unwrap();
        self.depth_textures = Some((depth_texture_left, depth_texture_right));
    }
    pub fn draw(&mut self) {
        let swapchain_image = self.xr.swapchains.get_images();
        if let Some((swapchain_image_left, swapchain_image_right)) = swapchain_image {
            if self.depth_textures.is_none() {
                self.create_depth_textures();
            }
            let depth_textures = self.depth_textures.as_ref().unwrap();

            self.xr.frame_stream_begin();
            let texture_left = unsafe {
                glium::texture::texture2d::Texture2d::from_id(
                    &self.context,
                    glium::texture::UncompressedFloatFormat::U8U8U8U8,
                    swapchain_image_left,
                    false,
                    glium::texture::MipmapsOption::NoMipmap,
                    glium::texture::Dimensions::Texture2d {
                        width: self.xr.swapchains.resolution_left.0,
                        height: self.xr.swapchains.resolution_left.1,
                    },
                )
            };
            let texture_right = unsafe {
                glium::texture::texture2d::Texture2d::from_id(
                    &self.context,
                    glium::texture::UncompressedFloatFormat::U8U8U8U8,
                    swapchain_image_right,
                    false,
                    glium::texture::MipmapsOption::NoMipmap,
                    glium::texture::Dimensions::Texture2d {
                        width: self.xr.swapchains.resolution_right.0,
                        height: self.xr.swapchains.resolution_right.1,
                    },
                )
            };
            let mut target = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                &self.context,
                &texture_left,
                &depth_textures.0,
            )
            .unwrap();
            target.clear_color_and_depth((0.6, 0.0, 0.0, 1.0), 1.0);

            let mut target = glium::framebuffer::SimpleFrameBuffer::with_depth_buffer(
                &self.context,
                &texture_right,
                &depth_textures.1,
            )
            .unwrap();
            target.clear_color_and_depth((0.0, 0.0, 0.6, 1.0), 1.0);

            self.xr.swapchains.release_images();
            self.xr.frame_stream_end();
        }
    }
    fn draw_image() {}
    pub fn update_xr(&mut self) {
        self.xr.update();
    }
    pub fn compile_shaders(&mut self) {
        use shaders::*;
        println!("Compiling shaders...");
        let simple = glium::Program::from_source(
            &self.context,
            SHADER_SIMPLE_VERT,
            SHADER_SIMPLE_FRAG,
            None,
        )
        .unwrap();
        self.shaders.insert("simple".to_string(), simple);
    }
    pub fn load_default_models(&mut self) {
        use crate::obj_loader::load_obj;
        self.models.insert(
            "block".to_string(),
            load_obj("./assets/models/block.obj", &self.context),
        );
        self.models.insert(
            "cube".to_string(),
            load_obj("./assets/models/cube.obj", &self.context),
        );
    }
    pub fn load_default_textures(&mut self) {
        use crate::textures::load_texture;
        self.textures.insert(
            "dev".to_string(),
            load_texture("./assets/textures/dev.png".to_string(), &self.context),
        );
        self.textures.insert(
            "mine".to_string(),
            load_texture("./assets/textures/mine.png".to_string(), &self.context),
        );
        self.textures.insert(
            "note_red".to_string(),
            load_texture("./assets/textures/note_red.png".to_string(), &self.context),
        );
        self.textures.insert(
            "obstacle".to_string(),
            load_texture("./assets/textures/obstacle.png".to_string(), &self.context),
        );
        self.textures.insert(
            "note_blue".to_string(),
            load_texture("./assets/textures/note_blue.png".to_string(), &self.context),
        );
        self.textures.insert(
            "note_middle_red".to_string(),
            load_texture(
                "./assets/textures/note_middle_red.png".to_string(),
                &self.context,
            ),
        );
        self.textures.insert(
            "note_middle_blue".to_string(),
            load_texture(
                "./assets/textures/note_middle_blue.png".to_string(),
                &self.context,
            ),
        );
    }
}

#[derive(Copy, Clone)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub tex_coords: [f32; 2],
}
implement_vertex!(Vertex, position, normal, tex_coords);

pub fn calc_transform(
    scale: (f32, f32, f32),
    position: Translation3<f32>,
    rotation: UnitQuaternion<f32>,
) -> Matrix4<f32> {
    let scale_matrix: Matrix4<f32> = Matrix4::new(
        scale.0, 0.0, 0.0, 0.0, 0.0, scale.1, 0.0, 0.0, 0.0, 0.0, scale.2, 0.0, 0.0, 0.0, 0.0, 1.0,
    );
    let translation_matrix = position.to_homogeneous();
    let rotation_matrix = rotation.to_homogeneous();
    translation_matrix * rotation_matrix * scale_matrix
}
pub fn get_params() -> DrawParameters<'static> {
    use glium::{draw_parameters, Depth, DepthTest};
    DrawParameters {
        depth: Depth {
            test: DepthTest::IfLess,
            write: true,
            ..Default::default()
        },
        backface_culling: draw_parameters::BackfaceCullingMode::CullClockwise,
        blend: draw_parameters::Blend::alpha_blending(),
        ..Default::default()
    }
}
