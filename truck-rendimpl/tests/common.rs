#![allow(dead_code)]

use glsl_to_spirv::ShaderType;
use rayon::prelude::*;
use std::io::Read;
use std::sync::Arc;
use truck_platform::*;
use wgpu::*;

#[derive(Clone, Debug)]
pub struct Plane<'a> {
    pub vertex_shader: &'a str,
    pub fragment_shader: &'a str,
    pub id: RenderID,
}

#[macro_export]
macro_rules! new_plane {
    ($vertex_shader: expr, $fragment_shader: expr) => {
        Plane {
            vertex_shader: include_str!($vertex_shader),
            fragment_shader: include_str!($fragment_shader),
            id: RenderID::gen(),
        }
    };
}

impl<'a> Rendered for Plane<'a> {
    impl_render_id!(id);
    fn vertex_buffer(
        &self,
        handler: &DeviceHandler,
    ) -> (Arc<BufferHandler>, Option<Arc<BufferHandler>>) {
        let buffer = BufferHandler::from_slice(
            &[0 as u32, 1, 2, 2, 1, 3],
            handler.device(),
            BufferUsage::VERTEX,
        );
        (Arc::new(buffer), None)
    }
    fn bind_group_layout(&self, handler: &DeviceHandler) -> Arc<BindGroupLayout> {
        Arc::new(bind_group_util::create_bind_group_layout(
            handler.device(),
            &[],
        ))
    }
    fn bind_group(&self, handler: &DeviceHandler, layout: &BindGroupLayout) -> Arc<BindGroup> {
        Arc::new(handler.device().create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
            entries: &[],
        }))
    }
    fn pipeline(
        &self,
        handler: &DeviceHandler,
        layout: &PipelineLayout,
        sample_count: u32,
    ) -> Arc<RenderPipeline> {
        let (device, sc_desc) = (handler.device(), handler.sc_desc());
        let vertex_spirv = compile_shader(self.vertex_shader, ShaderType::Vertex);
        let vertex_module = device.create_shader_module(&ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(&vertex_spirv),
            flags: ShaderFlags::VALIDATION,
            label: None,
        });
        let fragment_spirv = compile_shader(self.fragment_shader, ShaderType::Fragment);
        let fragment_module = device.create_shader_module(&ShaderModuleDescriptor {
            source: wgpu::util::make_spirv(&fragment_spirv),
            flags: ShaderFlags::VALIDATION,
            label: None,
        });
        Arc::new(
            handler
                .device()
                .create_render_pipeline(&RenderPipelineDescriptor {
                    layout: Some(layout),
                    vertex: VertexState {
                        module: &vertex_module,
                        entry_point: "main",
                        buffers: &[VertexBufferLayout {
                            array_stride: std::mem::size_of::<u32>() as BufferAddress,
                            step_mode: InputStepMode::Vertex,
                            attributes: &[VertexAttribute {
                                format: VertexFormat::Uint,
                                offset: 0,
                                shader_location: 0,
                            }],
                        }],
                    },
                    primitive: PrimitiveState {
                        topology: PrimitiveTopology::TriangleList,
                        front_face: FrontFace::Ccw,
                        cull_mode: CullMode::None,
                        polygon_mode: PolygonMode::Fill,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                        clamp_depth: false,
                    }),
                    multisample: MultisampleState {
                        count: sample_count,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    fragment: Some(FragmentState {
                        module: &fragment_module,
                        entry_point: "main",
                        targets: &[ColorTargetState {
                            format: sc_desc.format,
                            color_blend: BlendState::REPLACE,
                            alpha_blend: BlendState::REPLACE,
                            write_mask: ColorWrite::ALL,
                        }],
                    }),
                    label: None,
                }),
        )
    }
}

pub fn render_one<R: Rendered>(scene: &mut Scene, texture: &Texture, object: &R) {
    scene.add_object(object);
    scene.render_scene(&texture.create_view(&Default::default()));
    scene.remove_object(object);
}

pub fn render_ones<'a, R: 'a + Rendered, I: IntoIterator<Item = &'a R>>(
    scene: &mut Scene,
    texture: &Texture,
    object: I,
) {
    scene.add_objects(object);
    scene.render_scene(&texture.create_view(&Default::default()));
    scene.clear_objects();
}

pub fn compile_shader(code: &str, shadertype: ShaderType) -> Vec<u8> {
    let mut spirv = glsl_to_spirv::compile(&code, shadertype).unwrap();
    let mut compiled = Vec::new();
    spirv.read_to_end(&mut compiled).unwrap();
    compiled
}

