// Frame graph (Phase 5).
//
// Per-frame ordering:
//   1. Optional compute bake (3D noise volume — only when seed/octaves dirty).
//   2. Nebula raymarch (+ inline starfield) → HDR target. Clear.
//   3. Bloom downsample chain: HDR target → bloom.mip[0] (first pass with
//      threshold + Karis), then bloom.mip[i-1] → bloom.mip[i] for i = 1..N.
//   4. Bloom upsample chain: bloom.mip[i] → bloom.mip[i-1] additively, for
//      i = N-1 down to 1. Final result lives in bloom.mip[0].
//   5. Tonemap pass: read HDR + bloom.mip[0], apply exposure, grade, AgX/ACES/
//      Reinhard tonemap, triangular dither → swapchain.

use std::sync::Arc;

use bytemuck::bytes_of;
use web_time::Instant;

use crate::app::config::HDR_FORMAT;
#[cfg(not(target_arch = "wasm32"))]
use crate::render::context::shader_root;
use crate::render::hot_reload::ShaderWatcher;
use crate::render::resources::samplers::linear_clamp;
use crate::render::resources::textures::{
    BlackbodyLut, BloomPyramid, GradientLut, HdrTarget, NoiseVolume,
};
use crate::render::uniforms::{
    BakeUniforms, FrameUniforms, LightingUniforms, NebulaUniforms, PostUniforms,
    StarfieldUniforms, RENDER_MODE_CUBEMAP,
};

pub struct ErebusRenderer {
    device: Arc<wgpu::Device>,
    #[allow(dead_code)]
    queue: Arc<wgpu::Queue>,
    surface_format: wgpu::TextureFormat,

    hdr: HdrTarget,
    sampler: wgpu::Sampler,
    bloom: BloomPyramid,

    #[allow(dead_code)]
    gradient: GradientLut,
    #[allow(dead_code)]
    gradient_sampler: wgpu::Sampler,
    #[allow(dead_code)]
    noise_volume: NoiseVolume,
    #[allow(dead_code)]
    noise_sampler: wgpu::Sampler,
    #[allow(dead_code)]
    blackbody: BlackbodyLut,
    #[allow(dead_code)]
    blackbody_sampler: wgpu::Sampler,

    frame_buffer: wgpu::Buffer,
    nebula_buffer: wgpu::Buffer,
    lighting_buffer: wgpu::Buffer,
    starfield_buffer: wgpu::Buffer,
    post_buffer: wgpu::Buffer,
    bake_buffer: wgpu::Buffer,

    nebula_bgl: wgpu::BindGroupLayout,
    nebula_bg: wgpu::BindGroup,
    nebula_pipeline: wgpu::RenderPipeline,

    bake_bgl: wgpu::BindGroupLayout,
    bake_bg: wgpu::BindGroup,
    bake_pipeline: wgpu::ComputePipeline,
    last_bake: Option<BakeUniforms>,

    bloom_bgl: wgpu::BindGroupLayout,
    bloom_downsample_first_pipeline: wgpu::RenderPipeline,
    bloom_downsample_pipeline: wgpu::RenderPipeline,
    bloom_upsample_pipeline: wgpu::RenderPipeline,
    bloom_downsample_bgs: Vec<wgpu::BindGroup>, // len = mip_count
    bloom_upsample_bgs: Vec<wgpu::BindGroup>,   // len = mip_count - 1

    tonemap_bgl: wgpu::BindGroupLayout,
    tonemap_bg: wgpu::BindGroup,
    tonemap_pipeline: wgpu::RenderPipeline,

    // Phase 6: dedicated tonemap pipeline targeting Rgba8UnormSrgb so the
    // exported texture can be read back directly as sRGB-encoded PNG bytes
    // without per-pixel format swizzling.
    export_tonemap_pipeline: wgpu::RenderPipeline,

    // Phase 6.5: linear HDR pipeline targeting Rgba16Float, used by EXR
    // export. Same composite shader, but `tonemap_mode` is forced to 3
    // (passthrough) at dispatch time so on-disk values stay scene-referred.
    export_linear_pipeline: wgpu::RenderPipeline,

    watcher: ShaderWatcher,
    pub last_shader_error: Option<String>,
}

