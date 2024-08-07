#![allow(invalid_reference_casting)]
#![allow(clippy::module_inception)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
//#![feature(negative_impls)]
#![feature(stmt_expr_attributes)]
#![feature(async_closure)]
#![allow(clippy::type_complexity)]

use crate::rendering::systems::general::update_rendering;

//pub type QueryFilter<A, B = Passthrough> = EntityFilterTuple<A, B>;

use bevy_ecs::component::Component;
use futures::{
    future::join_all, stream::FuturesUnordered, FutureExt, StreamExt
};
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
    any::{type_name, Any}, collections::HashMap, hash::Hash, ops::{Deref, DerefMut}, sync::Arc, time::{Duration, Instant}
};

use cgmath::{num_traits::Signed, Deg, Point3, Rad, Vector3};

use rapier3d::na::Vector3 as NaVec3;

use engine_runtime::{
    schedule_manager::{
        runtime_schedule::{
            RuntimePostPhysicsSyncSchedule,
            RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
        },
        system_params::{
            system_local::Local,
            system_query::EcsWorld,
            system_resource::Res,
        },
    }, target_fps::TargetFps, EngineRuntime
};
use engine_runtime::schedule_manager::runtime_schedule::RuntimePostUpdateSchedule;

use ordered_float::OrderedFloat;
use rendering::{
    asset::{AssetEntrie, AssetLoader, SendWrapper}, asset_loaders::file_resource_loader::{FileResourceLoader, RawFileLoader, ShaderLoader}, buffer_manager::{managed_buffer::ManagedBufferGeneric, InstanceBufferType, UniformBufferType}, builtin_materials::{
        light_material::material::update_light_material,
        texture_material::material::update_texture_material,
    }, camera::camera::Camera, managers::RenderManagerResource, mesh::mesh::TextureVertexTemplates, systems::general::{
        make_light_follow_camera, sync_light, Color, Light, Transform,
    }
};

use wgpu::{BufferUsages, SurfaceError};
use winit::{dpi::PhysicalSize, event::*, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder};

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
fn startup(mut world: EcsWorld, mut render: Res<RenderManagerResource>, mut gravity: Res<Gravity>, mut target_fps: Res<TargetFps>) {

    target_fps.0 = 90.;
    gravity.0 = NaVec3::new(0.04, -0.5, 0.);
    dbg!("ran startup");
    //let mut entities = vec![];

    //gravity.0 = NaVec3::new(0., -2., 0.);

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

    render.light_material.set_mesh(mesh);
    render.texture_material.set_mesh(light_mesh);
}

#[system_fn(RuntimeUpdateSchedule)]
fn fps_logger(update: Res<RuntimeUpdateScheduleData>, mut prev_full_sec: Local<u64>) {
    if *prev_full_sec != update.since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;
        dbg!(*prev_full_sec);
    }
}

