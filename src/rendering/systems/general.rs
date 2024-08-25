use std::ops::Deref;
use std::ops::DerefMut;

use crate::engine_runtime::input::KeyboardInput;
use crate::engine_runtime::input::MouseInput;
use crate::engine_runtime::schedule_manager::system_params::system_local::Local;
use crate::engine_runtime::schedule_manager::system_params::system_query::SystemQuery;
use crate::marker_comps;
use crate::rendering::camera::camera::Camera;
use crate::rendering::lights::PointLight;
use crate::rendering::lights::PointLightType;
use crate::rendering::lights::SpotLight;
use crate::rendering::lights::SpotLightType;
use crate::rendering::text::DebugText;
use crate::RenderManagerResource;
use crate::RuntimeUpdateScheduleData;
use crate::TextureMaterialMarker;

use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::query::With;
//use cgmath::Vector3;
use krajc_macros::system_fn;
use rapier3d::na::Isometry3;
use rapier3d::na::Translation3;
use rapier3d::na::Vector3;
use winit::event::VirtualKeyCode;

use crate::Res;

pub type Translation = Translation3<f32>;
pub type Vector = Vector3<f32>;

#[derive(Component, Clone, Default, PartialEq, Debug)]
pub struct Transform {
    pub iso: Isometry3<f32>,
}

impl Eq for Transform {}

impl Transform {
    pub fn new(iso: Isometry3<f32>) -> Self {
        Self { iso }
    }
    pub fn new_vec(vec: Vector3<f32>) -> Self {
        Self::new(Isometry3::new(vec, Default::default()))
    }
}

impl Deref for Transform {
    type Target = Isometry3<f32>;
    fn deref(&self) -> &Self::Target {
        &self.iso
    }
}
impl DerefMut for Transform {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.iso
    }
}

marker_comps!(CameraRotText);

#[system_fn(RuntimeUpdateSchedule)]
pub fn update_rendering(
    mut camera: SystemQuery<(&mut Transform, &mut Camera)>,
    mut render_state: Res<RenderManagerResource>,
    update: Res<RuntimeUpdateScheduleData>,
    mut camera_rot_text: SystemQuery<&mut DebugText, With<CameraRotText>>,
    mouse: Res<MouseInput>,
) {
    let motion = mouse.get_mouse_motion();
    render_state
        .camera_controller
        .process_mouse(motion.0 as f64, motion.1 as f64);
    let render_state = render_state.get_static_mut();
    let (mut iso, mut camera) = camera.get_single_mut().unwrap();

    let mut camera_rot_text = camera_rot_text.get_single_mut().unwrap();

    render_state.camera_controller.update_camera(
        &mut iso,
        update.dt.as_secs_f64(),
        &mut camera,
        &mut camera_rot_text.text,
    );
    render_state
        .camera_uniform
        .update_view_proj(&mut iso, &render_state.projection);

    render_state
        .camera_buffer
        .set_data(*render_state.camera_uniform);
}

#[system_fn(RuntimeUpdateSchedule)]
pub fn move_stuff_up(
    mut objects: SystemQuery<(Entity, &mut Transform), With<TextureMaterialMarker>>,
) {
    objects.iter_mut().for_each(|(id, mut trans)| {
        trans.translation.y += 0.1 + (id.index() as f32) / 50000.;
    })
}

#[derive(Component)]
pub struct Light;
#[derive(Component, Clone, Default)]
pub struct Color(pub wgpu::Color);

impl Deref for Color {
    type Target = wgpu::Color;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[system_fn]
pub fn sync_light(
    render_state: Res<RenderManagerResource>,
    mut point_light: SystemQuery<(&Transform, &Color, &PointLight)>,
    mut spot_light: SystemQuery<(&Transform, &Color, &SpotLight)>,
) {
    let point_lights = point_light
        .iter()
        .map(|(trans, color, light)| {
            PointLightType::new(
                trans.translation,
                color.0,
                light.light_strenght,
                light.ambient_strenght,
            )
            .to_raw()
        })
        .collect::<Vec<_>>();
    let spot_lights = spot_light
        .iter()
        .map(|(trans, color, light)| {
            SpotLightType::new(
                trans.clone(),
                color.0,
                light.light_strenght,
                light.ambient_strenght,
                light.inner_angle,
                light.outer_angle,
            )
            .to_raw()
        })
        .collect::<Vec<_>>();

    // *EXTREMELY IMPORTANT!*
    // you should *CAST* the usize from the vector lenght to *U32* because shader uses u32 and there is no way to use usize there
    render_state
        .point_light_count_buffer
        .set_data(point_lights.len() as u32);
    render_state
        .spot_light_count_buffer
        .set_data(spot_lights.len() as u32);

    render_state.point_light_buffer.set_data_vec(point_lights);
    render_state.spot_light_buffer.set_data_vec(spot_lights);
}
#[system_fn]
pub fn make_light_follow_camera(
    mut camera: SystemQuery<(&Transform, &Camera)>,
    mut light: SystemQuery<&mut Transform, With<SpotLight>>,
    keyboard: Res<KeyboardInput>,
    mut should_sync: Local<bool>,
) {
    if keyboard.is_pressed(VirtualKeyCode::F) {
        *should_sync = !*should_sync;
    }
    if !*should_sync {
        return;
    }

    if camera.get_single().is_err() {
        return;
    }
    let (transform, camera) = camera.single();
    let mut light = match light.get_single_mut() {
        Ok(x) => x,
        Err(_) => return,
    };

    let light = &mut light.iso;

    light.translation.x = transform.iso.translation.x;
    light.translation.y = transform.iso.translation.y;
    light.translation.z = transform.iso.translation.z;
    light.rotation = camera.quat;
}
