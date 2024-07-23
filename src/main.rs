#![allow(invalid_reference_casting)]
#![allow(macro_expanded_macro_exports_accessed_by_absolute_paths)]
#![feature(negative_impls)]
#![feature(downcast_unchecked)]

use crate::rendering::systems::general::update_rendering;
use bevy_ecs::{change_detection::DetectChanges, component::Component, query::Changed, world::Ref};
use krajc::{system_fn, Comp};

//pub type QueryFilter<A, B = Passthrough> = EntityFilterTuple<A, B>;

use physics::systems::{
    handle_rigidbody_insert, physics_systems, step_physics, sync_fixed_bodies_to_rapier,
    sync_physics_transform,
};
use pollster::FutureExt;
use rapier3d::{
    math::Translation,
    na::{Isometry3, Transform3, Translation2, UnitQuaternion, Vector3 as Vector},
};
use typed_addr::TypedAddr;

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    time::Instant,
};

use cgmath::{Deg, Point3, Rad, Vector3};

use engine_runtime::{
    schedule_manager::{
        runtime_schedule::{
            self, DepGraph, RuntimeEndFrameSchedule, RuntimeEngineLoadScheduleData,
            RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
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
    buffer_manager::{
        dupe, managed_buffer::ManagedBufferGeneric, InstanceBufferType, UniformBufferType,
    },
    camera::camera::Camera,
    managers::RenderManagerResource,
    material::update_texture_material,
    mesh::mesh::Mesh,
    render_entity::{instancing::TestInstanceSchemes, render_entity::TextureMaterialInstance},
    systems::general::{move_stuff_up, Transform},
};

use wgpu::{Buffer, BufferUsages, RenderBundle, SurfaceError};
use winit::{dpi::PhysicalSize, event::*, event_loop::EventLoop, window::WindowBuilder};

use crate::{
    engine_runtime::schedule_manager::runtime_schedule::RuntimeEngineLoadSchedule,
    rendering::material::MaterialGeneric,
};

pub static mut ENGINE_RUNTIME: TypedAddr<EngineRuntime> = TypedAddr::<EngineRuntime>::default();

pub mod ecs;
pub mod physics;

pub mod engine_runtime;
pub mod rendering;
pub mod typed_addr;

fn main() {
    run().block_on();
}

#[derive(Component)]
pub struct Marker;

#[system_fn(RuntimeEngineLoadSchedule)]
fn startup(mut world: EcsWorld, mut render: Res<RenderManagerResource>) {
    dbg!("ran startup");
    let mut entities: Vec<(Transform, Marker)> = vec![];

    let width = 99;
    let height = 99;

    for y in 0..height {
        for x in 0..width {
            entities.push((
                Transform::new_vec(Vector::new(x as f32, 0., y as f32)),
                Marker,
            ));
        }
    }

    dbg!(entities.len());
    world.spawn_batch(entities);

    let trans = Translation::new(0., 5., 10.);
    let quat = UnitQuaternion::from_euler_angles(
        0.,
        std::convert::Into::<Rad<f32>>::into(Deg(-90.)).0,
        std::convert::Into::<Rad<f32>>::into(Deg(-20.)).0,
    );

    world.spawn((Transform::new(Isometry3::from_parts(trans, quat)), Camera));

    let render = render.get_static_mut();

    let mesh = Mesh::build_cube(&render.device, 1., 1., 1.);

    render.material.set_mesh(mesh);
}

#[system_fn(RuntimeUpdateSchedule)]
fn fps_logger(
    //query: SystemQuery<Read<TextureMaterialInstance>>,
    update: SchedData<RuntimeUpdateScheduleData>,
    mut prev_full_sec: Local<u64>,
    mut render_state: Res<RenderManagerResource>,
) {
    let render_state = render_state.get_static_mut();

    let a = update.since_start;

    if *prev_full_sec != update.deref().since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;
        dbg!(*prev_full_sec);

        dbg!(render_state.camera_uniform.view_pos);
    }
}

pub async fn run() {
    let runtime = EngineRuntime::new();

    runtime.buffer_manager.engine.set(&mut runtime);
    runtime
        .buffer_manager
        .register_new_buffer::<UniformBufferType>();
    runtime
        .buffer_manager
        .register_new_buffer::<InstanceBufferType>();

    let render_states = dupe(&runtime).get_resource_mut::<RenderManagerResource>();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(700, 700))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    //EngineRuntime::init_rendering(window, &mut runtime).await;
    runtime.init_rendering(window).await;

    /*startup!(&runtime);
    fps_logger!(&runtime);
    update_rendering!(&runtime);

    update_texture_material!(&runtime);*/

    physics_systems(&mut runtime);
    //update_texture_material!(runtime);
    /*step_physics!(runtime);
    sync_physics_transform!(runtime);
    sync_fixed_bodies_to_rapier!(runtime);
    handle_rigidbody_insert!(runtime);*/

    //move_stuff_up!(&runtime);

    startup!(&mut runtime);
    fps_logger!(&mut runtime);
    update_rendering!(&mut runtime);

    update_rendering!(&mut runtime);

    move_stuff_up!(&mut runtime);

    dupe(&runtime)
        .get_resource_mut::<RuntimeUpdateSchedule>()
        .calc_dep_graph(&mut runtime);

    dupe(&runtime)
        .get_resource_mut::<RuntimeEngineLoadSchedule>()
        .calc_dep_graph(&mut runtime);

    dupe(&runtime)
        .get_resource_mut::<RuntimeEndFrameSchedule>()
        .calc_dep_graph(&mut runtime);

    env_logger::init();

    let mut last_render_time = Instant::now();

    let start = Instant::now();
    let mut prev_full_sec = 0_u64;
    let window_ref = render_states.window;

    let load = dupe(&runtime).get_resource_mut::<RuntimeEngineLoadSchedule>();
    load.execute(dupe(&runtime));

    //render_states.material.register_systems(runtime);
    event_loop.run(move |event, _window_target, control_flow| match event {
        Event::DeviceEvent {
             event: DeviceEvent::MouseMotion{ delta, },
            .. // We're not using device_id currently
        } => {
            render_states.camera_controller.process_mouse(delta.0, delta.1);
        }

        Event::MainEventsCleared => {
            (*window_ref).request_redraw();
        }
        Event::RedrawRequested(id) if id == (*window_ref).id() => {
            let now = Instant::now();
            let dt = now - last_render_time;
            runtime.update(dt, start);
            last_render_time = now;

            let since_start = now - start;
            let since_start = since_start.as_secs_f32();
            if prev_full_sec != since_start as u64 {
                //println!("fps: {}", 1. / dt.as_secs_f32());
                prev_full_sec = since_start as u64;
            }


            match runtime.render() {
                Ok(_) => {}
                Err(SurfaceError::Lost) => {
                    runtime.resize(*render_states.size)
                },
                Err(SurfaceError::Outdated) => control_flow.set_exit(),
                Err(e) => eprintln!("{:?}", e),
            }
        }
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == window_ref.deref().id() => {
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
        pub struct $struct_name<'a> {
            $(pub $field: GenericStateRefTemplate<$type>),*
        }

        impl<'a> $struct_name {
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
            $(pub $field: $type),*
        }

        impl $struct_name {
            pub fn new() -> Self {
                Self {
                    $($field: $value),*
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
    fn get(&self) -> &T {
        match &self.value {
            LateinitEnum::Some(x) => x,
            LateinitEnum::Uninited => panic!(""),
        }
    }
    fn get_mut(&mut self) -> &mut T {
        match &mut self.value {
            LateinitEnum::Some(x) => x,
            LateinitEnum::Uninited => panic!(""),
        }
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

/*#[system_fn(RuntimeEngineLoadSchedule)]
fn test_a(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_b(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_c(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_d(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_e(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_f(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_g(_q: SystemQuery<(Read<A>, Write<A>)>) {}
fn test_h(_q: SystemQuery<(Read<A>, Write<A>)>) {}

#[derive(Comp, Default)]
struct A {}
#[derive(Comp, Default)]
struct B {}
#[derive(Comp, Default)]
struct C {}
#[derive(Comp, Default)]
struct D {}
#[derive(Comp, Default)]
struct E {}
#[derive(Comp, Default)]
struct F {}
#[derive(Comp, Default)]
struct G {}
#[derive(Comp, Default)]
struct H {}*/

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