impl ErebusRenderer {
    pub fn new(state: &egui_wgpu::RenderState) -> anyhow::Result<Self> {
        let device = state.device.clone();
        let queue = state.queue.clone();
        let surface_format = state.target_format;

        let hdr = HdrTarget::new(&device, (1280, 800));
        let bloom = BloomPyramid::new(&device, hdr.size);
        let sampler = linear_clamp(&device);

        let gradient = GradientLut::new(&device, &queue);
        let gradient_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("erebus.sampler.gradient"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let noise_volume = NoiseVolume::new(&device);
        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("erebus.sampler.noise"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let blackbody = BlackbodyLut::new(&device, &queue);
        let blackbody_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("erebus.sampler.blackbody"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let frame_buffer = uniform_buf(&device, "frame", std::mem::size_of::<FrameUniforms>());
        let nebula_buffer = uniform_buf(&device, "nebula", std::mem::size_of::<NebulaUniforms>());
        let lighting_buffer =
            uniform_buf(&device, "lighting", std::mem::size_of::<LightingUniforms>());
        let starfield_buffer = uniform_buf(
            &device,
            "starfield",
            std::mem::size_of::<StarfieldUniforms>(),
        );
        let post_buffer = uniform_buf(&device, "post", std::mem::size_of::<PostUniforms>());
        let bake_buffer = uniform_buf(&device, "bake", std::mem::size_of::<BakeUniforms>());

        let nebula_bgl = create_nebula_bgl(&device);
        let bake_bgl = create_bake_bgl(&device);
        let bloom_bgl = create_bloom_bgl(&device);
        let tonemap_bgl = create_tonemap_bgl(&device);

        let pipelines = build_pipelines(
            &device,
            surface_format,
            &nebula_bgl,
            &bake_bgl,
            &bloom_bgl,
            &tonemap_bgl,
        )?;

        let nebula_bg = create_nebula_bg(
            &device,
            &nebula_bgl,
            &frame_buffer,
            &nebula_buffer,
            &lighting_buffer,
            &starfield_buffer,
            &gradient.view,
            &gradient_sampler,
            &noise_volume.view,
            &noise_sampler,
            &blackbody.view,
            &blackbody_sampler,
        );
        let bake_bg = create_bake_bg(&device, &bake_bgl, &bake_buffer, &noise_volume.view);
        let bloom_downsample_bgs =
            build_bloom_downsample_bgs(&device, &bloom_bgl, &hdr.view, &bloom, &sampler, &post_buffer);
        let bloom_upsample_bgs =
            build_bloom_upsample_bgs(&device, &bloom_bgl, &bloom, &sampler, &post_buffer);
        let tonemap_bg = create_tonemap_bg(
            &device,
            &tonemap_bgl,
            &hdr.view,
            &bloom.mips[0],
            &sampler,
            &post_buffer,
        );

        // ShaderWatcher takes a path so the API stays uniform; the WASM
        // stub ignores it and `shader_root()` is only defined on native.
        #[cfg(not(target_arch = "wasm32"))]
        let watcher = ShaderWatcher::new(shader_root())?;
        #[cfg(target_arch = "wasm32")]
        let watcher = ShaderWatcher::new(std::path::PathBuf::new())?;

        Ok(Self {
            device,
            queue,
            surface_format,
            hdr,
            sampler,
            bloom,
            gradient,
            gradient_sampler,
            noise_volume,
            noise_sampler,
            blackbody,
            blackbody_sampler,
            frame_buffer,
            nebula_buffer,
            lighting_buffer,
            starfield_buffer,
            post_buffer,
            bake_buffer,
            nebula_bgl,
            nebula_bg,
            nebula_pipeline: pipelines.nebula,
            bake_bgl,
            bake_bg,
            bake_pipeline: pipelines.bake,
            last_bake: None,
            bloom_bgl,
            bloom_downsample_first_pipeline: pipelines.bloom_downsample_first,
            bloom_downsample_pipeline: pipelines.bloom_downsample,
            bloom_upsample_pipeline: pipelines.bloom_upsample,
            bloom_downsample_bgs,
            bloom_upsample_bgs,
            tonemap_bgl,
            tonemap_bg,
            tonemap_pipeline: pipelines.tonemap,
            export_tonemap_pipeline: pipelines.export_tonemap,
            export_linear_pipeline: pipelines.export_linear,
            watcher,
            last_shader_error: None,
        })
    }

    /// Returns true if a rebuild fired (regardless of success). Callers use
    /// the bool to invalidate any frame caches keyed on shader output.
    pub fn poll_hot_reload(&mut self) -> bool {
        if !self.watcher.poll() {
            return false;
        }
        log::info!("shaders dirty — rebuilding pipelines");
        match build_pipelines(
            &self.device,
            self.surface_format,
            &self.nebula_bgl,
            &self.bake_bgl,
            &self.bloom_bgl,
            &self.tonemap_bgl,
        ) {
            Ok(p) => {
                self.nebula_pipeline = p.nebula;
                self.bake_pipeline = p.bake;
                self.bloom_downsample_first_pipeline = p.bloom_downsample_first;
                self.bloom_downsample_pipeline = p.bloom_downsample;
                self.bloom_upsample_pipeline = p.bloom_upsample;
                self.tonemap_pipeline = p.tonemap;
                self.export_tonemap_pipeline = p.export_tonemap;
                self.export_linear_pipeline = p.export_linear;
                self.last_bake = None;
                self.last_shader_error = None;
                log::info!("shader rebuild OK");
            }
            Err(e) => {
                log::error!("shader rebuild failed:\n{e}");
                self.last_shader_error = Some(format!("{e}"));
            }
        }
        true
    }

    /// Resize HDR target + bloom pyramid + per-mip bind groups when the
    /// requested size differs.
    fn ensure_hdr_size(&mut self, size: (u32, u32)) {
        if size == self.hdr.size || size.0 == 0 || size.1 == 0 {
            return;
        }
        self.hdr = HdrTarget::new(&self.device, size);
        self.bloom = BloomPyramid::new(&self.device, size);
        self.bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &self.hdr.view,
            &self.bloom,
            &self.sampler,
            &self.post_buffer,
        );
        self.bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &self.bloom,
            &self.sampler,
            &self.post_buffer,
        );
        self.tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &self.hdr.view,
            &self.bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );
    }

