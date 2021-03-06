#[macro_use]
extern crate glium;
#[macro_use]
extern crate specs_derive;

mod components;
mod obj_loader;
mod openxr_module;
mod parser;
mod render;
mod textures;

pub static SCALE: f32 = 5.0;

fn main() {
    let mut window = render::Window::new();

    'main: loop {
        let frame = window.draw();
        window.update_xr();
    }
}
