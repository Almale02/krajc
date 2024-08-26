#![allow(invalid_reference_casting)]
#![allow(clippy::module_inception)]
//#![deny(unused_imports)]
#![allow(clippy::type_complexity)]

// this aint happening now!
//#![deny(warnings)]

pub mod prelude {
    pub use krajc_macros::*;
    pub use crate::span;
    pub use bevy_ecs;
    pub use rapier3d;
    pub use libloading;
    pub use stabby;
    pub use crate::abi::prelude::*;
    pub use crate::run;
    pub use tracing_tracy;
    
}
use crate::rendering::{builtin_materials::texture_material::material::update_texture_material, systems::general::update_rendering};

use bevy_ecs::query::With;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use gilrs::Gilrs;
use krajc_macros::system_fn;
use libloading::Library;
use physics::{
    components::{
        collider::Collider,
        general::{PhysicsDontSyncRotation, PhysicsSyncDirectBodyModifications, RigidBody},
    },
    systems::rigid_body::physics_systems,
    Gravity,
};
use prelude::{SystemPlugin, SystemPluginRegister};
use rapier3d::{
    dynamics::RigidBodyType,
    geometry::ColliderShape,
    math::Translation,
    na::{Isometry3, UnitQuaternion, Vector3 as Vector},
};

use stabby::libloading::StabbyLibrary;
use tokio::sync::RwLock;
use typed_addr::{dupe, TypedAddr};

use std::{
    any::type_name,
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    sync::{atomic::Ordering, Arc},
    time::{Duration, Instant},
};

use rapier3d::na::Vector3 as NaVec3;

use engine_runtime::{input::{KeyboardInput as KeyInput, MouseInput}, schedule_manager::{
    runtime_schedule::RuntimePostUpdateSchedule,
    schedule::{IntoSystem, Schedule as _},
    system_params::{system_param::FunctionSystem, system_query::SystemQuery},
}};
use engine_runtime::{
    schedule_manager::{
        runtime_schedule::{
            RuntimePostPhysicsSyncSchedule, RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
        },
        system_params::{system_local::Local, system_query::EcsWorld, system_resource::Res},
    },
    target_fps::TargetFps,
    EngineRuntime,
};

use ordered_float::OrderedFloat;
use rendering::{
    asset::{AssetHandleUntype, AssetLoader, AssetManager, SendWrapper},
    asset_loaders::{
        file_resource_loader::{FileResourceLoader, MemoryAsset, ShaderLoader, TextureLoader},
        obj_loader::ObjAsset,
    },
    buffer_manager::{
        managed_buffer::ManagedBufferGeneric, InstanceBufferType, StorageBufferType,
        UniformBufferType,
    },
    builtin_materials::{light_material::material::{
        update_light_material, LightMaterial, LightMaterialResource
    }, texture_material::material::{TextureMaterial, TextureMaterialResource}},
    camera::camera::Camera,
    lights::{LightLookDirText, PointLight, SpotLight},
    managers::RenderManagerResource,
    mesh::mesh::TextureVertexTemplates,
    systems::general::{make_light_follow_camera, sync_light, CameraRotText, Color, Transform},
    text::{update_debug_text, DebugText, DebugTextProducer, FpsText, MouseMotionText},
};

use wgpu::{BufferUsages, ShaderModule, SurfaceError};
use winit::{
    dpi::PhysicalSize, event::*, event_loop::EventLoop,
    platform::run_return::EventLoopExtRunReturn, window::WindowBuilder,
};

use crate::engine_runtime::schedule_manager::runtime_schedule::RuntimeEngineLoadSchedule;

pub static mut ENGINE_RUNTIME: TypedAddr<EngineRuntime> = TypedAddr::<EngineRuntime>::default();

pub mod ecs;
pub mod physics;

pub mod engine_runtime;
pub mod rendering;
pub mod typed_addr;
pub mod abi;

/*#[cfg(not(feature = "prod"))]
#[global_allocator]
static ALLOC: GlobalAllocatorSampled = GlobalAllocatorSampled::new(100);*/

pub static mut CAMERA_ROT: (f32, f32) = (0., 0.0);

//#[tokio::main]
marker_comps!(TextureMaterialMarker, LightMaterialMarker, ArrowEntity);

