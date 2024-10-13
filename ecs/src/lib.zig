const std = @import("std");
pub const archetype = @import("archetype.zig");
const entity = @import("entity.zig");
const view = @import("view.zig");

pub const utils = @import("utils");

pub const ArchetypeId = archetype.ArchetypeId;
pub const RowId = archetype.RowId;
pub const ArchetypeRegistry = archetype.ArchetypeRegistry;
pub const ArchetypeStorage = archetype.ArchetypeStorage;
pub const ComponentStorage = archetype.ComponentStorage;
pub const GenericComponentStorage = archetype.GenericComponentStorage;
//
pub const Entity = entity.Entity;
pub const EntityHandle = entity.EntityHandle;
pub const EntityIndex = entity.EntityIndex;
//
pub const View = view.View;
pub const With = view.With;
pub const Without = view.Without;
pub const Changed = view.Changed;
pub const Added = view.Added;
pub const Removed = view.Removed;
pub const Or = view.Or;
pub const Not = view.Not;

pub const Tick = struct {
    tick: u64 = 0,
    subtick: u64 = 0,

    const Self = @This();

    pub fn new(tick: u64, subtick: u64) Self {
        return Self{ .tick = tick, .subtick = subtick };
    }
    pub fn inc_tick(self: *Self) void {
        self.tick += 1;
        self.subtick = 0;
    }
    pub fn inc_subtick(self: *Self) void {
        self.subtick += 1;
    }
    pub fn equal(self: *const Self, other: Self) bool {
        return other.tick == self.tick and self.subtick == other.subtick;
    }
    pub fn newer_than(self: *const Self, other: Self) bool {
        if (self.tick == other.tick) {
            return self.subtick > other.subtick;
        }
        return self.tick > other.tick;
    }
    pub fn older_than(self: *const Self, other: Self) bool {
        if (self.tick == other.tick) {
            return self.subtick < other.subtick;
        }
        return self.tick < other.tick;
    }
};
