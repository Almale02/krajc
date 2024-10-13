pub const krajc_ecs = @import("krajc_ecs");
//
pub const System = @import("systems/system.zig").System;
pub const SystemParam = @import("systems/system_param.zig").SystemParam;
pub const FunctionSystem = @import("systems/function_system.zig").FunctionSystem;
pub const schedule = @import("systems/schedule.zig");
pub const register = @import("systems/system.zig").register;
//
pub const Res = @import("systems/params/res.zig").Res;
pub const Query = @import("systems/params/query.zig").Query;

// Re-exports
pub const ArchetypeId = krajc_ecs.ArchetypeId;
pub const RowId = krajc_ecs.RowId;
pub const ArchetypeRegistry = krajc_ecs.ArchetypeRegistry;
pub const ArchetypeStorage = krajc_ecs.ArchetypeStorage;
pub const ComponentStorage = krajc_ecs.ComponentStorage;
pub const GenericComponentStorage = krajc_ecs.GenericComponentStorage;
//
pub const Entity = krajc_ecs.Entity;
pub const EntityHandle = krajc_ecs.EntityHandle;
pub const EntityIndex = krajc_ecs.EntityIndex;
//
pub const View = krajc_ecs.View;
pub const With = krajc_ecs.With;
pub const Without = krajc_ecs.Without;
pub const Changed = krajc_ecs.Changed;
pub const Added = krajc_ecs.Added;
pub const Removed = krajc_ecs.Removed;
pub const Or = krajc_ecs.Or;
pub const Not = krajc_ecs.Not;
//
pub const Tick = krajc_ecs.Tick;