    /// Encode every offscreen pass into the supplied encoder. The final
    /// Re-upload the `PostUniforms` UBO without doing any of the offscreen
    /// work. Used by the skip_prepare path so skybox camera changes
    /// (yaw/pitch/fov) reach the composite pass even when the offscreen
    /// render is frozen. `resolution` is overridden to the cached HDR size,
    /// matching the convention used by `prepare()`.
    pub fn upload_post(&self, queue: &wgpu::Queue, post: PostUniforms) {
        let mut p = post;
        p.resolution = [self.hdr.size.0 as f32, self.hdr.size.1 as f32];
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));
    }

    /// tonemap pass is encoded later via `composite()` into the egui-owned
    /// render pass that already targets the swapchain.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
        target_size: (u32, u32),
    ) {
        self.ensure_hdr_size(target_size);

        let mut f = frame;
        f.resolution = [self.hdr.size.0 as f32, self.hdr.size.1 as f32];
        let mut p = post;
        p.resolution = [self.hdr.size.0 as f32, self.hdr.size.1 as f32];

        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        // 1. Optional compute bake.
        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
            let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("erebus.pass.bake_3d_noise"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&self.bake_pipeline);
            compute.set_bind_group(0, &self.bake_bg, &[]);
            compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
            drop(compute);
            self.last_bake = Some(want_bake);
        }

        // 2. Nebula + starfield → HDR.
        {
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

        // 3. Bloom downsample chain. Skipped when the user has dialed bloom
        // off — each pass costs ~40 ms of per-pass overhead in browser WebGPU
        // on integrated GPUs, so 9 bloom passes can dominate the frame budget
        // when their visual contribution is zero anyway. The composite pass
        // multiplies by `bloom_intensity` so the stale mip data is harmless.
        if post.bloom_intensity <= 1.0e-4 {
            return;
        }
        let mip_count = self.bloom.mip_count() as usize;
        for i in 0..mip_count {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.pass.bloom.down"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom.mips[i],
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
            if i == 0 {
                pass.set_pipeline(&self.bloom_downsample_first_pipeline);
            } else {
                pass.set_pipeline(&self.bloom_downsample_pipeline);
            }
            pass.set_bind_group(0, &self.bloom_downsample_bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }

        // 4. Bloom upsample chain — additive blend back up into mip 0.
        for i in (0..(mip_count - 1)).rev() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.pass.bloom.up"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.bloom.mips[i],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.bloom_upsample_pipeline);
            pass.set_bind_group(0, &self.bloom_upsample_bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }
    }

    /// Re-bake the 256-texel gradient LUT from the supplied stops. Called
    /// once at startup and whenever the user loads a preset (or, eventually,
    /// edits a stop in the gradient widget).
    pub fn update_gradient(&self, queue: &wgpu::Queue, stops: &[crate::render::GradientStop]) {
        self.gradient.upload(queue, stops);
    }

    /// Final tonemap pass into the egui-owned render pass.
    pub fn composite(&self, pass: &mut wgpu::RenderPass<'static>) {
        pass.set_pipeline(&self.tonemap_pipeline);
        pass.set_bind_group(0, &self.tonemap_bg, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Render the current scene at `(width, height)` and return the
    /// tonemapped sRGB-encoded RGBA8 pixels. Allocates a fresh set of
    /// HDR / bloom / output / readback resources sized for this single
    /// shot — does NOT touch the live preview targets.
    ///
    /// Synchronous: `device.poll(Wait)` blocks until the readback finishes.
    /// At 8K equirect this is typically 1–3 seconds on a discrete GPU.
    #[allow(clippy::too_many_arguments)]
    pub fn render_equirect_rgba8(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
    ) -> anyhow::Result<Vec<u8>> {
        anyhow::ensure!(width > 0 && height > 0, "export size must be non-zero");

        // Allocate one-shot export resources at the requested resolution.
        let export_hdr = HdrTarget::new(&self.device, (width, height));
        let export_bloom = BloomPyramid::new(&self.device, (width, height));

        let export_output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.export.output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EXPORT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let export_output_view =
            export_output.create_view(&wgpu::TextureViewDescriptor::default());

        // wgpu requires per-row alignment of 256 bytes for buffer copies.
        let unpadded_row_bytes = width * 4;
        let row_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = unpadded_row_bytes.div_ceil(row_align) * row_align;
        let buffer_size = (padded_row_bytes as u64) * (height as u64);
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("erebus.export.readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // The nebula bind group shares its layout with the live preview, so we
        // reuse the existing one (uniforms + LUTs + noise volume don't depend
        // on the export target). Bloom + tonemap need new bind groups that
        // reference the export targets.
        let bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_hdr.view,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &export_hdr.view,
            &export_bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );

        // Push the per-frame uniforms — same as the live `prepare()` pass,
        // but with `resolution` set to the export target size.
        let mut f = frame;
        f.resolution = [width as f32, height as f32];
        let mut p = post;
        p.resolution = [width as f32, height as f32];
        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("erebus.export.encoder"),
            });

        // Re-bake noise if dirty. Same logic as the live path.
        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
            let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("erebus.export.bake_3d_noise"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&self.bake_pipeline);
            compute.set_bind_group(0, &self.bake_bg, &[]);
            compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
            drop(compute);
            self.last_bake = Some(want_bake);
        }

        // Nebula → export HDR.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.nebula"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_hdr.view,
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

        // Bloom downsample chain.
        let mip_count = export_bloom.mip_count() as usize;
        for i in 0..mip_count {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.bloom.down"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_bloom.mips[i],
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
            if i == 0 {
                pass.set_pipeline(&self.bloom_downsample_first_pipeline);
            } else {
                pass.set_pipeline(&self.bloom_downsample_pipeline);
            }
            pass.set_bind_group(0, &bloom_downsample_bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }

        // Bloom upsample chain.
        if mip_count > 1 {
            for i in (0..(mip_count - 1)).rev() {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("erebus.export.pass.bloom.up"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &export_bloom.mips[i],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bloom_upsample_pipeline);
                pass.set_bind_group(0, &bloom_upsample_bgs[i], &[]);
                pass.draw(0..3, 0..1);
            }
        }

        // Tonemap → export output (Rgba8UnormSrgb).
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_output_view,
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
            pass.set_pipeline(&self.export_tonemap_pipeline);
            pass.set_bind_group(0, &tonemap_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        // Texture → mappable buffer.
        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &export_output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        log::info!("export: queue.submit");
        queue.submit(Some(encoder.finish()));
        log::info!("export: queue.submit returned");

        // Map + block until the GPU has finished writing.
        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<(), wgpu::BufferAsyncError>>(1);
        log::info!("export: map_async issued");
        slice.map_async(wgpu::MapMode::Read, move |res| {
            log::info!("export: map_async callback fired");
            let _ = tx.send(res);
        });
        log::info!("export: device.poll(Wait) entering");
        self.device.poll(wgpu::Maintain::Wait);
        log::info!("export: device.poll returned, rx.recv entering");
        rx.recv()
            .map_err(|e| anyhow::anyhow!("readback channel: {e}"))?
            .map_err(|e| anyhow::anyhow!("buffer map: {e:?}"))?;
        log::info!("export: rx.recv returned");

        // Strip per-row padding into a tightly packed RGBA buffer.
        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity((unpadded_row_bytes as usize) * (height as usize));
        for row in 0..height {
            let row_start = (row as usize) * (padded_row_bytes as usize);
            let row_end = row_start + (unpadded_row_bytes as usize);
            out.extend_from_slice(&mapped[row_start..row_end]);
        }
        drop(mapped);
        readback.unmap();
        log::info!("export: unmap done, returning {} bytes", out.len());

        Ok(out)
    }

    /// Render six cube faces (+X, -X, +Y, -Y, +Z, -Z) at `face_size`²
    /// resolution and return their tonemapped sRGB RGBA8 pixels. Each face
    /// uses its own readback buffer; one `device.poll(Wait)` at the end
    /// blocks until all six have finished.
    #[allow(clippy::too_many_arguments)]
    pub fn render_cubemap_rgba8(
        &mut self,
        queue: &wgpu::Queue,
        face_size: u32,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
    ) -> anyhow::Result<[Vec<u8>; 6]> {
        anyhow::ensure!(face_size > 0, "face size must be non-zero");
        let size = (face_size, face_size);

        let export_hdr = HdrTarget::new(&self.device, size);
        let export_bloom = BloomPyramid::new(&self.device, size);

        let export_output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.export.cubemap.output"),
            size: wgpu::Extent3d {
                width: face_size,
                height: face_size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EXPORT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let export_output_view =
            export_output.create_view(&wgpu::TextureViewDescriptor::default());

        let unpadded_row_bytes = face_size * 4;
        let row_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = unpadded_row_bytes.div_ceil(row_align) * row_align;
        let buffer_size = (padded_row_bytes as u64) * (face_size as u64);

        let readbacks: Vec<wgpu::Buffer> = (0..6)
            .map(|i| {
                self.device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some(&format!("erebus.export.cubemap.readback.{i}")),
                    size: buffer_size,
                    usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            })
            .collect();

        let bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_hdr.view,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &export_hdr.view,
            &export_bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );

        // Static uniforms — same across all six faces.
        let mut p = post;
        p.resolution = [face_size as f32, face_size as f32];
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        // Bake check — runs at most once even though we'll render six times.
        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
        }

        for face in 0..6u32 {
            let mut f = frame;
            f.resolution = [face_size as f32, face_size as f32];
            f.mode = RENDER_MODE_CUBEMAP;
            f.cube_face = face;
            queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));

            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some(&format!("erebus.export.cubemap.face{face}")),
                });

            // Bake on the first face only; subsequent faces reuse the volume.
            if face == 0 && needs_bake {
                let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("erebus.export.cubemap.bake"),
                    timestamp_writes: None,
                });
                compute.set_pipeline(&self.bake_pipeline);
                compute.set_bind_group(0, &self.bake_bg, &[]);
                compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
                drop(compute);
                self.last_bake = Some(want_bake);
            }

            encode_pipeline_pass(
                &mut encoder,
                &self.nebula_pipeline,
                &self.nebula_bg,
                &self.bloom_downsample_first_pipeline,
                &self.bloom_downsample_pipeline,
                &self.bloom_upsample_pipeline,
                &self.export_tonemap_pipeline,
                &tonemap_bg,
                &export_hdr,
                &export_bloom,
                &export_output_view,
                &bloom_downsample_bgs,
                &bloom_upsample_bgs,
            );

            encoder.copy_texture_to_buffer(
                wgpu::ImageCopyTexture {
                    texture: &export_output,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                wgpu::ImageCopyBuffer {
                    buffer: &readbacks[face as usize],
                    layout: wgpu::ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(padded_row_bytes),
                        rows_per_image: Some(face_size),
                    },
                },
                wgpu::Extent3d {
                    width: face_size,
                    height: face_size,
                    depth_or_array_layers: 1,
                },
            );

            queue.submit(Some(encoder.finish()));
        }

        // Map all six readbacks; one Wait covers them all.
        for (i, rb) in readbacks.iter().enumerate() {
            let slice = rb.slice(..);
            slice.map_async(wgpu::MapMode::Read, move |res| {
                if let Err(e) = res {
                    log::error!("cube face {i} map failed: {e:?}");
                }
            });
        }
        self.device.poll(wgpu::Maintain::Wait);

        // Strip per-row padding into tightly packed RGBA buffers.
        let mut faces: [Vec<u8>; 6] = Default::default();
        for (i, rb) in readbacks.iter().enumerate() {
            let mapped = rb.slice(..).get_mapped_range();
            let mut out = Vec::with_capacity((unpadded_row_bytes as usize) * (face_size as usize));
            for row in 0..face_size {
                let row_start = (row as usize) * (padded_row_bytes as usize);
                let row_end = row_start + (unpadded_row_bytes as usize);
                out.extend_from_slice(&mapped[row_start..row_end]);
            }
            drop(mapped);
            rb.unmap();
            faces[i] = out;
        }

        Ok(faces)
    }

    /// Render an equirect at `(width, height)` and return raw linear-space
    /// `f32` RGBA pixels — bypasses the display-tonemap by forcing
    /// `tonemap_mode = 3` so the on-disk EXR contains scene-referred radiance.
    /// Bloom is still applied (so the EXR matches the look you'd see in
    /// preview if you cranked exposure into the same range).
    #[allow(clippy::too_many_arguments)]
    pub fn render_equirect_rgba32f(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
    ) -> anyhow::Result<Vec<f32>> {
        anyhow::ensure!(width > 0 && height > 0, "export size must be non-zero");
        let size = (width, height);

        let export_hdr = HdrTarget::new(&self.device, size);
        let export_bloom = BloomPyramid::new(&self.device, size);

        let export_output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.export.linear.output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EXPORT_LINEAR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let export_output_view =
            export_output.create_view(&wgpu::TextureViewDescriptor::default());

        // RGBA16F = 8 bytes per pixel.
        let unpadded_row_bytes = width * 8;
        let row_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = unpadded_row_bytes.div_ceil(row_align) * row_align;
        let buffer_size = (padded_row_bytes as u64) * (height as u64);
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("erebus.export.linear.readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_hdr.view,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &export_hdr.view,
            &export_bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );

        // Force linear passthrough + disable deband; everything else stays
        // user-controlled.
        let mut p = post;
        p.resolution = [width as f32, height as f32];
        p.tonemap_mode = 3;
        p.deband_amount = 0.0;

        let mut f = frame;
        f.resolution = [width as f32, height as f32];

        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
        }

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("erebus.export.linear.encoder"),
            });
        if needs_bake {
            let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("erebus.export.linear.bake"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&self.bake_pipeline);
            compute.set_bind_group(0, &self.bake_bg, &[]);
            compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
            drop(compute);
            self.last_bake = Some(want_bake);
        }

        encode_pipeline_pass(
            &mut encoder,
            &self.nebula_pipeline,
            &self.nebula_bg,
            &self.bloom_downsample_first_pipeline,
            &self.bloom_downsample_pipeline,
            &self.bloom_upsample_pipeline,
            &self.export_linear_pipeline,
            &tonemap_bg,
            &export_hdr,
            &export_bloom,
            &export_output_view,
            &bloom_downsample_bgs,
            &bloom_upsample_bgs,
        );

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &export_output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        queue.submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<(), wgpu::BufferAsyncError>>(1);
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv()
            .map_err(|e| anyhow::anyhow!("readback channel: {e}"))?
            .map_err(|e| anyhow::anyhow!("buffer map: {e:?}"))?;

        let mapped = slice.get_mapped_range();
        let mut out = Vec::with_capacity((width as usize) * (height as usize) * 4);
        for row in 0..height {
            let row_start = (row as usize) * (padded_row_bytes as usize);
            let row_end = row_start + (unpadded_row_bytes as usize);
            let row_bytes = &mapped[row_start..row_end];
            // 4 channels × 2 bytes each, little-endian f16.
            for px in row_bytes.chunks_exact(8) {
                for half in px.chunks_exact(2) {
                    let bits = u16::from_le_bytes([half[0], half[1]]);
                    out.push(f16_bits_to_f32(bits));
                }
            }
        }
        drop(mapped);
        readback.unmap();

        Ok(out)
    }

    /// Run the full nebula → bloom → tonemap pipeline `warmup + runs` times
    /// at the supplied resolution, return the **median** wall-clock ms over
    /// the measured frames. No readback — we only time the GPU pipeline,
    /// not the CPU↔GPU memory transfer.
    ///
    /// `nebula.steps` is the dominant cost driver; pass different values per
    /// call to characterise the step-count ↔ ms-frame curve.
    #[allow(clippy::too_many_arguments)]
    pub fn bench_render(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        warmup: u32,
        runs: u32,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
    ) -> anyhow::Result<f32> {
        anyhow::ensure!(width > 0 && height > 0 && runs > 0, "invalid bench size");

        let bench_hdr = HdrTarget::new(&self.device, (width, height));
        let bench_bloom = BloomPyramid::new(&self.device, (width, height));
        let bench_output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.bench.output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EXPORT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let bench_output_view =
            bench_output.create_view(&wgpu::TextureViewDescriptor::default());

        let bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &bench_hdr.view,
            &bench_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &bench_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &bench_hdr.view,
            &bench_bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );

        let mut f = frame;
        f.resolution = [width as f32, height as f32];
        let mut p = post;
        p.resolution = [width as f32, height as f32];
        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        // Bake once before we start timing so warmup frames don't pay the
        // bake cost (which isn't representative of the per-frame steady state).
        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("erebus.bench.bake"),
                });
            {
                let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("erebus.bench.bake_3d_noise"),
                    timestamp_writes: None,
                });
                compute.set_pipeline(&self.bake_pipeline);
                compute.set_bind_group(0, &self.bake_bg, &[]);
                compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
            }
            queue.submit(Some(encoder.finish()));
            self.device.poll(wgpu::Maintain::Wait);
            self.last_bake = Some(want_bake);
        }

        let run_one = |label: &'static str| -> f32 {
            let started = Instant::now();
            let mut encoder = self
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });
            encode_pipeline_pass(
                &mut encoder,
                &self.nebula_pipeline,
                &self.nebula_bg,
                &self.bloom_downsample_first_pipeline,
                &self.bloom_downsample_pipeline,
                &self.bloom_upsample_pipeline,
                &self.export_tonemap_pipeline,
                &tonemap_bg,
                &bench_hdr,
                &bench_bloom,
                &bench_output_view,
                &bloom_downsample_bgs,
                &bloom_upsample_bgs,
            );
            queue.submit(Some(encoder.finish()));
            self.device.poll(wgpu::Maintain::Wait);
            started.elapsed().as_secs_f64() as f32 * 1000.0
        };

        for _ in 0..warmup {
            let _ = run_one("erebus.bench.warmup");
        }
        let mut samples: Vec<f32> = (0..runs).map(|_| run_one("erebus.bench.measure")).collect();
        samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        Ok(samples[samples.len() / 2])
    }
}