#[system_fn]
fn startup(
    mut world: EcsWorld,
    mut render: Res<RenderManagerResource>,
    mut gravity: Res<Gravity>,
    mut target_fps: Res<TargetFps>,
    mut asset_manager: Res<AssetManager>,
    mut text_producer: Res<DebugTextProducer>,
) {
    let mud = asset_manager.load_resource(
        FileResourceLoader::<TextureLoader>::new(
            "resources/image/mud.png",
            TextureLoader::default(),
        ),
        vec![],
    );
    let dirt = asset_manager.load_resource(
        FileResourceLoader::<TextureLoader>::new(
            "resources/image/dirt/dirt.png",
            TextureLoader::default(),
        ),
        vec![],
    );
    let stone = asset_manager.load_resource(
        FileResourceLoader::<TextureLoader>::new(
            "resources/image/stone/stone.png",
            TextureLoader::default(),
        ),
        vec![],
    );
    let monkey = asset_manager.load_resource(
        FileResourceLoader::<ObjAsset>::new("resources/meshes/monkey.obj", ObjAsset::default()),
        vec![],
    );
    let arrow = asset_manager.load_resource(
        FileResourceLoader::<ObjAsset>::new("resources/meshes/arrow.obj", ObjAsset::default()),
        vec![],
    );
    let mesh = TextureVertexTemplates::cube(&render.device);
    let mesh = asset_manager.load_resource(MemoryAsset::new(mesh), vec![]);

    target_fps.0 = 90.;
    gravity.0 = NaVec3::new(0.04, -0.5, 0.);

    let stack = 1;
    let width = 32;
    let height = 32;

    for stack in 0..stack {
        for y in 0..height {
            for x in 0..width {
                if x % 2 != 1 || y % 2 != 0 {
                    continue;
                }
                world.spawn((
                    Transform::new_vec(Vector::new(x as f32, stack as f32 * 30., y as f32)),
                    //LightMaterialMarker,
                    dirt.clone(),
                    monkey.clone(),
                    /*RigidBody::new(RigidBodyType::Dynamic)
                        .linvel(NaVec3::new(0., 0., 0.))
                        .can_sleep(false)
                        .build(),
                    Collider::new(ColliderShape::ball(0.5)).build()*/
                ));
            }
        }
    }
    world.spawn((
        //ArrowEntity,
        Transform::new_vec(Vector::new(0., 6., 0.)),
        LightMaterialMarker,
        //TextureMaterialMarker,
        monkey.clone(),
        dirt.clone(),
    ));

    world.spawn((text_producer.create_text("fps: -69"), FpsText));
    world.spawn((
        text_producer.create_text("camera_rot_text_not_set"),
        CameraRotText,
    ));
    world.spawn((text_producer.create_text("idk"), LightLookDirText));
    world.spawn((
        text_producer.create_text("you are in the hell! :)"),
        PlayerPositionText,
    ));
    world.spawn((
        text_producer.create_text(""),
        MouseMotionText,
    ));
    for y in 0..128 {
        for x in 0..128 {
            let x = x - 64;
            let y = y - 64;
            world.spawn((
                Transform::new_vec(Vector::new(x as f32, -6., y as f32)),
                LightMaterialMarker,
                //TextureMaterialMarker,
                stone.clone(),
                mesh.clone(),
                RigidBody::new(RigidBodyType::Fixed)
                    .can_sleep(false)
                    .build(),
                Collider::new(ColliderShape::cuboid(0.5, 0.5, 0.5)).build(),
            ));
        }
    }

    let trans = Translation::new(0., 5., 10.);
    let quat = UnitQuaternion::from_euler_angles(0., 90_f32.to_radians(), -20_f32.to_radians());

    world.spawn((
        Transform::new(Isometry3::from_parts(trans, quat)),
        Camera::default(),
        RigidBody::new(RigidBodyType::Fixed)
            .can_sleep(false)
            //.linvel(NaVec3::new(0., -2., 0.))
            .build(),
        Collider::new(ColliderShape::ball(3.)).density(1.).build(),
        PhysicsSyncDirectBodyModifications,
        PhysicsDontSyncRotation,
    ));

    world.spawn((
        PointLight::new(0.4),
        Transform::new_vec(NaVec3::new(0., 2., 0.)),
        Color(wgpu::Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }),
    ));
    world.spawn((
        SpotLight::new_with_ambient(0.2, 0.00, 0.3, 0.5),
        Transform::new_vec(NaVec3::new(0., 3., 0.)),
        Color(wgpu::Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }),
    ));
}
marker_comps!(PlayerPositionText);

