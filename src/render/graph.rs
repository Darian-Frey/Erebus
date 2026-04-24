// Frame graph for Phase 1.
//
// Two passes for now:
//   1. nebula  — fullscreen triangle, debug fragment outputs UV gradient into
//                an HDR RGBA16F target. (Replaced in Phase 2 by the real
//                volumetric raymarch.)
//   2. composite — samples the HDR target, clamps to [0,1] and writes to the
//                  egui-provided surface. (Replaced in Phase 5 by the full
//                  exposure -> tonemap -> grade -> dither chain.)

use std::path::PathBuf;
use std::sync::Arc;

use bytemuck::bytes_of;

use crate::app::config::HDR_FORMAT;
use crate::render::context::shader_root;
use crate::render::hot_reload::ShaderWatcher;
use crate::render::resources::samplers::linear_clamp;
use crate::render::resources::textures::HdrTarget;
use crate::render::uniforms::FrameUniforms;

pub struct ErebusRenderer {
    device: Arc<wgpu::Device>,
    #[allow(dead_code)] // Held for future passes; current write goes via the prepare()-supplied queue.
    queue: Arc<wgpu::Queue>,
    surface_format: wgpu::TextureFormat,

    hdr: HdrTarget,
    sampler: wgpu::Sampler,

    frame_buffer: wgpu::Buffer,

    nebula_bgl: wgpu::BindGroupLayout,
    nebula_bg: wgpu::BindGroup,
    nebula_pipeline: wgpu::RenderPipeline,

    composite_bgl: wgpu::BindGroupLayout,
    composite_bg: wgpu::BindGroup,
    composite_pipeline: wgpu::RenderPipeline,

    watcher: ShaderWatcher,
    pub last_shader_error: Option<String>,
}

impl ErebusRenderer {
    pub fn new(state: &egui_wgpu::RenderState) -> anyhow::Result<Self> {
        let device = state.device.clone();
        let queue = state.queue.clone();
        let surface_format = state.target_format;

        let hdr = HdrTarget::new(&device, (1280, 800));
        let sampler = linear_clamp(&device);

        let frame_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("erebus.frame_uniforms"),
            size: std::mem::size_of::<FrameUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let nebula_bgl = create_nebula_bgl(&device);
        let composite_bgl = create_composite_bgl(&device);

        let (nebula_pipeline, composite_pipeline) =
            build_pipelines(&device, surface_format, &nebula_bgl, &composite_bgl)?;

        let nebula_bg = create_nebula_bg(&device, &nebula_bgl, &frame_buffer);
        let composite_bg = create_composite_bg(&device, &composite_bgl, &hdr.view, &sampler);

        let watcher = ShaderWatcher::new(shader_root())?;

        Ok(Self {
            device,
            queue,
            surface_format,
            hdr,
            sampler,
            frame_buffer,
            nebula_bgl,
            nebula_bg,
            nebula_pipeline,
            composite_bgl,
            composite_bg,
            composite_pipeline,
            watcher,
            last_shader_error: None,
        })
    }

    pub fn poll_hot_reload(&mut self) {
        if !self.watcher.poll() {
            return;
        }
        log::info!("shaders dirty — rebuilding pipelines");
        match build_pipelines(
            &self.device,
            self.surface_format,
            &self.nebula_bgl,
            &self.composite_bgl,
        ) {
            Ok((nebula, composite)) => {
                self.nebula_pipeline = nebula;
                self.composite_pipeline = composite;
                self.last_shader_error = None;
                log::info!("shader rebuild OK");
            }
            Err(e) => {
                log::error!("shader rebuild failed:\n{e}");
                self.last_shader_error = Some(format!("{e}"));
            }
        }
    }

    /// Resize the HDR offscreen target if the requested size differs.
    /// Rebuilds the composite bind group since the texture view changes.
    fn ensure_hdr_size(&mut self, size: (u32, u32)) {
        if size == self.hdr.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.hdr = HdrTarget::new(&self.device, size);
        self.composite_bg = create_composite_bg(
            &self.device,
            &self.composite_bgl,
            &self.hdr.view,
            &self.sampler,
        );
    }

    /// Encode the offscreen nebula pass into the supplied encoder.
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        uniforms: FrameUniforms,
        target_size: (u32, u32),
    ) {
        self.ensure_hdr_size(target_size);

        let mut u = uniforms;
        u.resolution = [self.hdr.size.0 as f32, self.hdr.size.1 as f32];
        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&u));

        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("erebus.pass.nebula"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.hdr.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&self.nebula_pipeline);
        pass.set_bind_group(0, &self.nebula_bg, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Sample the HDR target into the supplied egui-owned render pass.
    pub fn composite<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.composite_pipeline);
        pass.set_bind_group(0, &self.composite_bg, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn create_nebula_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.nebula"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}

fn create_nebula_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    frame: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("erebus.bg.nebula"),
        layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: frame.as_entire_binding(),
        }],
    })
}

fn create_composite_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.composite"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn create_composite_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("erebus.bg.composite"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(sampler),
            },
        ],
    })
}

fn build_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    nebula_bgl: &wgpu::BindGroupLayout,
    composite_bgl: &wgpu::BindGroupLayout,
) -> anyhow::Result<(wgpu::RenderPipeline, wgpu::RenderPipeline)> {
    let root = shader_root();
    let fullscreen_src = read_shader(&root.join("fullscreen.wgsl"))?;
    let nebula_src = read_shader(&root.join("nebula").join("raymarch.wgsl"))?;
    let composite_src = read_shader(&root.join("composite.wgsl"))?;

    validate(&fullscreen_src, "fullscreen.wgsl")?;
    validate(&nebula_src, "nebula/raymarch.wgsl")?;
    validate(&composite_src, "composite.wgsl")?;

    let fullscreen_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("fullscreen"),
        source: wgpu::ShaderSource::Wgsl(fullscreen_src.into()),
    });
    let nebula_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("nebula.raymarch"),
        source: wgpu::ShaderSource::Wgsl(nebula_src.into()),
    });
    let composite_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("composite"),
        source: wgpu::ShaderSource::Wgsl(composite_src.into()),
    });

    let nebula_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.nebula"),
        bind_group_layouts: &[nebula_bgl],
        push_constant_ranges: &[],
    });
    let composite_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.composite"),
        bind_group_layouts: &[composite_bgl],
        push_constant_ranges: &[],
    });

    let nebula = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.nebula"),
        layout: Some(&nebula_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &nebula_mod,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });

    let composite = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.composite"),
        layout: Some(&composite_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: "vs_main",
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &composite_mod,
            entry_point: "fs_main",
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    });

    Ok((nebula, composite))
}

fn read_shader(path: &PathBuf) -> anyhow::Result<String> {
    std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))
}

fn validate(src: &str, name: &str) -> anyhow::Result<()> {
    // Naga front-end gives us source-located error messages before the
    // shader hits the device — much nicer than a wgpu validation panic.
    wgpu::naga::front::wgsl::parse_str(src)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("{name}:\n{}", e.emit_to_string(src)))
}
