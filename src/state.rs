use wgpu::{RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline};
// lib.rs
use winit::window::Window;

use crate::{
    buffers::Buffers,
    vertex::{Vertex, INDICES},
};

pub struct State<'window> {
    // Surface は Window よりも長い LifeTime を持たなければならない
    surface: wgpu::Surface<'window>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: RenderPipeline,
    window: &'window Window,

    buffers: Buffers,
}

impl<'window> State<'window> {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: &'window Window) -> State<'window> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let surface = instance.create_surface(window).unwrap();

        // GPU Handler
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        // GPUのInterface と CommandBufferを実行する
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(), // adapter.features()で機能一覧を見られる
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None,
            )
            .await
            .unwrap();

        // 画面サイズなどの設定
        let config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();
        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let buffers = Buffers::new(&device);
        let bind_group_and_layout = buffers
            .bind_groups
            .iter()
            .map(|bind_group| bind_group.group_and_layout(&device))
            .collect::<Vec<_>>();
        let (bind_group_layouts, bind_group): (Vec<_>, Vec<_>) = bind_group_and_layout
            .iter()
            .map(|(group, layout)| (group, layout))
            .unzip();
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[], // push定数 変換行列などを渡す
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 正面判定の方法
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default() // 以降は拡張機能に関する設定
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // マルチサンプリングの設定
                mask: !0, // すべての機能をオン
                alpha_to_coverage_enabled: false,
            },
            multiview: None, // 複数視点でのレンダリング（VRなら2）
        });
        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            window,
            buffers,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    pub fn input(&mut self, _: &winit::event::WindowEvent) -> bool {
        todo!()
    }

    pub fn update(&mut self) {
        todo!()
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        // レンダリング先
        let output = self.surface.get_current_texture()?;

        // Textureに関するメタデータ
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        // コマンドをGPUに送るためのオブジェクト
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            // encoderを借用しているため、ライフタイムを制限する
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
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

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.buffers.vertex.buffer.slice(..));
            render_pass.set_index_buffer(
                self.buffers.index.buffer.slice(..),
                self.buffers.index.format,
            );
            render_pass.draw_indexed(0..(INDICES.len() as u32), 0, 0..1); // 2.
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