fn update_player_pos_text(
    mut camera_pos: SystemQuery<&Transform, With<Camera>>,
    mut text: SystemQuery<&mut DebugText, With<PlayerPositionText>>,
    mut mouse_text: SystemQuery<&mut DebugText, With<MouseMotionText>>,
    mouse_input: Res<MouseInput>,
) {
    let mut text = text.get_single_mut().unwrap();
    let mut mouse_text = mouse_text.get_single_mut().unwrap();
    let vector = camera_pos.get_single_mut().unwrap().translation.vector;

    text.text = format!(
        "player position: (x: {:.2}m, y: {:.2}m, z: {:.2}m)",
        vector.x, vector.y, vector.z
    );
    mouse_text.text = format!("mouse motion x: {:.2}, y: {:.2}", mouse_input.get_mouse_motion().0, mouse_input.get_mouse_motion().1);
}

#[system_fn]
fn fps_logger(
    update: Res<RuntimeUpdateScheduleData>,
    mut prev_full_sec: Local<u64>,
    mut fps_text: SystemQuery<&mut DebugText, With<FpsText>>,
) {
    if *prev_full_sec != update.since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;
        dbg!(*prev_full_sec);
    }
    let mut text = fps_text.get_single_mut().unwrap();
    text.text = format!("fps: {:.1}", 1. / update.dt.as_secs_f32());
}

//#[system_fn(RuntimeUpdateSchedules)]
fn testing(_a: Res<Gravity>) {
    //
}

#[system_fn]
fn sync_arrow(
    mut camera: SystemQuery<&Camera>,
    mut arrow: SystemQuery<&mut Transform, With<ArrowEntity>>,
) {
    if arrow.get_single().is_err() {
        return;
    }

    let arrow = &mut arrow.single_mut().rotation;
    let camera = camera.single();

    *arrow = UnitQuaternion::from_matrix(&camera.rot_matrix);
}

