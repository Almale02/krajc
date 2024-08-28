use core::panic;
use std::{fs, marker::PhantomData, path::PathBuf, str::FromStr};

use crate::{
    engine_runtime::{
        schedule_manager::{
            schedule::{Schedule, ScheduleRunnable},
            system_params::system_param::FunctionSystem,
        },
        EngineRuntime,
    },
    typed_addr::{dupe, TypedAddr},
    Takeable, CUSTOM_SYSTEM, ENGINE_RUNTIME,
};

use super::prelude::*;

/// In your plugin inplementation you have to create an extern function call `get_plugin` which returns the `SystemPlugin` instance
///
/// # Implementation
/// ```
/// #[stabby::export]
/// #[stabby]
/// pub extern fn get_plugin() -> SystemPlugin;
///
/// ```

#[stabby]
pub struct SystemPlugin {
    pub register_systems: extern fn(TypedAddr<EngineRuntime>, SystemPluginRegister, TypedAddr<u32>),
}

#[stabby]
pub struct SystemPluginRegister {
    _d: i32,
    custom: &'static mut TypedAddr<Takeable<Box<dyn ScheduleRunnable>>>,
    engine: TypedAddr<EngineRuntime>,
}

impl SystemPluginRegister {
    pub fn new(
        engine: TypedAddr<EngineRuntime>,
        custom: &'static mut TypedAddr<Takeable<Box<dyn ScheduleRunnable>>>,
    ) -> Self {
        SystemPluginRegister {
            _d: 0,
            engine,
            custom,
        }
    }
    pub fn start_register<Sched: Schedule>(&self) -> ScheduleSystemRegister<Sched> {
        ScheduleSystemRegister::new(self.engine.clone(), dupe(self.custom))
    }
    pub fn bubu(&self) {
        dbg!("nana");
    }
}

#[stabby]
pub struct ScheduleSystemRegister<Sched: Schedule> {
    custom: &'static mut TypedAddr<Takeable<Box<dyn ScheduleRunnable>>>,
    engine: TypedAddr<EngineRuntime>,
    _p: PhantomData<Sched>,
}

impl<Sched: Schedule> ScheduleSystemRegister<Sched> {
    pub fn new(
        engine: TypedAddr<EngineRuntime>,
        custom: &'static mut TypedAddr<Takeable<Box<dyn ScheduleRunnable>>>,
    ) -> ScheduleSystemRegister<Sched> {
        Self {
            _p: PhantomData,
            engine,
            custom,
        }
    }
    pub fn register<Func: Clone + 'static, Marker: 'static>(
        &mut self,
        system: FunctionSystem<Func, Marker>,
    ) where
        FunctionSystem<Func, Marker>: ScheduleRunnable,
    {
        self.engine.get().register_system::<Sched>(system.clone());
        *self.custom =
            TypedAddr::new_with_ref(Box::leak(Box::new(Takeable::new(Box::new(system)))));
    }
}

pub fn get_game_plugin_path(game: String) -> PathBuf {
    let games = dirs_next::data_local_dir().unwrap();
    let games = games.join("KrajcEngine/installed_games.txt");

    let paths = fs::read_to_string(games.clone()).unwrap_or_else(|_| {
        panic!("no games were installed, consider adding your game to the following txt file: data_local_dir/KrajcEngine/installed_games.txt, the install process should have automatically done this");
    });
    let paths = paths
        .split('\n')
        .map(|x| {
            let path = PathBuf::from_str(x).unwrap_or_else(|_| {
                panic!(
                    "game path `{}` in the games list `data_local_dir/KrajcEngine/installed_games.txt` is malformated",
                    x
                )
            });
            if !path.exists() {
                panic!(
                    "game path `{}` in the games list `data_local_dir/KrajcEngine/installed_games.txt` doesnt exists",
                    x
                );
            }
            path
        })
        .collect::<Vec<_>>();

    let mut game_path = None;
    paths.iter().for_each(|x| {
        if x.ends_with(format!("{}.dll", game)) {
            game_path = Some(x)
        }
    });
    if game_path.is_none() {
        panic!("game {} wasnt found in games list `data_local_dir/KrajcEngine/installed_games.txt`, the install process should have already added this, consider adding it; here are all the games registered: {:?}", game, paths);
    }
    let game_path = game_path.unwrap();
    return game_path.clone();
}