pub fn nontex_answer_texture(scene: &mut Scene) -> Texture {
    let sc_desc = scene.sc_desc();
    let tex_desc = texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut plane = new_plane!("shaders/plane.vert", "shaders/unicolor.frag");
    render_one(scene, &texture, &mut plane);
    texture
}

pub fn random_texture(scene: &mut Scene) -> Texture {
    let sc_desc = scene.sc_desc();
    let tex_desc = texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut plane = new_plane!("shaders/plane.vert", "shaders/random.frag");
    render_one(scene, &texture, &mut plane);
    texture
}

pub fn gradation_texture(scene: &mut Scene) -> Texture {
    let sc_desc = scene.sc_desc();
    let tex_desc = texture_descriptor(&sc_desc);
    let texture = scene.device().create_texture(&tex_desc);
    let mut plane = new_plane!("shaders/plane.vert", "shaders/gradation.frag");
    render_one(scene, &texture, &mut plane);
    texture
}

pub fn init_device(instance: &Instance) -> (Arc<Device>, Arc<Queue>) {
    futures::executor::block_on(async {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: None,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    features: Default::default(),
                    limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();
        (Arc::new(device), Arc::new(queue))
    })
}

pub fn swap_chain_descriptor(size: (u32, u32)) -> SwapChainDescriptor {
    SwapChainDescriptor {
        usage: TextureUsage::RENDER_ATTACHMENT,
        format: TextureFormat::Rgba8Unorm,
        width: size.0,
        height: size.1,
        present_mode: PresentMode::Mailbox,
    }
}

pub fn texture_descriptor(sc_desc: &SwapChainDescriptor) -> TextureDescriptor<'static> {
    TextureDescriptor {
        label: None,
        size: Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
        format: sc_desc.format,
        usage: TextureUsage::RENDER_ATTACHMENT | TextureUsage::COPY_SRC,
    }
}

pub fn texture_copy_view<'a>(texture: &'a Texture) -> TextureCopyView<'a> {
    TextureCopyView {
        texture: &texture,
        mip_level: 0,
        origin: Origin3d::ZERO,
    }
}

pub fn buffer_copy_view<'a>(buffer: &'a Buffer, size: (u32, u32)) -> BufferCopyView<'a> {
    BufferCopyView {
        buffer: &buffer,
        layout: TextureDataLayout {
            offset: 0,
            bytes_per_row: size.0 * 4,
            rows_per_image: size.1,
        },
    }
}

pub fn read_buffer(device: &Device, buffer: &Buffer) -> Vec<u8> {
    let buffer_slice = buffer.slice(..);
    let buffer_future = buffer_slice.map_async(MapMode::Read);
    device.poll(Maintain::Wait);
    futures::executor::block_on(async {
        match buffer_future.await {
            Ok(_) => buffer_slice.get_mapped_range().iter().map(|b| *b).collect(),
            Err(_) => panic!("failed to run compute on gpu!"),
        }
    })
}

pub fn read_texture(handler: &DeviceHandler, texture: &Texture) -> Vec<u8> {
    let (device, queue, sc_desc) = (handler.device(), handler.queue(), handler.sc_desc());
    let size = (sc_desc.width * sc_desc.height * 4) as u64;
    let buffer = device.create_buffer(&BufferDescriptor {
        label: None,
        mapped_at_creation: false,
        usage: BufferUsage::COPY_DST | BufferUsage::MAP_READ,
        size,
    });
    let mut encoder = device.create_command_encoder(&CommandEncoderDescriptor { label: None });
    encoder.copy_texture_to_buffer(
        texture_copy_view(&texture),
        buffer_copy_view(&buffer, (sc_desc.width, sc_desc.height)),
        Extent3d {
            width: sc_desc.width,
            height: sc_desc.height,
            depth: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    read_buffer(device, &buffer)
}

pub fn same_buffer(vec0: &Vec<u8>, vec1: &Vec<u8>) -> bool {
    vec0.par_iter()
        .zip(vec1)
        .all(move |(i, j)| std::cmp::max(i, j) - std::cmp::min(i, j) < 3)
}

pub fn count_difference(vec0: &Vec<u8>, vec1: &Vec<u8>) -> usize {
    vec0.par_iter()
        .zip(vec1)
        .filter(move |(i, j)| *std::cmp::max(i, j) - *std::cmp::min(i, j) > 2)
        .count()
}
