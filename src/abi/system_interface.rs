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
    typed_addr::TypedAddr,
    ENGINE_RUNTIME,
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
    pub register_systems: extern fn(SystemPluginRegister),
}

#[stabby]
pub struct SystemPluginRegister {
    _d: i32,
    engine: TypedAddr<EngineRuntime>,
}

impl SystemPluginRegister {
    pub fn new() -> Self {
        SystemPluginRegister {
            _d: 0,
            engine: unsafe { ENGINE_RUNTIME.clone() },
        }
    }
    pub fn start_register<Sched: Schedule>(&self) -> ScheduleSystemRegister<Sched> {
        let engine = self.engine.clone();
        ScheduleSystemRegister::new(engine)
    }
    pub fn bubu(&self) {
        dbg!("nana");
    }
}

impl Default for SystemPluginRegister {
    fn default() -> Self {
        Self::new()
    }
}

#[stabby]
#[derive(Clone)]
pub struct ScheduleSystemRegister<Sched: Schedule> {
    engine: TypedAddr<EngineRuntime>,
    _p: PhantomData<Sched>,
}

impl<Sched: Schedule> ScheduleSystemRegister<Sched> {
    pub fn new(engine: TypedAddr<EngineRuntime>) -> ScheduleSystemRegister<Sched> {
        Self {
            _p: PhantomData,
            engine,
        }
    }
    pub fn register<Func: 'static, Marker: 'static>(
        &self,
        system: FunctionSystem<Func, Marker>,
    ) -> Self
    where
        FunctionSystem<Func, Marker>: ScheduleRunnable,
    {
        let engine = self.engine.get();
        engine.register_system::<Sched>(system);
        dbg!("registered system");

        ScheduleSystemRegister::new(self.engine.clone())
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