pub async fn run() {
    let runtime = EngineRuntime::init();

    let thread_rx: flume::Receiver<(&'static std::sync::Arc<tokio::sync::RwLock<AssetEntrie>>, Box<dyn AssetLoader<Output = Box<dyn Any + Send>>>)> =
        runtime.render_resource_manager.thread_rx.clone();
    let handle = std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let rx = thread_rx;

        rt.block_on(async move {
            let mut futures = FuturesUnordered::new();
            loop {
                tokio::select! {
                    Ok((lock, loader)) = rx.recv_async() => {
                        let future = async move {
                            let res = loader.await;
                            let mut unlock = lock.write().await;
                            unlock.asset = Some(res);
                            unlock.loaded = true; // Ensure loaded is set to true
                        }.boxed();
                        futures.push(future);
                    },
                    Some(_) = futures.next() => {},
                };
            }
            
        });
    });

    runtime.buffer_manager.engine = unsafe { ENGINE_RUNTIME.get() };
    runtime.render_resource_manager.engine.set( unsafe { ENGINE_RUNTIME.get() });
    runtime.render_resource_manager.engine_locked.set(Arc::new(RwLock::new(SendWrapper::new(unsafe {ENGINE_RUNTIME.get()}))));
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<UniformBufferType>();
    dupe(runtime)
        .buffer_manager
        .register_new_buffer_type::<InstanceBufferType>();

    let render_states = runtime.get_resource_mut::<RenderManagerResource>();
    let render = TypedAddr::new_with_ref(render_states);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(700, 700))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    runtime.init_rendering(window).await;

    startup!(runtime);
    fps_logger!(runtime);
    update_rendering!(runtime);
    sync_light!(runtime);

    update_light_material!(runtime);
    update_texture_material!(runtime);

    make_light_follow_camera!(runtime);

    physics_systems(runtime);

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
    //load.execute(dupe(runtime));

    //render_states.material.register_systems(runtime);

    
    let shader_res = 
        runtime
        .render_resource_manager
        .load_resource(FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl"
        
        ));
    let shader_res2 = 
        runtime
        .render_resource_manager
        .load_resource(FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl"
        
        ));
    let shader_res3 = 
        runtime
        .render_resource_manager
        .load_resource(FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl"
        
        ));
    let shader_res4 = 
        runtime
        .render_resource_manager
        .load_resource(FileResourceLoader::<ShaderLoader>::new(
            "resources/shaders/shader_light.wgsl"
        
        ));
    dbg!(shader_res.await.is_loaded().await);
    dbg!(shader_res2.await.is_loaded().await);
    dbg!(shader_res3.await.is_loaded().await);
    dbg!(shader_res4.await.is_loaded().await);
    /*let assets = join_all(runtime.render_resource_manager.load_resource_bulk(vec![
        FileResourceLoader::<ShaderLoader>::new(
                "resources/shaders/shader_light.wgsl"),
        FileResourceLoader::<ShaderLoader>::new(
                "resources/shaders/shader_light.wgsl"),
        FileResourceLoader::<ShaderLoader>::new(
                "resources/shaders/shader_light.wgsl"),
        FileResourceLoader::<ShaderLoader>::new(
                "resources/shaders/shader_light.wgsl"),
    ])).await;

    for i in assets {
        dbg!(i.is_loaded().await);
    }*/

    event_loop.run(move |event, _window_target, control_flow: &mut ControlFlow| {
        span!(trace_loop, "event loop");

        match event {
            Event::NewEvents(StartCause::Poll) => {
                let frame_start = Instant::now();
                let dt = frame_start - last_render_time;

                //dbg!(shader_res.try_is_loaded().unwrap());

                //dbg!(shader_res.try_is_loaded().unwrap());

                
                //runtime.update(dt, start);
                last_render_time = frame_start;
                //dbg!(1. / dt.as_secs_f32());
                let since_start = frame_start - start;
                let since_start = since_start.as_secs_f32();
                if prev_full_sec != since_start as u64 {
                    //println!("fps: {}", 1. / dt.as_secs_f32());
                    prev_full_sec = since_start as u64;
                }

                span!(trace_render, "rendering");
                match runtime.render() {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) => runtime.resize(*render.get().size),
                    Err(SurfaceError::Outdated) => control_flow.set_exit(),
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

                #[cfg(not(feature = "prod"))]
                tracing_tracy::client::frame_mark();
            }
            Event::DeviceEvent {
                event: DeviceEvent::MouseMotion{ delta, },
                .. // We're not using device_id currently
            } => {
                render.get().camera_controller.deref_mut().process_mouse(delta.0, delta.1);

                

            }

            /*Event::MainEventsCleared => {
                (*window_ref).request_redraw();
                (*window_ref).request_redraw();
            }
            Event::RedrawRequested(id) if id == (*window_ref).id() => {
            }*/
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window_ref.id() => {
                if !runtime.window_events(event) {
                    match event {
                        WindowEvent::CloseRequested | get_key_pressed!(VirtualKeyCode::Escape) => {
                            control_flow.set_exit()
                        }
                        WindowEvent::Resized(size) => runtime.resize(*size),
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: _,
                            new_inner_size,
                        } => runtime.resize(**new_inner_size),

                        _ => {}
                    }
                }
            }
            _ => {}
        };
    }
)}

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

create_system!(new_system(a: i32) {
    dbg!(a);
});
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
macro_rules! generate_state_struct_non_resource {
    ($struct_name:ident { $($field:ident: $type:ty = $value:expr),* $(,)? }) => {
        pub struct $struct_name {
            $(pub $field: GenericStateRefTemplate<$type>),*
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: GenericStateRefTemplate::<$type>::new($value)),*
                }
            }
            pub fn init() -> &'static mut Self {
                let mgr = Box::new(Self::default());
                let leaked = Box::leak(mgr);
                let raw = leaked as *mut _;

                leaked
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
            pub fn init() -> &'static mut Self {
                let mgr = Box::new(Self::default());
                let leaked = Box::leak(mgr);
                let _raw = leaked as *mut _;

                leaked
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
        Self {value: LateinitEnum::Some(data)}
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
