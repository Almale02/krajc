use crate::marker_comps;
use bevy_ecs::{component::Component, query::Without};
use glyphon::Attrs;
use krajc_macros::{system_fn, EngineResource};

use crate::{
    engine_runtime::schedule_manager::system_params::{
        system_local::Local, system_query::SystemQuery, system_resource::Res,
    },
    typed_addr::dupe,
};

use super::managers::RenderManagerResource;

#[derive(Default, Debug, EngineResource, PartialEq, PartialOrd)]
pub struct DebugTextProducer(u16);

impl DebugTextProducer {
    pub fn create_text(&mut self, init_text: &str) -> DebugText {
        let id = self.0;
        self.0 += 1;
        DebugText {
            id,
            text: init_text.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Component, Eq)]
pub struct DebugText {
    pub(crate) id: u16,
    pub text: String,
}

impl DebugText {
    pub fn new(id: u16, text: String) -> Self {
        Self { id, text }
    }
}
impl PartialOrd for DebugText {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}
impl PartialEq for DebugText {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Ord for DebugText {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[system_fn]
pub fn update_debug_text(
    mut render: Res<RenderManagerResource>,
    mut query: SystemQuery<&mut DebugText, Without<Disabled>>,
    mut text: Local<String>,
) {
    let mut texts = query.iter().collect::<Vec<_>>();
    texts.sort();

    dupe(&render).text_state.buffer.set_text(
        render.text_state.font_system,
        {
            let mut string = String::new();
            texts.iter().for_each(|x| {
                string.push_str(&x.text);
                string.push('\n');
            });
            *text = string;
            &text
        },
        Attrs::new().family(glyphon::Family::SansSerif),
        glyphon::Shaping::Advanced,
    )
}

#[macro_export]
macro_rules! marker_comps {
    ($($type: ident),*) => {
        $(
            #[derive(bevy_ecs::prelude::Component)]
            pub struct $type;
        )*
    };
}
marker_comps!(Disabled, FpsText, MouseMotionText);