// -------- Wasm-only async export --------------------------------------------
//
// Browsers can't run a synchronous `device.poll(Wait)` — the readback callback
// fires through the JS event loop, which is blocked while we're inside
// `update()`. So the export is split across multiple frames: frame N submits
// the GPU work + issues `map_async`, then each subsequent frame polls the
// device and checks whether the callback has delivered. Once it has, the
// caller reads the mapped buffer, encodes PNG, and triggers a download.

/// In-flight export job. Holds the readback buffer (the GPU writes the final
/// pixels here) and a non-blocking receiver that the map-async callback uses
/// to signal completion. Lives in `State` between frames.
#[cfg(target_arch = "wasm32")]
pub struct PendingExport {
    pub width: u32,
    pub height: u32,
    pub padded_row_bytes: u32,
    pub unpadded_row_bytes: u32,
    pub readback: wgpu::Buffer,
    pub rx: std::sync::mpsc::Receiver<Result<(), wgpu::BufferAsyncError>>,
}

#[cfg(target_arch = "wasm32")]
impl ErebusRenderer {
    /// Wasm export step 1: submit all GPU work + queue the readback. Returns
    /// a `PendingExport` that the caller stores in `State` and polls via
    /// `try_finish_export` on subsequent frames.
    #[allow(clippy::too_many_arguments)]
    pub fn submit_equirect_export(
        &mut self,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        frame: FrameUniforms,
        nebula: NebulaUniforms,
        lighting: LightingUniforms,
        starfield: StarfieldUniforms,
        post: PostUniforms,
    ) -> anyhow::Result<PendingExport> {
        anyhow::ensure!(width > 0 && height > 0, "export size must be non-zero");

        let export_hdr = HdrTarget::new(&self.device, (width, height));
        let export_bloom = BloomPyramid::new(&self.device, (width, height));

        let export_output = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.export.output"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: EXPORT_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let export_output_view =
            export_output.create_view(&wgpu::TextureViewDescriptor::default());

        let unpadded_row_bytes = width * 4;
        let row_align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let padded_row_bytes = unpadded_row_bytes.div_ceil(row_align) * row_align;
        let buffer_size = (padded_row_bytes as u64) * (height as u64);
        let readback = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("erebus.export.readback"),
            size: buffer_size,
            usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bloom_downsample_bgs = build_bloom_downsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_hdr.view,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let bloom_upsample_bgs = build_bloom_upsample_bgs(
            &self.device,
            &self.bloom_bgl,
            &export_bloom,
            &self.sampler,
            &self.post_buffer,
        );
        let tonemap_bg = create_tonemap_bg(
            &self.device,
            &self.tonemap_bgl,
            &export_hdr.view,
            &export_bloom.mips[0],
            &self.sampler,
            &self.post_buffer,
        );

        let mut f = frame;
        f.resolution = [width as f32, height as f32];
        let mut p = post;
        p.resolution = [width as f32, height as f32];
        queue.write_buffer(&self.frame_buffer, 0, bytes_of(&f));
        queue.write_buffer(&self.nebula_buffer, 0, bytes_of(&nebula));
        queue.write_buffer(&self.lighting_buffer, 0, bytes_of(&lighting));
        queue.write_buffer(&self.starfield_buffer, 0, bytes_of(&starfield));
        queue.write_buffer(&self.post_buffer, 0, bytes_of(&p));

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("erebus.export.encoder"),
                });

        let want_bake = BakeUniforms {
            seed: frame.seed,
            octaves: nebula.octaves_density,
            lacunarity: nebula.lacunarity,
            gain: nebula.gain,
        };
        let needs_bake = match self.last_bake {
            None => true,
            Some(prev) => prev.differs(&want_bake),
        };
        if needs_bake {
            queue.write_buffer(&self.bake_buffer, 0, bytes_of(&want_bake));
            let mut compute = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("erebus.export.bake_3d_noise"),
                timestamp_writes: None,
            });
            compute.set_pipeline(&self.bake_pipeline);
            compute.set_bind_group(0, &self.bake_bg, &[]);
            compute.dispatch_workgroups(
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
    NoiseVolume::DISPATCH_PER_AXIS,
);
            drop(compute);
            self.last_bake = Some(want_bake);
        }

        // Nebula → export HDR.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.nebula"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_hdr.view,
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

        // Bloom downsample chain.
        let mip_count = export_bloom.mip_count() as usize;
        for i in 0..mip_count {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.bloom.down"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_bloom.mips[i],
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
            if i == 0 {
                pass.set_pipeline(&self.bloom_downsample_first_pipeline);
            } else {
                pass.set_pipeline(&self.bloom_downsample_pipeline);
            }
            pass.set_bind_group(0, &bloom_downsample_bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }

        // Bloom upsample chain.
        if mip_count > 1 {
            for i in (0..(mip_count - 1)).rev() {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("erebus.export.pass.bloom.up"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &export_bloom.mips[i],
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                pass.set_pipeline(&self.bloom_upsample_pipeline);
                pass.set_bind_group(0, &bloom_upsample_bgs[i], &[]);
                pass.draw(0..3, 0..1);
            }
        }

        // Tonemap → export output.
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.export.pass.tonemap"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &export_output_view,
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
            pass.set_pipeline(&self.export_tonemap_pipeline);
            pass.set_bind_group(0, &tonemap_bg, &[]);
            pass.draw(0..3, 0..1);
        }

        encoder.copy_texture_to_buffer(
            wgpu::ImageCopyTexture {
                texture: &export_output,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::ImageCopyBuffer {
                buffer: &readback,
                layout: wgpu::ImageDataLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row_bytes),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        let slice = readback.slice(..);
        let (tx, rx) = std::sync::mpsc::sync_channel::<Result<(), wgpu::BufferAsyncError>>(1);
        slice.map_async(wgpu::MapMode::Read, move |res| {
            let _ = tx.send(res);
        });

        Ok(PendingExport {
            width,
            height,
            padded_row_bytes,
            unpadded_row_bytes,
            readback,
            rx,
        })
    }

    /// Drive wgpu's internal state machine. Must be called every frame while
    /// an export is in flight so `map_async` callbacks have a chance to fire.
    pub fn poll_export_progress(&self) {
        self.device.poll(wgpu::Maintain::Poll);
    }

    /// Wasm export step 2: non-blocking check on the in-flight readback.
    /// Returns `Ok(Some(pixels))` once the GPU is done, `Ok(None)` if still
    /// pending, `Err(...)` if the readback failed.
    pub fn try_finish_export(
        &self,
        job: &PendingExport,
    ) -> anyhow::Result<Option<Vec<u8>>> {
        match job.rx.try_recv() {
            Err(std::sync::mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("readback channel: {e}")),
            Ok(Err(e)) => Err(anyhow::anyhow!("buffer map: {e:?}")),
            Ok(Ok(())) => {
                let slice = job.readback.slice(..);
                let mapped = slice.get_mapped_range();
                let mut out = Vec::with_capacity(
                    (job.unpadded_row_bytes as usize) * (job.height as usize),
                );
                for row in 0..job.height {
                    let row_start = (row as usize) * (job.padded_row_bytes as usize);
                    let row_end = row_start + (job.unpadded_row_bytes as usize);
                    out.extend_from_slice(&mapped[row_start..row_end]);
                }
                drop(mapped);
                job.readback.unmap();
                Ok(Some(out))
            }
        }
    }
}

/// IEEE 754 f16 bit pattern → f32. Subnormals flush to zero (acceptable for
/// our pixel data — bloom + nebula values are well above subnormal range).
fn f16_bits_to_f32(half: u16) -> f32 {
    let sign = ((half >> 15) & 1) as u32;
    let exp = ((half >> 10) & 0x1f) as u32;
    let mant = (half & 0x3ff) as u32;
    let bits = if exp == 0 {
        sign << 31
    } else if exp == 0x1f {
        (sign << 31) | 0x7f80_0000 | (mant << 13)
    } else {
        let e = exp + (127 - 15);
        (sign << 31) | (e << 23) | (mant << 13)
    };
    f32::from_bits(bits)
}

/// Encode the shared nebula → bloom-down → bloom-up → tonemap pass sequence
/// against a caller-supplied set of resources. Used by all three render
/// methods (live preview, equirect PNG, cubemap PNG, equirect EXR).
#[allow(clippy::too_many_arguments)]
fn encode_pipeline_pass(
    encoder: &mut wgpu::CommandEncoder,
    nebula_pipeline: &wgpu::RenderPipeline,
    nebula_bg: &wgpu::BindGroup,
    bloom_down_first_pipeline: &wgpu::RenderPipeline,
    bloom_down_pipeline: &wgpu::RenderPipeline,
    bloom_up_pipeline: &wgpu::RenderPipeline,
    final_pipeline: &wgpu::RenderPipeline,
    final_bg: &wgpu::BindGroup,
    hdr: &HdrTarget,
    bloom: &BloomPyramid,
    final_view: &wgpu::TextureView,
    bloom_downsample_bgs: &[wgpu::BindGroup],
    bloom_upsample_bgs: &[wgpu::BindGroup],
) {
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("erebus.encode.nebula"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &hdr.view,
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
        pass.set_pipeline(nebula_pipeline);
        pass.set_bind_group(0, nebula_bg, &[]);
        pass.draw(0..3, 0..1);
    }

    let mip_count = bloom.mip_count() as usize;
    for i in 0..mip_count {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("erebus.encode.bloom.down"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &bloom.mips[i],
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
        if i == 0 {
            pass.set_pipeline(bloom_down_first_pipeline);
        } else {
            pass.set_pipeline(bloom_down_pipeline);
        }
        pass.set_bind_group(0, &bloom_downsample_bgs[i], &[]);
        pass.draw(0..3, 0..1);
    }
    if mip_count > 1 {
        for i in (0..(mip_count - 1)).rev() {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("erebus.encode.bloom.up"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &bloom.mips[i],
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(bloom_up_pipeline);
            pass.set_bind_group(0, &bloom_upsample_bgs[i], &[]);
            pass.draw(0..3, 0..1);
        }
    }

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("erebus.encode.final"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: final_view,
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
        pass.set_pipeline(final_pipeline);
        pass.set_bind_group(0, final_bg, &[]);
        pass.draw(0..3, 0..1);
    }
}

fn uniform_buf(device: &wgpu::Device, name: &str, size: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(&format!("erebus.{name}_uniforms")),
        size: size as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn create_nebula_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let uniform_entry = |binding: u32| wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    };
    let tex_1d_filterable = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D1,
            multisampled: false,
        },
        count: None,
    };
    let sampler_filtering = wgpu::BindGroupLayoutEntry {
        binding: 0,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.nebula"),
        entries: &[
            uniform_entry(0),
            uniform_entry(1),
            uniform_entry(2),
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                ..tex_1d_filterable
            },
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                ..sampler_filtering
            },
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D3,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 6,
                ..sampler_filtering
            },
            uniform_entry(7),
            wgpu::BindGroupLayoutEntry {
                binding: 8,
                ..tex_1d_filterable
            },
            wgpu::BindGroupLayoutEntry {
                binding: 9,
                ..sampler_filtering
            },
        ],
    })
}

