#![allow(invalid_reference_casting)]
#![allow(clippy::module_inception)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![feature(stmt_expr_attributes)]
#![feature(async_closure)]
#![deny(unused_imports)]
#![allow(clippy::type_complexity)]

use crate::rendering::systems::general::update_rendering;

use bevy_ecs::component::Component;
use futures::{stream::FuturesUnordered, FutureExt, StreamExt};
use krajc::system_fn;
use physics::{
    components::{
        collider::Collider,
        general::{PhysicsSyncDirectBodyModifications, RigidBody},
    },
    systems::rigid_body::physics_systems,
    Gravity,
};
use rapier3d::{
    dynamics::RigidBodyType,
    geometry::ColliderShape,
    math::Translation,
    na::{Isometry3, UnitQuaternion, Vector3 as Vector},
};

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

use cgmath::{num_traits::Signed, Deg, Point3, Quaternion, Rad, Vector3, Zero};

use rapier3d::na::Vector3 as NaVec3;

use engine_runtime::schedule_manager::{
    runtime_schedule::RuntimePostUpdateSchedule,
    schedule::{IntoSystem, Schedule as _},
    system_params::system_param::FunctionSystem,
};
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
    asset::{AssetHandleUntype, AssetLoader, SendWrapper},
    asset_loaders::file_resource_loader::{FileResourceLoader, ShaderLoader, TextureLoader},
    buffer_manager::{managed_buffer::ManagedBufferGeneric, InstanceBufferType, UniformBufferType},
    builtin_materials::light_material::{
        instance_data::LightMaterialInstance,
        material::{update_light_material, LightMaterial},
    },
    camera::camera::Camera,
    draw_pass::DrawPass,
    managers::RenderManagerResource,
    mesh::mesh::TextureVertexTemplates,
    systems::general::{make_light_follow_camera, sync_light, Color, Light, Transform},
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

/*#[cfg(not(feature = "prod"))]
#[global_allocator]
static ALLOC: GlobalAllocatorSampled = GlobalAllocatorSampled::new(100);*/

#[tokio::main]
async fn main() {
    run().await
}

#[derive(Component)]
pub struct TextureMaterialMarker;

#[derive(Component)]
pub struct LightMaterialMarker;

#[system_fn(RuntimeEngineLoadSchedule)]
fn startup(
    mut world: EcsWorld,
    mut render: Res<RenderManagerResource>,
    mut gravity: Res<Gravity>,
    mut target_fps: Res<TargetFps>,
) {
    target_fps.0 = 90.;
    gravity.0 = NaVec3::new(0.04, -0.5, 0.);

    let stack = 1;
    let width = 32;
    let height = 32;

    for stack in 0..stack {
        for y in 0..height {
            for x in 0..width {
                world.spawn((
                    Transform::new_vec(Vector::new(x as f32, stack as f32 * 30., y as f32)),
                    LightMaterialMarker,
                    RigidBody::new(RigidBodyType::Dynamic)
                        .linvel(NaVec3::new(0., 2., 0.))
                        .can_sleep(false)
                        .build(),
                    Collider::new(ColliderShape::ball(0.5)).build(),
                ));
            }
        }
    }
    for y in 0..32 {
        for x in 0..32 {
            world.spawn((
                Transform::new_vec(Vector::new(x as f32, -6., y as f32)),
                LightMaterialMarker,
                RigidBody::new(RigidBodyType::Fixed)
                    .can_sleep(false)
                    .build(),
                Collider::new(ColliderShape::cuboid(0.5, 0.5, 0.5)).build(),
            ));
        }
    }

    let trans = Translation::new(0., 5., 10.);
    let quat = UnitQuaternion::from_euler_angles(
        0.,
        std::convert::Into::<Rad<f32>>::into(Deg(-90.)).0,
        std::convert::Into::<Rad<f32>>::into(Deg(-20.)).0,
    );

    world.spawn((
        Transform::new(Isometry3::from_parts(trans, quat)),
        Camera,
        RigidBody::new(RigidBodyType::Fixed)
            .can_sleep(false)
            //.linvel(NaVec3::new(0., -2., 0.))
            .build(),
        Collider::new(ColliderShape::ball(3.)).density(1.).build(),
        PhysicsSyncDirectBodyModifications,
        //PhysicsDontSyncRotation,
    ));

    world.spawn((
        RigidBody::new(RigidBodyType::Dynamic).build(),
        TextureMaterialMarker,
        Light,
        Transform::new_vec(NaVec3::new(0., 10., 0.)),
        Color(wgpu::Color {
            r: 1.,
            g: 1.,
            b: 1.,
            a: 1.,
        }),
    ));

    let render = render.get_static_mut();

    let mesh = TextureVertexTemplates::cube(&render.device);
    let light_mesh = TextureVertexTemplates::build_cube(&render.device, 0.3, 0.3, 0.3);

    //render.light_material.set_mesh(mesh);
}

