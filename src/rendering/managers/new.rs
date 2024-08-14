use cgmath::Vector3;

use wgpu::*;
use winit::window::Window;

use crate::{
    engine_runtime::EngineRuntime,
    rendering::{
        buffer_manager::managed_buffer::ManagedBufferInstanceHandle,
        builtin_materials::light_material::material::LightUniform,
        camera::camera::{CameraController, CameraUniform, Projection},
        texture::texture::Texture,
    },
    InstanceBufferType, UniformBufferType,
};

use super::RenderManagerResource;

impl EngineRuntime {
    pub async fn init_rendering(&mut self, window: Window) {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::all(),
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
        let first_limits = Limits::default();
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

        let light_uniform =
            LightUniform::new([0., 30., 50.], [1., 1., 1.], Vector3::unit_x().into());

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
            "camera_buffer".to_owned(),
            camera_uniform,
        );

        let light_buffer = ManagedBufferInstanceHandle::<UniformBufferType>::new_with_init(
            "light_buffer".into(),
            light_uniform,
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
            present_mode: PresentMode::Immediate,
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
        let projection =
            Projection::new(config.width, config.height, cgmath::Deg(45.0), 0.1, 100.0);
        let camera_controller = CameraController::new(4.0, 0.4);

        let depth_texture = Texture::create_depth_texture(&device, &config, "Depth Texture");

        render_state.window.set(window);
        render_state.surface.set(surface);
        render_state.queue.set(queue);
        render_state.config.set(config);
        render_state.size.set(size);
        render_state.texture.set(texture);
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

        render_state.light_uniform.set(light_uniform);
        render_state.light_buffer.set(light_buffer);

        //dbg!(render_state.light_instance_buffer.get_buffer().size());
    }
}
