use crate::app::{AppContext, AppState, DrawItem};
use crate::view::{View, ViewType};
use backer::{Area, Layout};
use bytemuck::{NoUninit, Pod, Zeroable};
use std::collections::HashMap;
use vello_svg::vello::kurbo::{Affine, Point, RoundedRect, Size, Vec2};
use vello_svg::vello::peniko;
use vello_svg::vello::peniko::{Fill, Mix};
use vello_svg::vello::util::RenderContext;

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
struct ShaderPrefix {
    size: [f32; 2],
    cursor: [f32; 2],
    time: f32,
    _pad: f32,
}

pub(crate) struct ShaderEntry {
    pipeline: wgpu::RenderPipeline,
    framework_bgl: wgpu::BindGroupLayout,
    user_bgl: Option<wgpu::BindGroupLayout>,
    texture: wgpu::Texture,
    pub(crate) image_data: peniko::ImageData,
    framework_buffer: wgpu::Buffer,
    user_buffer: Option<wgpu::Buffer>,
    pub(crate) width: u32,
    pub(crate) height: u32,
    user_buffer_size: usize,
}

pub(crate) struct ShaderRequest {
    pub(crate) id: u64,
    pub(crate) wgsl: &'static str,
    pub(crate) input_bytes: Vec<u8>,
    pub(crate) area: backer::Area,
    pub(crate) scale_factor: f64,
    pub(crate) cursor_local: [f32; 2],
    pub(crate) time: f32,
}

pub(crate) type ShaderCache = HashMap<u64, ShaderEntry>;

fn create_pipeline(
    device: &wgpu::Device,
    wgsl: &str,
    framework_bgl: &wgpu::BindGroupLayout,
    user_bgl: Option<&wgpu::BindGroupLayout>,
) -> wgpu::RenderPipeline {
    let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("custom_shader"),
        source: wgpu::ShaderSource::Wgsl(wgsl.into()),
    });
    let layouts: Vec<&wgpu::BindGroupLayout> = if let Some(ubl) = user_bgl {
        vec![framework_bgl, ubl]
    } else {
        vec![framework_bgl]
    };
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: None,
        bind_group_layouts: &layouts,
        push_constant_ranges: &[],
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("custom_shader_pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &module,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &module,
            entry_point: Some("main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    })
}

pub(crate) fn process_shader(
    entry: &ShaderEntry,
    request: &ShaderRequest,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
) -> wgpu::CommandBuffer {
    let prefix = ShaderPrefix {
        size: [entry.width as f32, entry.height as f32],
        cursor: request.cursor_local,
        time: request.time,
        _pad: 0.0,
    };
    queue.write_buffer(&entry.framework_buffer, 0, bytemuck::bytes_of(&prefix));

    if let Some(ref user_buffer) = entry.user_buffer
        && !request.input_bytes.is_empty()
    {
        queue.write_buffer(user_buffer, 0, &request.input_bytes);
    }

    let framework_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &entry.framework_bgl,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: entry.framework_buffer.as_entire_binding(),
        }],
    });

    let user_bg = entry.user_bgl.as_ref().map(|layout| {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: entry.user_buffer.as_ref().unwrap().as_entire_binding(),
            }],
        })
    });

    let view = entry
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("shader_render"),
    });
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("custom_shader_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        pass.set_pipeline(&entry.pipeline);
        pass.set_bind_group(0, &framework_bg, &[]);
        if let Some(ref ubg) = user_bg {
            pass.set_bind_group(1, ubg, &[]);
        }
        pass.draw(0..3, 0..1);
    }
    encoder.finish()
}

pub(crate) fn ensure_shader_entry(
    cache: &mut ShaderCache,
    renderer: &mut vello_svg::vello::Renderer,
    device: &wgpu::Device,
    request: &ShaderRequest,
) -> bool {
    fn uniform_bgl_entry() -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }
    }

    let pw = (request.area.width as f64 * request.scale_factor) as u32;
    let ph = (request.area.height as f64 * request.scale_factor) as u32;
    let w = pw.max(1);
    let h = ph.max(1);

    if let Some(entry) = cache.get(&request.id) {
        if entry.width == w
            && entry.height == h
            && entry.user_buffer_size == request.input_bytes.len()
        {
            return true;
        }
        renderer.unregister_texture(entry.image_data.clone());
    }

    let framework_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: None,
        entries: &[uniform_bgl_entry()],
    });

    let has_user_inputs = !request.input_bytes.is_empty();

    let user_bgl = if has_user_inputs {
        Some(
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: None,
                entries: &[uniform_bgl_entry()],
            }),
        )
    } else {
        None
    };

    let pipeline = create_pipeline(device, request.wgsl, &framework_bgl, user_bgl.as_ref());
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("shader_output"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });

    let framework_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("shader_framework_uniforms"),
        size: size_of::<ShaderPrefix>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let user_buffer = if has_user_inputs {
        Some(device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("shader_user_uniforms"),
            size: request.input_bytes.len() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        }))
    } else {
        None
    };

    let image_data = renderer.register_texture(texture.clone());

    cache.insert(
        request.id,
        ShaderEntry {
            pipeline,
            framework_bgl,
            user_bgl,
            texture,
            image_data,
            framework_buffer,
            user_buffer,
            width: w,
            height: h,
            user_buffer_size: request.input_bytes.len(),
        },
    );
    true
}

