use krajc::{system_fn, system_fn2, Comp};
use legion::{
    component,
    internals::{query::view::IntoView, storage::component, world::Comp},
    query::{Changed, ComponentFilter, EntityFilter, EntityFilterTuple, Passthrough},
    storage::Component,
    Entity, Query, Read,
};

type QueryFilter<A, B = Passthrough> = EntityFilterTuple<A, B>;

use pollster::FutureExt;
use typed_addr::TypedAddr;

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Deref, DerefMut},
    rc::Rc,
    time::Instant,
};

use cgmath::Vector3;

use engine_runtime::{
    schedule_manager::{
        runtime_schedule::{
            RuntimeEngineLoadScheduleData, RuntimeUpdateSchedule, RuntimeUpdateScheduleData,
        },
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
    buffer_manager::managed_buffer::ManagedBufferGeneric,
    managers::RenderManagerResource,
    mesh::mesh::Mesh,
    render_entity::{instancing::TestInstanceSchemes, render_entity::TextureMaterialInstance},
};

use wgpu::{Buffer, BufferUsages, SurfaceError};
use winit::{dpi::PhysicalSize, event::*, event_loop::EventLoop, window::WindowBuilder};

use crate::{
    engine_runtime::schedule_manager::runtime_schedule::RuntimeEngineLoadSchedule,
    rendering::material::MaterialGeneric,
};

pub static mut ENGINE_RUNTIME: TypedAddr<EngineRuntime> = TypedAddr::<EngineRuntime>::default();

pub mod ecs;
pub mod engine_runtime;
///#[forbid(clippy::unwrap_used)]
#[allow(invalid_reference_casting)]
pub mod rendering;
pub mod typed_addr;

#[derive(Default)]
pub struct UniformBufferType {
    instance_handles: HashMap<String, (&'static [u8], Buffer)>,
}
impl ManagedBufferGeneric for UniformBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::UNIFORM | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("uniform buffer")
    }
    fn instance_handles(
        &mut self,
    ) -> &mut std::collections::HashMap<String, (&'static [u8], Buffer)> {
        &mut self.instance_handles
    }
}
#[derive(Default)]
pub struct InstanceBufferType {
    instance_handles: HashMap<String, (&'static [u8], Buffer)>,
}
impl ManagedBufferGeneric for InstanceBufferType {
    fn buffer_usages() -> wgpu::BufferUsages {
        BufferUsages::VERTEX | BufferUsages::COPY_DST
    }
    fn label() -> String {
        String::from("instance_buffer")
    }
    fn instance_handles(
        &mut self,
    ) -> &mut std::collections::HashMap<String, (&'static [u8], Buffer)> {
        &mut self.instance_handles
    }
}

fn main() {
    run().block_on();
}

struct Vel(Vec3);
struct Position(Vec3);
#[derive(Default, Comp)]
struct Health(u32);

#[system_fn(RuntimeEngineLoadSchedule)]
fn startup(startup: SchedData<RuntimeEngineLoadScheduleData>, mut world: EcsWorld) {
    dbg!("ran startup");
    let mut entities_1 = vec![];
    let mut entities_2 = vec![];

    for i in 0..9999 {
        entities_1.push((
            Vel(Vec3 {
                x: i as f32,
                y: -i as f32,
                z: 0.,
            }),
            Position(Vec3 {
                x: -i as f32,
                y: i as f32,
                z: 2. * i as f32,
            }),
        ))
    }
    for i in 0..9999 {
        entities_2.push((
            Vel(Vec3 {
                x: i as f32,
                y: -(i as f32),
                z: 0.,
            }),
            Health(i),
        ))
    }
    world.extend(entities_1);
    world.extend(entities_2);
}

#[system_fn(RuntimeUpdateSchedule)]
fn fps_logger(
    update: SchedData<RuntimeUpdateScheduleData>,
    mut prev_full_sec: Local<u64>,
    mut render_state: Res<RenderManagerResource>,

    mut world: EcsWorld,

    mut counter: Local<u32>,
) {
    *counter += 1;

    let render_state = render_state.get_static_mut();

    world.push((
        Position(Vec3::new(0., 0., *prev_full_sec as f32 + 1.)),
        TextureMaterialInstance::from_pos(Vec3::new(0., 0., *counter.deref() as f32)),
    ));
    dbg!(*counter);

    if *prev_full_sec != update.since_start.as_secs_f64() as u64 {
        dbg!(1. / update.dt.as_secs_f64());
        *prev_full_sec = update.since_start.as_secs_f64() as u64;

        /*let instance_scheme = TestInstanceSchemes::row(*prev_full_sec as i32 + 1);
        *render_state.instance_scheme = instance_scheme.clone();
        let instance_data = instance_scheme
            .iter()
            .map(TextureMaterialInstance::to_raw)
            .collect::<Vec<_>>();
        render_state.instance_buffer.set_data_vec(instance_data);*/

        dbg!(render_state.camera_uniform.view_pos);
    }
}

#[system_fn(RuntimeUpdateSchedule)]
fn update_rendering(
    mut render_state: Res<RenderManagerResource>,
    update: SchedData<RuntimeUpdateScheduleData>,
) {
    let render_state = render_state.get_static_mut();

    render_state
        .camera_controller
        .update_camera(&mut render_state.camera, update.dt.as_secs_f64());
    render_state
        .camera_uniform
        .update_view_proj(&render_state.camera, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);

    render_state.queue.write_buffer(
        &render_state.camera_buffer_actual,
        0,
        bytemuck::cast_slice(&[*render_state.camera_uniform]),
    );

    let new_aspect_uniform = AspectUniform::from_size(*render_state.size);
    render_state.queue.write_buffer(
        &render_state.aspect_buffer,
        0,
        bytemuck::cast_slice(&[new_aspect_uniform]),
    );

    let mesh = Mesh::build_cube(&render_state.device, 1., 1., 1.);

    render_state.material.set_mesh(mesh);
}

pub async fn run() {
    let mut runtime = EngineRuntime::init();

    runtime.buffer_manager.engine = unsafe { ENGINE_RUNTIME.get() };
    runtime
        .buffer_manager
        .register_new_buffer::<UniformBufferType>();
    runtime
        .buffer_manager
        .register_new_buffer::<InstanceBufferType>();

    let render_states = runtime.get_resource::<RenderManagerResource>();
    fps_logger!(runtime);
    update_rendering!(runtime);
    startup!(runtime);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(700, 700))
        .build(&event_loop)
        .unwrap();

    window.set_cursor_visible(false);

    EngineRuntime::init_rendering(window).await;

    env_logger::init();

    let mut last_render_time = Instant::now();

    let start = Instant::now();
    let mut prev_full_sec = 0_u64;
    let window_ref = render_states.window.get_ref();

    let load = runtime.get_resource::<RuntimeEngineLoadSchedule>();
    load.execute();

    render_states.material.register_systems(&mut runtime);
    event_loop.run(move |event, _window_target, control_flow| match event {
        Event::DeviceEvent {
             event: DeviceEvent::MouseMotion{ delta, },
            .. // We're not using device_id currently
        } => {
            render_states.camera_controller.get_ref_mut().process_mouse(delta.0, delta.1);
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
                    runtime.resize(*render_states.size.get_ref())
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
#[derive(PartialEq, Debug, Copy, Clone)]
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
        pub struct $struct_name {
            $(pub $field: GenericStateRefTemplate<$type>),*
        }
        $crate::init_resource!($struct_name);

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
