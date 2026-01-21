use std::{env, fs::File, io::Read, path::Path};

use wgpu::{self, ShaderModule, ShaderModuleDescriptor, ShaderSource};

pub fn load_shader(
    device: &wgpu::Device,
    shader_file_name: &str,
    shader_label: &str,
) -> ShaderModule {
    let shader_source_dir = env::var("SHADER_SOURCE_DIR").unwrap();
    let shader_path = Path::new(&shader_source_dir).join(shader_file_name);
    let mut shader_source_file = File::open(shader_path).unwrap();

    let mut shader_source_string = String::new();
    shader_source_file
        .read_to_string(&mut shader_source_string)
        .unwrap();

    device.create_shader_module(ShaderModuleDescriptor {
        label: Some(shader_label),
        source: ShaderSource::Wgsl(shader_source_string.into()),
    })
}