#[allow(clippy::too_many_arguments)]
fn create_nebula_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    frame: &wgpu::Buffer,
    nebula: &wgpu::Buffer,
    lighting: &wgpu::Buffer,
    starfield: &wgpu::Buffer,
    gradient_view: &wgpu::TextureView,
    gradient_sampler: &wgpu::Sampler,
    noise_view: &wgpu::TextureView,
    noise_sampler: &wgpu::Sampler,
    blackbody_view: &wgpu::TextureView,
    blackbody_sampler: &wgpu::Sampler,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("erebus.bg.nebula"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: frame.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: nebula.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 2, resource: lighting.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(gradient_view) },
            wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::Sampler(gradient_sampler) },
            wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(noise_view) },
            wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::Sampler(noise_sampler) },
            wgpu::BindGroupEntry { binding: 7, resource: starfield.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(blackbody_view) },
            wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(blackbody_sampler) },
        ],
    })
}

fn create_bake_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.bake"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: NoiseVolume::FORMAT,
                    view_dimension: wgpu::TextureViewDimension::D3,
                },
                count: None,
            },
        ],
    })
}

fn create_bake_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    bake_buffer: &wgpu::Buffer,
    noise_view: &wgpu::TextureView,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("erebus.bg.bake"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: bake_buffer.as_entire_binding() },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(noise_view) },
        ],
    })
}