pub async fn run(game: SystemPlugin) {
    let runtime = EngineRuntime::init();

    runtime
        .asset_manager
        .engine
        .set(unsafe { ENGINE_RUNTIME.get() });
    runtime
        .asset_manager
        .engine_locked
        .set(Arc::new(RwLock::new(SendWrapper::new(unsafe {
            ENGINE_RUNTIME.get()
        }))));

    let thread_rx = runtime.asset_manager.thread_rx.clone();

    let thread_runtime = dupe(runtime);
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rx = thread_rx;

        rt.block_on(async move {
            let runtime = thread_runtime;
            let mut futures = FuturesUnordered::new();
            loop {
                let runtime = dupe(runtime);
                tokio::select! {
                    Ok((asset, loader)) = rx.recv_async() => {
                        let future = async move {
                            let res = loader.await;
                            let uuid = asset.uuid;

                            asset.loaded.store(false, Ordering::SeqCst);

                            let asset_mut = asset.get_mut().unwrap();
                            *asset_mut = res;
                            let handle = AssetHandleUntype::new(uuid, &mut dupe(runtime).asset_manager);

                            let callbacks = &asset.callbacks;
                            for callback in callbacks {
                                callback(handle.clone(), dupe(runtime));
                            }
                            asset.loaded.store(true, Ordering::SeqCst);
                            dbg!("ran end");

                        }.boxed();
                        futures.push(future);
                    },
                    Some(_) = futures.next() => {},
                };
            }
        });
    });

    let shader_light = runtime.asset_manager.load_resource(
        FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl",
            ShaderLoader::default(),
        ),
        vec![|x, runtime| {
            let shader = x.get_typed::<ShaderModule>();
            unsafe { dbg!(shader.get_unchecked()) };
            let shader = unsafe { shader.get_unchecked() };
            LightMaterial::set_render_pipeline(runtime, shader.unwrap(), x);
        }],
    );
    let shader_texture = runtime.asset_manager.load_resource(
        FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_texture.wgsl",
            ShaderLoader::default(),
        ),
        vec![|x, runtime| {
            let shader = x.get_typed::<ShaderModule>();
            unsafe { dbg!(shader.get_unchecked()) };
            let shader = unsafe { shader.get_unchecked() };
            TextureMaterial::set_render_pipeline(runtime, shader.unwrap(), x);
        }],
    );
    

    runtime.buffer_manager.engine = unsafe { ENGINE_RUNTIME.get() };
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<UniformBufferType>();
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<InstanceBufferType>();
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<StorageBufferType>();

    let render_states = runtime.get_resource_mut::<RenderManagerResource>();
    let render = TypedAddr::new_with_ref(render_states);

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(700, 700))
        .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None)))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    runtime.init_rendering(window).await;
    
    runtime.register_system::<RuntimeEngineLoadSchedule>(startup.system());
    runtime.register_system::<RuntimeUpdateSchedule>(fps_logger.system());
    runtime.register_system::<RuntimeUpdateSchedule>(update_rendering.system());
    runtime.register_system::<RuntimeUpdateSchedule>(sync_light.system());
    runtime.register_system::<RuntimeUpdateSchedule>(update_debug_text.system());
    runtime.register_system::<RuntimeUpdateSchedule>(update_player_pos_text.system());
    runtime.register_system::<RuntimeUpdateSchedule>(sync_arrow.system());

    runtime.register_system::<RuntimePostPhysicsSyncSchedule>(update_light_material.system());
    runtime.register_system::<RuntimePostPhysicsSyncSchedule>(update_texture_material.system());
    /*runtime
    .register_system::<RuntimePostPhysicsSyncSchedule>(update_texture_material.system());*/

    (game.register_systems)(SystemPluginRegister::new());

    runtime.register_system::<RuntimeUpdateSchedule>(make_light_follow_camera.system());

    physics_systems(runtime);

    /*AppBuilder::register_systems(|a| {
        a.register(RuntimeEngineLoadSchedule, startup, other_things);
    }).build();*/

    dupe(runtime)
        .get_resource_mut::<RuntimeEngineLoadSchedule>()
        .register(FunctionSystem::new(testing));

    runtime
        .get_resource_mut::<RuntimeEngineLoadSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimeUpdateSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimeEngineLoadSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimePostUpdateSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimePostPhysicsSyncSchedule>()
        .calc_dep_graph(runtime);

    env_logger::init();

    let mut last_render_time = Instant::now();

    let start = Instant::now();
    let mut prev_full_sec = 0_u64;
    let window_ref = render_states.window.deref();

    let load = runtime.get_resource_mut::<RuntimeEngineLoadSchedule>();
    load.execute(dupe(runtime));

    runtime
        .get_resource_mut::<LightMaterialResource>()
        .shader_asset_handle = shader_light.as_untype();
    runtime
        .get_resource_mut::<TextureMaterialResource>()
        .shader_asset_handle = shader_texture.as_untype();


    let mut controller = Gilrs::new().unwrap();


    //let mut finished = false;

    //let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    let key_input = runtime.get_resource_mut::<KeyInput>();
    let mouse_input = runtime.get_resource_mut::<MouseInput>();
    let mut should_run = true;

    loop {
        key_input.reset_events();
        mouse_input.reset_events();
        //let a = controller.next_event_blocking(Some(Duration::from_secs(10)));
        //dbg!(a.unwrap());
        while let Some(x) = controller.next_event() {
            match x.event {
                gilrs::EventType::ButtonPressed(button, code) => {
                    println!("controller pressed {:?} with code {:?}", button, code);
                },
                gilrs::EventType::ButtonRepeated(_, _) => (),
                gilrs::EventType::ButtonReleased(_, _) => (),
                gilrs::EventType::ButtonChanged(button, value, code) => {
                    println!("button {:?} with code {:?} was changed to {}", button, code, value);
                },
                gilrs::EventType::AxisChanged(axis, value, code) => {
                    println!("axis {:?} with code {:?} changed to value {}", axis, code, value);
                },
                gilrs::EventType::Connected => (),
                gilrs::EventType::Disconnected => (),
                gilrs::EventType::Dropped => (),
            }
            dbg!(x);
            //println!("controller event: {:?} received from controller: {:?}, at time: {:?}", x.event, controller.gamepad(x.id).name(), x.time);
        }
        let mut events = Vec::new();
        let mut scale_factor_changed: Option<PhysicalSize<u32>> = None;
        let frame_start = Instant::now();
        let dt = frame_start - last_render_time;

        while let Ok(req) = runtime.asset_manager.main_exec_rx.try_recv() {
            req();
        }
        while let Ok(req) = runtime.asset_manager.loaded_callback_rx.try_recv() {
            req.1();
            req.2(req.0, dupe(runtime))
        }
        event_loop.run_return(|event, _, control_flow_event| match event {
            Event::WindowEvent {
                event: ref window_event,
                ..
            } => match window_event {
                WindowEvent::ScaleFactorChanged {
                    scale_factor: _,
                    new_inner_size,
                } => scale_factor_changed = Some(**new_inner_size),
                _ => events.push(event.to_static().unwrap()),
            },
            Event::MainEventsCleared => {
                control_flow_event.set_exit();

                events.push(event.to_static().unwrap().clone())
            }
            _ => events.push(event.to_static().unwrap().clone()),
        });

        for event in events {
            match event {
                Event::DeviceEvent {
                    event: DeviceEvent::MouseMotion { delta, },
                    .. // We're not using device_id currently
                } => {
                    mouse_input.mouse_motion = (delta.0 as f32, delta.1 as f32);
                },
                Event::DeviceEvent { event: DeviceEvent::Key(KeyboardInput { scancode: _, state, virtual_keycode, modifiers }), .. } => {
                    key_input.register_input(virtual_keycode.unwrap(), state, modifiers);
                    
                },
                Event::DeviceEvent { event: DeviceEvent::Button { button, state }, .. } => {
                    dbg!("yayayaaaaaaaaaaaaaaa");
                    mouse_input.register_input(button, state);
                }
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window_ref.id() => {
                    runtime.window_events(event);
                    match event {
                        WindowEvent::CloseRequested | get_key_pressed!(VirtualKeyCode::Escape) => {
                            should_run = false;
                        }
                        WindowEvent::Resized(size) => runtime.resize(*size),
                        WindowEvent::AxisMotion { device_id, axis, value } => {
                            println!("axis motion event, device: {:?}, axis: {}, value: {}", device_id, axis, value);
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let frame_start = Instant::now();

        while let Ok(req) = runtime.asset_manager.main_exec_rx.try_recv() {
            req();
        }
        if mouse_input.is_pressed(1) {
            println!("left clicked");
        }
        if mouse_input.is_released(1) {
            println!("released left click");
        }
        if mouse_input.is_held_down(1) {
            println!("holding down left click");
        }
        runtime.update(dt, start);
        last_render_time = frame_start;
        let since_start = frame_start - start;
        let since_start = since_start.as_secs_f32();
        if prev_full_sec != since_start as u64 {
            prev_full_sec = since_start as u64;
        }
        let engine = dupe(runtime);

        span!(trace_render, "rendering");
        match runtime.render() {
            Ok(_) => {}
            Err(SurfaceError::Lost) => runtime.resize(*render.get().size),
            Err(SurfaceError::Outdated) => break,
            Err(e) => eprintln!("{:?}", e),
        }
        drop_span!(trace_render);

        let target_fps = runtime.get_resource::<TargetFps>();

        let target_frame_time = 1. / (target_fps.0);

        let current_frame_time = frame_start.elapsed().as_secs_f32();


        let diff = target_frame_time - current_frame_time;

        if diff.is_sign_positive() {
            spin_sleep::sleep(Duration::from_secs_f32(diff));
        }

        #[cfg(not(feature = "prod"))]
        tracing_tracy::client::frame_mark();

        if !should_run {
            //game.close().unwrap();
            break;
        }
    }
}

mod key_macros {
    #[macro_export]
    macro_rules! get_key_pressed {
        ( $key:pat) => {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state: ElementState::Pressed,
                    virtual_keycode: Some($key),
                    ..
                },
                ..
            }
        };
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord, Hash)]
pub struct Float(OrderedFloat<f32>);

impl From<f32> for Float {
    fn from(val: f32) -> Self {
        Float(OrderedFloat(val))
    }
}

#[macro_export]
macro_rules! generate_state_struct{
    ($struct_name:ident { $($field:ident: $type:ty = $value:expr),* $(,)? }) => {
        #[derive(krajc_macros::EngineResource)]
        pub struct $struct_name {
            $(pub $field: GenericStateRefTemplate<$type>),*
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: GenericStateRefTemplate::<$type>::new($value)),*
                }
            }

        }
        impl Default for $struct_name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
    ($struct_name:ident { $($field:ident: $type:ty = $name:expr => $init_value: expr),* $(,)? }) => {
        pub struct $struct_name {
            $(pub $field: GenericStateRefTemplate<$type>),*
        }
        //$crate::init_resource!($struct_name);

        $crate::init_resource!($struct_name);
        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: GenericStateRefTemplate::<$type>::new($name)),*
                }
            }
            pub fn create_new() -> Self {
                Self {
                    $($field: GenericStateRefTemplate::<$type>::new_and_init($name, $init_value)),*
                }
            }
        }
        impl Default for $struct_name {
            fn default() -> Self {
                Self::create_new()
            }
        }
    };
}