pub struct Shader {
    pub(crate) id: u64,
    pub(crate) wgsl: &'static str,
    pub(crate) input_bytes: Vec<u8>,
    pub(crate) corner_rounding: f32,
}

impl Clone for Shader {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            wgsl: self.wgsl,
            input_bytes: self.input_bytes.clone(),
            corner_rounding: self.corner_rounding,
        }
    }
}

pub fn shader(id: u64, wgsl: &'static str) -> Shader {
    Shader {
        id,
        wgsl,
        input_bytes: Vec::new(),
        corner_rounding: 0.,
    }
}

impl Shader {
    pub fn inputs<Inputs: NoUninit>(mut self, inputs: Inputs) -> Self {
        self.input_bytes = bytemuck::bytes_of(&inputs).to_vec();
        self
    }

    pub fn corner_rounding(mut self, radius: f32) -> Self {
        self.corner_rounding = radius;
        self
    }

    pub fn view<State>(self) -> View<State> {
        View {
            view_type: ViewType::Shader(self),
            z_index: 0,
            gesture_handlers: Vec::new(),
        }
    }

    pub fn finish<State: 'static>(
        self,
        app: &mut AppState<State>,
    ) -> Layout<DrawItem<State>, AppContext> {
        self.view().finish(app)
    }

    pub(crate) fn draw<State>(
        &self,
        area: Area,
        app_state: &mut AppState<State>,
        context: &RenderContext,
        renderers: &mut [Option<vello_svg::vello::Renderer>],
        shader_cache: &mut ShaderCache,
        dev_id: usize,
        start_instant: std::time::Instant,
        command_buffers: &mut Vec<wgpu::CommandBuffer>,
    ) {
        let device = &context.devices[dev_id].device;
        let scale_factor = app_state.scale_factor;

        let cursor_local = app_state
            .cursor_position
            .map(|pos| {
                let lx = (pos.x as f32 - area.x) / area.width;
                let ly = (pos.y as f32 - area.y) / area.height;
                if (0.0..=1.0).contains(&lx) && (0.0..=1.0).contains(&ly) {
                    [lx, ly]
                } else {
                    [-1.0, -1.0]
                }
            })
            .unwrap_or([-1.0, -1.0]);

        let request = ShaderRequest {
            id: self.id,
            wgsl: self.wgsl,
            input_bytes: self.input_bytes.clone(),
            area,
            scale_factor,
            cursor_local,
            time: start_instant.elapsed().as_secs_f32(),
        };

        if let Some(renderer) = renderers[dev_id].as_mut() {
            ensure_shader_entry(shader_cache, renderer, device, &request);
        }

        if let Some(entry) = shader_cache.get(&self.id) {
            let w = entry.width as f64;
            let h = entry.height as f64;
            let area_x = area.x as f64 * scale_factor;
            let area_y = area.y as f64 * scale_factor;
            let area_w = area.width as f64 * scale_factor;
            let area_h = area.height as f64 * scale_factor;

            let scale = (area_w / w).min(area_h / h);
            let dx = area_x + (area_w - w * scale) / 2.0;
            let dy = area_y + (area_h - h * scale) / 2.0;
            let transform = Affine::IDENTITY
                .then_scale(scale)
                .then_translate(Vec2::new(dx, dy));

            if self.corner_rounding > 0.0 {
                app_state.scene.push_layer(
                    Fill::NonZero,
                    Mix::Normal,
                    1.,
                    transform,
                    &RoundedRect::from_origin_size(
                        Point::ZERO,
                        Size::new(w, h),
                        self.corner_rounding as f64 / scale,
                    ),
                );
            }
            app_state.scene.draw_image(&entry.image_data, transform);
            if self.corner_rounding > 0.0 {
                app_state.scene.pop_layer();
            }

            let queue = &context.devices[dev_id].queue;
            command_buffers.push(process_shader(entry, &request, device, queue));
        }
    }
}