// One layout shared by both downsample and upsample passes: source 2-D texture
// + filtering sampler + post uniform.
fn create_bloom_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.bloom"),
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
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn make_bloom_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    src_view: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    post_buffer: &wgpu::Buffer,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(src_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(sampler) },
            wgpu::BindGroupEntry { binding: 2, resource: post_buffer.as_entire_binding() },
        ],
    })
}

fn build_bloom_downsample_bgs(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    hdr_view: &wgpu::TextureView,
    bloom: &BloomPyramid,
    sampler: &wgpu::Sampler,
    post_buffer: &wgpu::Buffer,
) -> Vec<wgpu::BindGroup> {
    let mip_count = bloom.mip_count() as usize;
    let mut out = Vec::with_capacity(mip_count);
    for i in 0..mip_count {
        let src = if i == 0 { hdr_view } else { &bloom.mips[i - 1] };
        out.push(make_bloom_bg(
            device,
            layout,
            src,
            sampler,
            post_buffer,
            "erebus.bg.bloom.down",
        ));
    }
    out
}

fn build_bloom_upsample_bgs(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    bloom: &BloomPyramid,
    sampler: &wgpu::Sampler,
    post_buffer: &wgpu::Buffer,
) -> Vec<wgpu::BindGroup> {
    let mip_count = bloom.mip_count() as usize;
    if mip_count < 2 {
        return Vec::new();
    }
    let mut out = Vec::with_capacity(mip_count - 1);
    // bgs[i] reads bloom.mips[i+1] (smaller) so it can write into mips[i].
    for i in 0..(mip_count - 1) {
        out.push(make_bloom_bg(
            device,
            layout,
            &bloom.mips[i + 1],
            sampler,
            post_buffer,
            "erebus.bg.bloom.up",
        ));
    }
    out
}