#[macro_export]
macro_rules! struct_with_default {
    ($struct_name:ident { $($field:ident: $type:ty = $init_value: expr),* $(,)? }) => {
        #[derive(krajc_macros::EngineResource)]
        pub struct $struct_name {
            $(pub $field: $type),*
        }

        impl Default for $struct_name {
            fn default() -> Self {
                Self {
                    $($field:  $init_value),*
                }
            }
        }

    };
}
#[derive(Debug)]
enum LateinitEnum<T> {
    Some(T),
    Uninited,
}

/// Basically rebranded null because unlike an `Option`, you *dont* have to check if it *has value*,
/// the *library* which returns these to the users needs to *ensure* that the value is *present*
#[derive(Debug)]
pub struct Lateinit<T> {
    value: Option<T>,
}
unsafe impl<T> Send for Lateinit<T> {}

impl<T> Lateinit<T> {
    pub fn new(data: T) -> Self {
        Self {
            value: Some(data),
        }
    }
    pub fn set(&mut self, value: T) {
        self.value = Some(value);
    }
    pub fn as_option(&self) -> Option<&T> {
        match &self.value {
            Some(val) => Some(val),
            None => None,
        }
    }
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match &mut self.value {
            Some(val) => Some(val),
            None => None,
        }
    }
    pub const fn default_const() -> Self {
        Lateinit {
            value: None ,
        }
    }
    pub fn consume(&mut self) -> T {
        match self.value.take() {
            Some(x) => {
                x
            },
            None => panic!("attempted to consume an uninited Lateinit"),
        }
    }
    pub fn get(&self) -> &T {
        match &self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match &mut self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> Deref for Lateinit<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> DerefMut for Lateinit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.value {
            Some(value) => value,
            None => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
}
impl<T> Default for Lateinit<T> {
    fn default() -> Self {
        Self {
            value: None ,
        }
    }
}
impl<T: Clone> Clone for Lateinit<T> {
    fn clone(&self) -> Self {
        let value = match &self.value {
            Some(value) => value,
            None => panic!("tried to clone an uninited value"),
        };
        Self {
            value: Some(value.clone()),
        }
    }
}

pub struct ThreadRawPointer<T>(pub *mut T);
impl<T> ThreadRawPointer<T> {
    pub fn new(value: &T) -> ThreadRawPointer<T> {
        ThreadRawPointer((value as *const T) as *mut T)
    }
}

impl<T> Deref for ThreadRawPointer<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let ptr: &mut T = &mut *self.0;
            ptr
        }
    }
}
impl<T> DerefMut for ThreadRawPointer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let ptr: &mut T = &mut *self.0;
            ptr
        }
    }
}

unsafe impl<T> Sync for ThreadRawPointer<T> {}
unsafe impl<T> Send for ThreadRawPointer<T> {}

pub trait FromEngine {
    fn from_engine(engine: &'static mut EngineRuntime) -> Self;
}

impl<T: Default> FromEngine for T {
    fn from_engine(_engine: &'static mut EngineRuntime) -> Self {
        T::default()
    }
}

struct Takeable<T> {
    value: Option<T>,
}

impl<T> Takeable<T> {
    pub fn new(value: T) -> Self {
        Takeable { value: Some(value) }
    }

    pub fn take(&mut self) -> Option<T> {
        self.value.take()
    }
}
