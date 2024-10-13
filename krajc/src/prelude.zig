pub const main = @import("main.zig");
pub const utils = @import("utils.zig");
pub const ecs = @import("ecs/prelude.zig");
pub const rendering = @import("rendering/prelude.zig");

pub const RenderingState = rendering.RenderingState;
pub const ResourceState = main.ResourceState;

//
pub const System = ecs.System;
pub const SystemParam = ecs.SystemParam;
pub const FunctionSystem = ecs.FunctionSystem;
pub const schedule = ecs.schedule;
pub const register = ecs.register;
//
pub const Res = ecs.Res;
pub const Query = ecs.Query;

pub const ArchetypeId = ecs.ArchetypeId;
pub const RowId = ecs.RowId;
pub const ArchetypeRegistry = ecs.ArchetypeRegistry;
pub const ArchetypeStorage = ecs.ArchetypeStorage;
pub const ComponentStorage = ecs.ComponentStorage;
pub const GenericComponentStorage = ecs.GenericComponentStorage;
//
pub const Entity = ecs.Entity;
pub const EntityHandle = ecs.EntityHandle;
pub const EntityIndex = ecs.EntityIndex;
//
pub const View = ecs.View;
pub const With = ecs.With;
pub const Without = ecs.Without;
pub const Changed = ecs.Changed;
pub const Added = ecs.Added;
pub const Removed = ecs.Removed;
pub const Or = ecs.Or;
pub const Not = ecs.Not;
//
pub const Tick = ecs.Tick;