fn create_tonemap_bgl(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("erebus.bgl.tonemap"),
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
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

fn create_tonemap_bg(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    hdr_view: &wgpu::TextureView,
    bloom_mip0: &wgpu::TextureView,
    sampler: &wgpu::Sampler,
    post_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("erebus.bg.tonemap"),
        layout,
        entries: &[
            wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(hdr_view) },
            wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(bloom_mip0) },
            wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(sampler) },
            wgpu::BindGroupEntry { binding: 3, resource: post_buffer.as_entire_binding() },
        ],
    })
}

struct PipelineSet {
    nebula: wgpu::RenderPipeline,
    bake: wgpu::ComputePipeline,
    bloom_downsample_first: wgpu::RenderPipeline,
    bloom_downsample: wgpu::RenderPipeline,
    bloom_upsample: wgpu::RenderPipeline,
    tonemap: wgpu::RenderPipeline,
    export_tonemap: wgpu::RenderPipeline,
    export_linear: wgpu::RenderPipeline,
}

pub const EXPORT_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;
pub const EXPORT_LINEAR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

fn build_pipelines(
    device: &wgpu::Device,
    surface_format: wgpu::TextureFormat,
    nebula_bgl: &wgpu::BindGroupLayout,
    bake_bgl: &wgpu::BindGroupLayout,
    bloom_bgl: &wgpu::BindGroupLayout,
    tonemap_bgl: &wgpu::BindGroupLayout,
) -> anyhow::Result<PipelineSet> {
    let fullscreen_src = load_shader("fullscreen.wgsl")?;
    let nebula_src = load_shader("nebula/raymarch.wgsl")?;
    let composite_src = load_shader("composite.wgsl")?;
    let bake_src = load_shader("compute/bake_3d_noise.wgsl")?;
    let bloom_down_src = load_shader("bloom/downsample.wgsl")?;
    let bloom_up_src = load_shader("bloom/upsample.wgsl")?;

    validate(&fullscreen_src, "fullscreen.wgsl")?;
    validate(&nebula_src, "nebula/raymarch.wgsl")?;
    validate(&composite_src, "composite.wgsl")?;
    validate(&bake_src, "compute/bake_3d_noise.wgsl")?;
    validate(&bloom_down_src, "bloom/downsample.wgsl")?;
    validate(&bloom_up_src, "bloom/upsample.wgsl")?;

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
    let bake_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bake.3d_noise"),
        source: wgpu::ShaderSource::Wgsl(bake_src.into()),
    });
    let bloom_down_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bloom.downsample"),
        source: wgpu::ShaderSource::Wgsl(bloom_down_src.into()),
    });
    let bloom_up_mod = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("bloom.upsample"),
        source: wgpu::ShaderSource::Wgsl(bloom_up_src.into()),
    });

    let nebula_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.nebula"),
        bind_group_layouts: &[nebula_bgl],
        push_constant_ranges: &[],
    });
    let tonemap_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.tonemap"),
        bind_group_layouts: &[tonemap_bgl],
        push_constant_ranges: &[],
    });
    let bake_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.bake"),
        bind_group_layouts: &[bake_bgl],
        push_constant_ranges: &[],
    });
    let bloom_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("erebus.pl.bloom"),
        bind_group_layouts: &[bloom_bgl],
        push_constant_ranges: &[],
    });

    let nebula = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.nebula"),
        layout: Some(&nebula_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &nebula_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let bloom_target = wgpu::ColorTargetState {
        format: HDR_FORMAT,
        blend: None,
        write_mask: wgpu::ColorWrites::ALL,
    };
    let bloom_down_first = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.bloom.downsample_first"),
        layout: Some(&bloom_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &bloom_down_mod,
            entry_point: Some("fs_main_first"),
            compilation_options: Default::default(),
            targets: &[Some(bloom_target.clone())],
        }),
        multiview: None,
        cache: None,
    });

    let bloom_down = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.bloom.downsample"),
        layout: Some(&bloom_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &bloom_down_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(bloom_target.clone())],
        }),
        multiview: None,
        cache: None,
    });

    let bloom_up = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.bloom.upsample"),
        layout: Some(&bloom_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &bloom_up_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: HDR_FORMAT,
                // Additive blending so the upsample chain accumulates into
                // the existing mip rather than overwriting it.
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent::REPLACE,
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let tonemap = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.tonemap"),
        layout: Some(&tonemap_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &composite_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let export_tonemap = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.tonemap.export"),
        layout: Some(&tonemap_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &composite_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: EXPORT_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let export_linear = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("erebus.pipeline.tonemap.linear"),
        layout: Some(&tonemap_layout),
        vertex: wgpu::VertexState {
            module: &fullscreen_mod,
            entry_point: Some("vs_main"),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &composite_mod,
            entry_point: Some("fs_main"),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format: EXPORT_LINEAR_FORMAT,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
        cache: None,
    });

    let bake = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("erebus.pipeline.bake_3d_noise"),
        layout: Some(&bake_layout),
        module: &bake_mod,
        entry_point: Some("cs_main"),
        compilation_options: Default::default(),
        cache: None,
    });

    Ok(PipelineSet {
        nebula,
        bake,
        bloom_downsample_first: bloom_down_first,
        bloom_downsample: bloom_down,
        bloom_upsample: bloom_up,
        tonemap,
        export_tonemap,
        export_linear,
    })
}