#[system_fn(RuntimeUpdateSchedule)]
fn fps_logger(update: Res<RuntimeUpdateScheduleData>, mut prev_full_sec: Local<u64>) {
    if *prev_full_sec != update.since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;
        dbg!(*prev_full_sec);
    }
}

//#[system_fn(RuntimeUpdateSchedules)]
fn testing(_a: Res<Gravity>) {
    //
}

pub async fn run() {
    //dbg!("a");
    let runtime = EngineRuntime::init();

    let thread_rx = runtime.render_resource_manager.thread_rx.clone();

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
                                //dbg!("ran callback!!!!!!!!!!!!!!!!!!!!!!4");

                            let asset_mut = asset.get_mut().unwrap();
                            *asset_mut = res;
                            let handle = AssetHandleUntype::new(uuid, &mut dupe(runtime).render_resource_manager);

                            let callbacks = &asset.callbacks;
                            //dbg!(callbacks);
                            for callback in callbacks {
                                //dbg!("ran callback!!!!!!!!!!!!!!!!!!!!!!4!4!44! at thread");
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

    runtime.buffer_manager.engine = unsafe { ENGINE_RUNTIME.get() };
    runtime
        .render_resource_manager
        .engine
        .set(unsafe { ENGINE_RUNTIME.get() });
    runtime
        .render_resource_manager
        .engine_locked
        .set(Arc::new(RwLock::new(SendWrapper::new(unsafe {
            ENGINE_RUNTIME.get()
        }))));
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<UniformBufferType>();
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<InstanceBufferType>();

    let render_states = runtime.get_resource_mut::<RenderManagerResource>();
    let render = TypedAddr::new_with_ref(render_states);

    let mut event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(700, 700))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    runtime.init_rendering(window).await;

    runtime.register_system::<RuntimeEngineLoadSchedule>(startup.into_system());
    runtime.register_system::<RuntimeUpdateSchedule>(fps_logger.into_system());
    runtime.register_system::<RuntimeUpdateSchedule>(update_rendering.into_system());
    runtime.register_system::<RuntimeUpdateSchedule>(sync_light.into_system());

    runtime.register_system::<RuntimePostPhysicsSyncSchedule>(update_light_material.into_system());
    /*runtime
    .register_system::<RuntimePostPhysicsSyncSchedule>(update_texture_material.into_system());*/

    runtime.register_system::<RuntimeUpdateSchedule>(make_light_follow_camera.into_system());

    //collider_systems(runtime);

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
    let shader_res = runtime.render_resource_manager.load_resource(
        FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl",
            ShaderLoader::default(),
        ),
        vec![|x, runtime| {
            let shader = x.get_typed::<ShaderModule>();
            unsafe { dbg!(shader.get_unchecked()) };
            let shader = unsafe { shader.get_unchecked() };
            LightMaterial::set_render_pipeline(runtime, shader.unwrap());
        }],
    );
    let texture = runtime.render_resource_manager.load_resource(
        FileResourceLoader::<TextureLoader>::new(
            "resources/image/dirt/dirt.png",
            TextureLoader::default(),
        ),
        vec![],
    );

    //let mut finished = false;

    //let mut cx = Context::from_waker(futures::task::noop_waker_ref());
    let mut should_run = true;

    loop {
        let mut events = Vec::new();
        let mut scale_factor_changed: Option<PhysicalSize<u32>> = None;
        let frame_start = Instant::now();
        let dt = frame_start - last_render_time;

        while let Ok(req) = runtime.render_resource_manager.main_exec_rx.try_recv() {
            req();
        }
        while let Ok(req) = runtime
            .render_resource_manager
            .loaded_callback_rx
            .try_recv()
        {
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
                    event: DeviceEvent::MouseMotion{ delta, },
                    .. // We're not using device_id currently
                } => {
                    render.get().camera_controller.deref_mut().process_mouse(delta.0, delta.1);
                },
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window_ref.id() => {
                    if !runtime.window_events(event) {
                        match event {
                            WindowEvent::CloseRequested | get_key_pressed!(VirtualKeyCode::Escape) => {
                                should_run = false;
                            }
                            WindowEvent::Resized(size) => runtime.resize(*size),
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }

        let frame_start = Instant::now();

        while let Ok(req) = runtime.render_resource_manager.main_exec_rx.try_recv() {
            req();
        }
        runtime.update(dt, start);
        last_render_time = frame_start;
        let since_start = frame_start - start;
        let since_start = since_start.as_secs_f32();
        if prev_full_sec != since_start as u64 {
            prev_full_sec = since_start as u64;
        }
        let engine = dupe(runtime);
        let light_material_pass = dupe(engine).engine_cache.cache("light_draw_pass", || {
            let render = engine.get_resource_mut::<RenderManagerResource>();
            let mut material = LightMaterial::from_engine(engine);

            let mesh = TextureVertexTemplates::cube(&render.device);
            material.set_mesh(mesh);
            material.set_texture(texture.clone());
            material.set_instance(render.light_instance_buffer.get().clone());

            material.set_instance_value(vec![LightMaterialInstance::new(
                Vec3::new(0., 0., 0.),
                Quaternion::zero(),
            )]);

            DrawPass::new(
                Box::new(material),
                vec![shader_res.as_untype(), texture.as_untype()],
            )
        });
        render.get().draw_passes.push(light_material_pass);

        span!(trace_render, "rendering");
        match runtime.render() {
            Ok(_) => {}
            Err(SurfaceError::Lost) => runtime.resize(*render.get().size),
            Err(SurfaceError::Outdated) => break,
            Err(e) => eprintln!("{:?}", e),
        }
        drop_span!(trace_render);

        let target_fps = runtime.get_resource::<TargetFps>();

        // added 10 to it because it looks like atleaset on my computer that it slows it too much down by around 10 fps
        let target_frame_time = 1. / (target_fps.0 + 10.);

        let current_frame_time = frame_start.elapsed().as_secs_f32();

        let diff = target_frame_time - current_frame_time;

        if diff.is_positive() {
            spin_sleep::sleep(Duration::from_secs_f32(diff));
        }
        //dbg!("ran frame");

        #[cfg(not(feature = "prod"))]
        tracing_tracy::client::frame_mark();

        if !should_run {
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

#[repr(C)]
#[derive(PartialEq, Debug, Copy, Clone, Default)]
pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
    pub fn as_vector3(&self) -> Vector3<f32> {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
impl From<Vec3> for Vector3<f32> {
    fn from(val: Vec3) -> Self {
        val.as_vector3()
    }
}
impl From<Vector3<f32>> for Vec3 {
    fn from(value: Vector3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}
impl From<Point3<f32>> for Vec3 {
    fn from(value: Point3<f32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}
impl From<Transform> for Vec3 {
    fn from(value: Transform) -> Self {
        Self::new(
            value.translation.x,
            value.translation.y,
            value.translation.z,
        )
    }
}

impl Eq for Vec3 {}

impl Hash for Vec3 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        OrderedFloat(self.x).hash(state);
        OrderedFloat(self.y).hash(state);
        OrderedFloat(self.z).hash(state);
    }
}

#[macro_export]
macro_rules! generate_state_struct{
    ($struct_name:ident { $($field:ident: $type:ty = $value:expr),* $(,)? }) => {
        #[derive(krajc::EngineResource)]
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
        #[derive(krajc::EngineResource)]
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
#[derive(Debug)]
pub struct Lateinit<T> {
    value: LateinitEnum<T>,
}
unsafe impl<T> Send for Lateinit<T> {}

impl<T> Lateinit<T> {
    pub fn new(data: T) -> Self {
        Self {
            value: LateinitEnum::Some(data),
        }
    }
    fn set(&mut self, value: T) {
        self.value = LateinitEnum::<T>::Some(value);
    }
    pub fn as_option(&self) -> Option<&T> {
        match &self.value {
            LateinitEnum::Some(val) => Some(val),
            LateinitEnum::Uninited => None,
        }
    }
    pub fn as_option_mut(&mut self) -> Option<&mut T> {
        match &mut self.value {
            LateinitEnum::Some(val) => Some(val),
            LateinitEnum::Uninited => None,
        }
    }
    pub const fn default_const() -> Self {
        Lateinit {
            value: LateinitEnum::Uninited,
        }
    }
    pub fn consume(self) -> T {
        match self.value {
            LateinitEnum::Some(x) => x,
            LateinitEnum::Uninited => panic!("attempted to consume an uninited Lateinit"),
        }
    }
    pub fn get(&self) -> &T {
        match &self.value {
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
                panic!(
                    "dereferenced an uninited value with type {:?}",
                    type_name::<T>()
                );
            }
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match &mut self.value {
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
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
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
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
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
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
            value: LateinitEnum::Uninited,
        }
    }
}
impl<T: Clone> Clone for Lateinit<T> {
    fn clone(&self) -> Self {
        let value = match &self.value {
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => panic!("tried to clone an uninited value"),
        };
        Self {
            value: LateinitEnum::Some(value.clone()),
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
