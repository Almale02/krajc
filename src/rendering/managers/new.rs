use glyphon::{Buffer, FontSystem, Metrics, SwashCache, TextAtlas, TextRenderer};
use wgpu::*;
use winit::window::Window;

use crate::rendering::buffer_manager::StorageBufferType;
use crate::rendering::lights::{IndexUniform, PointLightUniform, SpotLightUniform};
use crate::typed_addr::dupe;
use crate::{
    engine_runtime::EngineRuntime,
    rendering::{
        buffer_manager::managed_buffer::ManagedBufferInstanceHandle,
        camera::camera::{CameraController, CameraUniform, Projection},
        texture::texture::Texture,
    },
    InstanceBufferType, UniformBufferType,
};

use super::RenderManagerResource;

pub struct TextState {
    pub swapchain_format: TextureFormat,
    pub config: SurfaceCapabilities,
    pub font_system: &'static mut FontSystem,
    pub cahce: SwashCache,
    pub atlas: &'static mut TextAtlas,
    pub text_render: TextRenderer,
    pub buffer: Buffer,
}
impl TextState {
    pub fn new(render: &'static mut RenderManagerResource) -> Self {
        let format = TextureFormat::Bgra8UnormSrgb;
        let atlas = Box::leak(Box::new(TextAtlas::new(
            &render.device,
            &render.queue,
            format,
        )));
        let font_system = Box::leak(Box::new(FontSystem::new()));

        Self {
            swapchain_format: format,
            config: wgpu::SurfaceCapabilities {
                formats: vec![format],
                present_modes: vec![PresentMode::AutoNoVsync],
                alpha_modes: vec![CompositeAlphaMode::Opaque],
                usages: TextureUsages::RENDER_ATTACHMENT,
            },
            font_system: dupe(font_system),
            cahce: SwashCache::new(),
            atlas: dupe(atlas),
            text_render: TextRenderer::new(
                atlas,
                &render.device,
                MultisampleState::default(),
                None,
            ),
            buffer: {
                let mut buffer = Buffer::new(font_system, Metrics::new(18., 18.));

                buffer.set_size(
                    font_system,
                    render.size.width as f32,
                    render.size.height as f32,
                );
                //buffer.set_text(font_system, "Hello world! 👋\nThis is rendered with 🦅 glyphon 🦁\nThe text below should be partially clipped.\na b c d e f g h i j k l m n o p q mamr s t u v w x y z", Attrs::new().family(Family::SansSerif), Shaping::Advanced);
                buffer.shape_until_scroll(font_system);
                buffer
            },
        }
    }
}

impl EngineRuntime {
    pub async fn init_rendering(&mut self, window: Window) {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY | Backends::SECONDARY,
            ..Default::default()
        });

        let surface = unsafe { instance.create_surface(&window) }.expect("failed");

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("failed");

        drop(instance);
        let mut first_limits = Limits::default();

        let second_limits = Limits::downlevel_webgl2_defaults();

        let features = Features::POLYGON_MODE_LINE
            | Features::POLYGON_MODE_POINT
            | Features::BUFFER_BINDING_ARRAY
            | Features::STORAGE_RESOURCE_BINDING_ARRAY;
        let (device, queue) = {
            match adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        features,
                        limits: first_limits,
                        label: None,
                    },
                    None,
                )
                .await
            {
                Ok(x) => x,
                Err(_) => adapter
                    .request_device(
                        &wgpu::DeviceDescriptor {
                            features: Features::default(),
                            limits: second_limits,

                            label: None,
                        },
                        None,
                    )
                    .await
                    .unwrap(),
            }
        };

        let camera_uniform = CameraUniform::default();

        let point_light_uniform: Vec<PointLightUniform> = Vec::new();
        let point_light_count_uniform = IndexUniform::new(0);
        let spot_light_uniform: Vec<SpotLightUniform> = Vec::new();
        let spot_light_count_uniform = IndexUniform::new(0);

        let render_state = self.get_resource_mut::<RenderManagerResource>();

        render_state.device.set(device);
        let device = &*render_state.device;

        let texture_instance_buffer =
            ManagedBufferInstanceHandle::<InstanceBufferType>::new_with_size(
                "texture_instance_buffer".to_owned(),
                1024 * 1024,
            );
        let light_instance_buffer =
            ManagedBufferInstanceHandle::<InstanceBufferType>::new_with_size(
                "light_instance_buffer".to_owned(),
                1024 * 1024,
            );

        let camera_buffer = ManagedBufferInstanceHandle::<UniformBufferType>::new_with_init(
            "camera_buffer",
            camera_uniform,
        );

        let point_light_buffer = ManagedBufferInstanceHandle::<StorageBufferType>::new_with_size(
            "point_light_buffer".to_owned(),
            std::mem::size_of::<PointLightUniform>() as u64,
        );
        let point_light_count_buffer =
            ManagedBufferInstanceHandle::<UniformBufferType>::new_with_init(
                "point_light_count_buffer",
                point_light_count_uniform,
            );
        let spot_light_buffer = ManagedBufferInstanceHandle::<StorageBufferType>::new_with_size(
            "spot_light_buffer".to_owned(),
            std::mem::size_of::<SpotLightUniform>() as u64,
        );
        let spot_light_count_buffer =
            ManagedBufferInstanceHandle::<UniformBufferType>::new_with_init(
                "spot_light_count_buffer",
                spot_light_count_uniform,
            );
        let mut texture =
            Texture::from_path("resources/image/dirt/dirt.png", device, &queue).expect("failed");

        texture
            .texture_bind_group
            .set(texture.get_texture_bind_group(device));

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: PresentMode::AutoNoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        /*let camera = RenderCamera::new(
            //(0.0f32, 5.0f32, 10.0f32).into(),
            Point3::<f32>::new(0., 5., 10.),
            cgmath::Deg(-90.0).into(),
            cgmath::Deg(-20.0).into(),
        );*/
        let projection = Projection::new(
            config.width,
            config.height,
            60_f32.to_radians(),
            0.1,
            1000.0,
        );
        let camera_controller = CameraController::new(4.0, 0.4);

        let depth_texture = Texture::create_depth_texture(device, &config, "Depth Texture");

        render_state.window.set(window);
        render_state.surface.set(surface);
        render_state.queue.set(queue);
        render_state.config.set(config);
        render_state.size.set(size);
        render_state.depth_texture.set(depth_texture);
        render_state
            .light_instance_buffer
            .set(light_instance_buffer);
        render_state
            .texture_instance_buffer
            .set(texture_instance_buffer);
        render_state.projection.set(projection);
        render_state.camera_controller.set(camera_controller);
        render_state.camera_uniform.set(camera_uniform);
        render_state.camera_buffer.set(camera_buffer);
        render_state.clear_color.set(Color::BLACK);

        render_state.point_light_buffer.set(point_light_buffer);
        render_state.spot_light_buffer.set(spot_light_buffer);

        render_state.point_light_uniform.set(point_light_uniform);
        render_state.spot_light_uniform.set(spot_light_uniform);

        render_state
            .point_light_count_buffer
            .set(point_light_count_buffer);
        render_state
            .spot_light_count_buffer
            .set(spot_light_count_buffer);

        render_state
            .text_state
            .set(TextState::new(dupe(render_state)));
    }
}