/// Native: read the requested shader file from disk so hot-reload sees
/// edits. WASM: return the version baked into the binary via `include_str!`,
/// since the browser sandbox has no filesystem.
fn load_shader(rel_path: &str) -> anyhow::Result<String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = shader_root().join(rel_path);
        return std::fs::read_to_string(&path)
            .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()));
    }

    #[cfg(target_arch = "wasm32")]
    {
        Ok(match rel_path {
            "fullscreen.wgsl" => include_str!("../../shaders/fullscreen.wgsl").to_string(),
            "composite.wgsl" => include_str!("../../shaders/composite.wgsl").to_string(),
            "nebula/raymarch.wgsl" => {
                include_str!("../../shaders/nebula/raymarch.wgsl").to_string()
            }
            "compute/bake_3d_noise.wgsl" => {
                include_str!("../../shaders/compute/bake_3d_noise.wgsl").to_string()
            }
            "bloom/downsample.wgsl" => {
                include_str!("../../shaders/bloom/downsample.wgsl").to_string()
            }
            "bloom/upsample.wgsl" => {
                include_str!("../../shaders/bloom/upsample.wgsl").to_string()
            }
            other => anyhow::bail!("unknown shader path: {other}"),
        })
    }
}


// Native: re-export of `naga` lives at `wgpu::naga` so we get
// source-located error messages before the shader hits the device.
// WASM: `wgpu::naga` isn't re-exported on the web target, so we skip the
// pre-validation and let `wgpu::Device::create_shader_module` produce the
// final error if any.
#[cfg(not(target_arch = "wasm32"))]
fn validate(src: &str, name: &str) -> anyhow::Result<()> {
    wgpu::naga::front::wgsl::parse_str(src)
        .map(|_| ())
        .map_err(|e| anyhow::anyhow!("{name}:\n{}", e.emit_to_string(src)))
}

#[cfg(target_arch = "wasm32")]
fn validate(_src: &str, _name: &str) -> anyhow::Result<()> {
    Ok(())
}
