#![allow(invalid_reference_casting)]
#![allow(clippy::module_inception)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
//#![feature(negative_impls)]
//#![feature(stmt_expr_attributes)]
#![allow(clippy::type_complexity)]

use crate::rendering::systems::general::update_rendering;
use bevy_ecs::{component::Component, query::With};

//pub type QueryFilter<A, B = Passthrough> = EntityFilterTuple<A, B>;

use krajc::system_fn;
use physics::{
    components::{
        collider::Collider,
        general::{LinearVelocity, PhysicsSyncDirectBodyModifications, RigidBody, RigidBodyHandle},
    },
    systems::rigid_body::physics_systems,
    Gravity,
};
use pollster::FutureExt;
use rapier3d::{
    dynamics::{RigidBodySet, RigidBodyType},
    geometry::ColliderShape,
    math::Translation,
    na::{Isometry3, UnitQuaternion, Vector3 as Vector},
};

use typed_addr::{dupe, TypedAddr};

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};

use cgmath::{Deg, Point3, Rad, Vector3};

use rapier3d::na::Vector3 as NaVec3;

use engine_runtime::{
    schedule_manager::{
        runtime_schedule::{
            RuntimeEndFrameSchedule, RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
        },
        schedule::ScheduleRunnable,
        system_params::{
            system_local::Local,
            system_query::{EcsWorld, SystemQuery},
            system_resource::Res,
            system_schedule_data::SchedData,
        },
    },
    EngineRuntime,
};

use ordered_float::OrderedFloat;
use rendering::{
    aspect_ratio::AspectUniform,
    buffer_manager::{managed_buffer::ManagedBufferGeneric, InstanceBufferType, UniformBufferType},
    builtin_materials::{
        light_material::material::update_light_material,
        texture_material::material::update_texture_material,
    },
    camera::camera::Camera,
    managers::RenderManagerResource,
    mesh::mesh::Mesh,
    systems::general::{move_light, move_stuff_up, Transform},
};

use wgpu::{BufferUsages, SurfaceError};
use winit::{dpi::PhysicalSize, event::*, event_loop::EventLoop, window::WindowBuilder};

use crate::engine_runtime::schedule_manager::runtime_schedule::RuntimeEngineLoadSchedule;

pub static mut ENGINE_RUNTIME: TypedAddr<EngineRuntime> = TypedAddr::<EngineRuntime>::default();

pub mod ecs;
pub mod physics;

pub mod engine_runtime;
pub mod rendering;
pub mod typed_addr;

/*#[cfg(not(feature="prod"))]
#[global_allocator]
static ALLOC: GlobalAllocatorSampled = GlobalAllocatorSampled::new(100);*/

fn main() {
    run().block_on();
}

#[derive(Component)]
pub struct Marker;

#[derive(Component)]
pub struct LightMaterialMarker;

#[system_fn(RuntimeEngineLoadSchedule)]
fn startup(mut world: EcsWorld, mut render: Res<RenderManagerResource>, gravity: Res<Gravity>) {
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
    for y in 0..16 {
        for x in 0..16 {
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

    let render = render.get_static_mut();

    let mesh = Mesh::cube(&render.device);

    render.light_material.set_mesh(mesh);
}

#[system_fn(RuntimeUpdateSchedule)]
fn fps_logger(
    update: SchedData<RuntimeUpdateScheduleData>,
    mut prev_full_sec: Local<u64>,
    mut render_state: Res<RenderManagerResource>,
) {
    let render_state = render_state.get_static_mut();

    if *prev_full_sec != update.since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;
        dbg!(*prev_full_sec);
    }
}

#[system_fn(RuntimeUpdateSchedule)]
fn move_objects_away(
    mut camera: SystemQuery<&Transform, With<Camera>>,
    mut bodies: SystemQuery<(&Transform, &mut LinearVelocity)>,
) {
    let camera = match camera.get_single() {
        Ok(x) => x,
        Err(_) => return,
    };

    for (trans, mut vel) in bodies.iter_mut() {
        let mut vec = camera.translation.vector - trans.translation.vector;
        vec.normalize_mut();

        vel.0 = vec * 2.;
    }
}

pub async fn run() {
    let runtime = EngineRuntime::init();

    runtime.buffer_manager.engine = unsafe { ENGINE_RUNTIME.get() };
    dupe(runtime)
        .buffer_manager
        .register_new_buffer::<UniformBufferType>();
    dupe(runtime)
        .buffer_manager
        .register_new_buffer::<InstanceBufferType>();

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

    //update_texture_material!(runtime);
    update_light_material!(runtime);
    //move_light!(runtime);

    physics_systems(runtime);

    //move_objects_away!(runtime);

    //move_stuff_up!(runtime);

    runtime
        .get_resource_mut::<RuntimeUpdateSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimeEngineLoadSchedule>()
        .calc_dep_graph(runtime);

    runtime
        .get_resource_mut::<RuntimeEndFrameSchedule>()
        .calc_dep_graph(runtime);

    env_logger::init();

    let mut last_render_time = Instant::now();

    let start = Instant::now();
    let mut prev_full_sec = 0_u64;
    let window_ref = render_states.window.deref();

    let load = runtime.get_resource_mut::<RuntimeEngineLoadSchedule>();
    load.execute(dupe(runtime));

    //render_states.material.register_systems(runtime);

    event_loop.run(move |event, _window_target, control_flow| {
        span!(trace_loop, "event loop");

        match event {
            Event::NewEvents(StartCause::Poll) => {
                let now = Instant::now();
                let dt = now - last_render_time;


                runtime.update(dt, start);
                last_render_time = now;
                //dbg!(1. / dt.as_secs_f32());
                let since_start = now - start;
                let since_start = since_start.as_secs_f32();
                if prev_full_sec != since_start as u64 {
                    //println!("fps: {}", 1. / dt.as_secs_f32());
                    prev_full_sec = since_start as u64;
                }

                match runtime.render() {
                    Ok(_) => {}
                    Err(SurfaceError::Lost) => runtime.resize(*render.get().size),
                    Err(SurfaceError::Outdated) => control_flow.set_exit(),
                    Err(e) => eprintln!("{:?}", e),
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
        }
    });
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
enum LateinitEnum<T> {
    Some(T),
    Uninited,
}
pub struct Lateinit<T> {
    value: LateinitEnum<T>,
}
impl<T> Lateinit<T> {
    fn set(&mut self, value: T) {
        self.value = LateinitEnum::<T>::Some(value);
    }
}
impl<T> Deref for Lateinit<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        match &self.value {
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
                panic!("dereferenced an uninited value")
            }
        }
    }
}
impl<T> DerefMut for Lateinit<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match &mut self.value {
            LateinitEnum::Some(value) => value,
            LateinitEnum::Uninited => {
                panic!("dereferenced an uninited value")
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
