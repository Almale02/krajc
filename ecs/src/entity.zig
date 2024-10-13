const std = @import("std");
const ecs = @import("lib.zig");
const utils = @import("utils");

pub const Entity = u64;

pub const EntityIndex = struct {
    archetype: ecs.ArchetypeId,
    row: ecs.RowId,
};

pub const EntityHandle = struct {
    created: ecs.Tick,
    registry: *ecs.ArchetypeRegistry,
    archetype: ecs.ArchetypeId,
    row: ecs.RowId,

    const Self = @This();

    pub fn init(reg: ecs.ArchetypeRegistry, archetype: ecs.ArchetypeId, row: ecs.RowId, tick: ecs.Tick) Self {
        return Self{
            .created = tick,
            .registry = reg,
            .archetype = archetype,
            .row = row,
        };
    }
    pub fn is_valid(self: *const Self, curr_tick: ecs.Tick) bool {
        if (self.created.older_than(curr_tick)) {
            return false;
        }
        if (self.created.newer_than(curr_tick)) {
            std.debug.panic(
                "Invalid ticks, `EntityHandle`s creation tick cannot be newer than the usage tick of the `EntityHanle`",
            );
        }
        return true;
    }
    pub fn get(self: *Self, T: type) ?*T {
        const arch = self.registry.archetypes.getEntry(self.archetype).?.value_ptr;
        const storage_entry = arch.components.getEntry(utils.type_id(T)) orelse return null;
        const storage: *ecs.GenericComponentStorage = storage_entry.value_ptr;
        return storage.cast(T).get_ptr(self.row);
    }
};
