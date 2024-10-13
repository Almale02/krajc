const std = @import("std");
const ecs = @import("lib.zig");
const utils = @import("utils");
const hashset = @import("hashset");

pub fn View(view: []const type, filters: []const type) type {
    return struct {
        system_last_run: ecs.Tick,
        registry: *ecs.ArchetypeRegistry,
        iter_archetype: usize = 0,
        iter_row: ecs.RowId = 0,
        viewed_archetypes: std.ArrayList(ecs.ArchetypeId),
        const Self = @This();

        pub fn init(registry: *ecs.ArchetypeRegistry, system_last_run: ecs.Tick) Self {
            var self = Self{ .registry = registry, .viewed_archetypes = std.ArrayList(ecs.ArchetypeId).init(registry.alloc), .system_last_run = system_last_run };

            for (self.registry.archetypes.keys(), self.registry.archetypes.values()) |archetype_id, *archetype| {
                var contains_all = true;

                inline for (view) |t| {
                    const ptr_info = @typeInfo(t);
                    const type_id = utils.type_id(ptr_info.Pointer.child);

                    if (!archetype.components.contains(type_id)) {
                        contains_all = false;
                    }
                }
                inline for (filters) |t| {
                    Filter.ensure_filter(t);
                    //const ptr_info = @typeInfo(t);
                    //const type_id = utils.type_id(ptr_info.Pointer.child);

                    if (Filter.access_archetype_filter(t)) |filter| {
                        if (!filter(archetype)) {
                            contains_all = false;
                        }
                    }
                }
                if (contains_all) {
                    if (self.registry.archetypes.getPtr(archetype_id).?.entity_ids.items.len != 0) {
                        self.viewed_archetypes.append(archetype_id) catch unreachable;
                    }
                }
            }

            return self;
        }
        pub fn reset(self: *Self) void {
            self.iter_archetype = 0;
            self.iter_row = 0;
        }
        pub const TypedAccess = struct {
            row: ecs.RowId,
            tick: ecs.Tick,
            archetype: *ecs.ArchetypeStorage,
            allowed_writes: hashset.HashSetManaged(utils.TypeId),
            allowed_reads: hashset.HashSetManaged(utils.TypeId),
            entity: ecs.Entity,

            pub fn get(self: *const TypedAccess, T: type) *T {
                if (!self.allowed_writes.contains(utils.type_id(T))) {
                    std.debug.panic("tried to access {s} in view, which was not defined in the query, or was set as readonly", .{@typeName(T)});
                } else {
                    self.archetype.components.getPtr(utils.type_id(T)).?.cast(T).set_change_tick(self.row, self.archetype.registry.curr_tick);
                    return self.archetype.components.getPtr(utils.type_id(T)).?.cast(T).get_ptr(self.row);
                }
            }
            pub fn get_const(self: *const TypedAccess, T: type) *const T {
                if (!self.allowed_reads.contains(utils.type_id(T))) {
                    std.debug.panic("tried to access {s} in view, which was not defined in the query", .{@typeName(T)});
                } else {
                    return self.archetype.components.getPtr(utils.type_id(T)).?.cast(T).get_const(self.row);
                }
            }
        };
        pub fn next(self: *Self) ?TypedAccess {

            // std.debug.print("got here with archetype: {}\n", .{self.registry.archetypes.getPtr(self.viewed_archetypes.items[self.iter_archetype]).?.id});
            if (self.viewed_archetypes.items.len == 0) {
                return null;
            }
            if (self.iter_row == self.registry.archetypes.getPtr(self.viewed_archetypes.items[self.iter_archetype]).?.entity_ids.items.len) {
                // std.debug.print("returned here", .{});
                return null;
            }
            var curr_archetype = self.registry.archetypes.getPtr(self.viewed_archetypes.items[self.iter_archetype]).?;

            // iterate until row found which matches filters
            while (true) {
                var is_in_filters = true;
                inline for (filters) |filter_t| {
                    if (Filter.access_entity_filter(filter_t)) |filter| {
                        if (!filter(curr_archetype, self.iter_row, FilterData{ .system_last_run = self.system_last_run })) {
                            //std.debug.print("wasnt in filter: {}\n", .{self.iter_row});
                            is_in_filters = false;
                        }
                    }
                }

                if (!is_in_filters) {
                    if (curr_archetype.entity_ids.items.len - 1 == self.iter_row) {
                        if (self.viewed_archetypes.items.len - 1 == self.iter_archetype) {
                            // std.debug.print("returned here with id: {}, with iter a: {}, view a len: {}", .{ curr_archetype.id, self.iter_archetype, self.viewed_archetypes.items.len });
                            return null;
                        } else {
                            self.iter_archetype += 1;
                            self.iter_row = 0;
                            curr_archetype = self.registry.archetypes.getPtr(self.viewed_archetypes.items[self.iter_archetype]).?;
                        }
                    } else {
                        self.iter_row += 1;
                    }
                } else {
                    break;
                }
            }

            curr_archetype = self.registry.archetypes.getPtr(self.viewed_archetypes.items[self.iter_archetype]).?;
            if (self.iter_row == curr_archetype.entity_ids.items.len) {
                //std.debug.print("returned here with id: {}, with iter a: {}, view a len: {}", .{ curr_archetype.id, self.iter_archetype, self.viewed_archetypes.items.len });
                return null;
            }
            var allowed_writes = hashset.HashSetManaged(utils.TypeId).init(self.registry.alloc);
            var allowed_reads = hashset.HashSetManaged(utils.TypeId).init(self.registry.alloc);

            inline for (view) |t_ptr| {
                const t_ptr_info = @typeInfo(t_ptr);
                const is_const = t_ptr_info.Pointer.is_const;
                const t = t_ptr_info.Pointer.child;
                const type_id = utils.type_id(t);

                if (is_const) {
                    _ = allowed_reads.add(type_id) catch unreachable;
                } else {
                    _ = allowed_reads.add(type_id) catch unreachable;
                    _ = allowed_writes.add(type_id) catch unreachable;
                }
            }

            const typed_access = TypedAccess{ .tick = self.registry.curr_tick, .row = self.iter_row, .archetype = curr_archetype, .allowed_writes = allowed_writes, .allowed_reads = allowed_reads, .entity = curr_archetype.entity_ids.items[self.iter_row] };

            if (curr_archetype.entity_ids.items.len - 1 == self.iter_row) {
                if (self.viewed_archetypes.items.len - 1 == self.iter_archetype) {
                    // setting next row to overflow so that it breaks the iterator at the next iteration but return the row in this one
                    self.iter_row += 1;
                } else {
                    self.iter_archetype += 1;
                    self.iter_row = 0;
                }
            } else {
                self.iter_row += 1;
            }
            return typed_access;
        }
    };
}
pub const FilterData = struct {
    system_last_run: ecs.Tick,
};
pub const Filter = struct {
    pub fn ensure_filter(T: type) void {
        if (access_archetype_filter(T) == null and access_entity_filter(T) == null) {
            @compileError(std.fmt.comptimePrint(
                \\to use {} as a query filter, you must implement atleast one of the following methods
                \\fn archetype_filter(*const ecs.ArchetypeStorage) bool,
                \\fn entity_filter(*const ecs.ArchetypeStorage, row: ecs.RowId, FilterData) bool
            , .{}));
        }
    }
    /// Filters the query based on the archetype
    pub fn access_archetype_filter(T: type) ?(fn (*const ecs.ArchetypeStorage) bool) {
        return utils.accessMethod(T, "archetype_filter", fn (*const ecs.ArchetypeStorage) bool);
    }
    /// Filters the entities individually in the query
    pub fn access_entity_filter(T: type) ?(fn (*const ecs.ArchetypeStorage, ecs.RowId, FilterData) bool) {
        return utils.accessMethod(T, "entity_filter", fn (*const ecs.ArchetypeStorage, ecs.RowId, FilterData) bool);
    }
};
pub fn With(T: type) type {
    const type_id = utils.type_id(T);

    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return a.components.contains(type_id);
        }
    };
}
pub fn Without(T: type) type {
    const type_id = utils.type_id(T);

    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return !a.components.contains(type_id);
        }
    };
}
pub fn Changed(T: type) type {
    const type_id = utils.type_id(T);
    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return a.components.contains(type_id);
        }
        pub fn entity_filter(a: *const ecs.ArchetypeStorage, row_id: ecs.RowId, data: FilterData) bool {
            const last_changed = a.components.getPtr(type_id).?.cast(T).get_change_tick(row_id);
            const passed = data.system_last_run.older_than(last_changed) or data.system_last_run.equal(last_changed);

            //std.debug.print("system last run: {}, component last modified: {}, row: {}, archetype: {}, passed: {}\n", .{ data.system_last_run, last_changed, row_id, a.id, passed });
            return passed;
        }
    };
}
pub fn Added(T: type) type {
    const type_id = utils.type_id(T);
    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return a.components.contains(type_id);
        }
        pub fn entity_filter(a: *const ecs.ArchetypeStorage, row_id: ecs.RowId, _: FilterData) bool {
            const contains_data = a.registry.added_components.getPtr(type_id) orelse return false;
            if (contains_data.contains(ecs.EntityIndex{ .row = row_id, .archetype = a.id })) {}
            return contains_data.contains(ecs.EntityIndex{ .row = row_id, .archetype = a.id });
        }
    };
}
pub fn Removed(T: type) type {
    const type_id = utils.type_id(T);
    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return !a.components.contains(type_id);
        }
        pub fn entity_filter(a: *const ecs.ArchetypeStorage, row_id: ecs.RowId, _: FilterData) bool {
            const removed_data = a.registry.removed_components.getPtr(type_id) orelse {
                return false;
            };
            const entity = a.entity_ids.items[row_id];
            return removed_data.contains(entity);
        }
    };
}

pub fn Or(A: type, B: type) type {
    Filter.ensure_filter(A);
    Filter.ensure_filter(B);

    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return Filter.access_archetype_filter(A).?(a) or Filter.access_archetype_filter(B).?(a);
        }
        pub fn entity_filter(a: *const ecs.ArchetypeStorage, row_id: ecs.RowId, data: FilterData) bool {
            return Filter.access_entity_filter(A).?(a, row_id, data) or Filter.access_entity_filter(B).?(a, row_id, data);
        }
    };
}
pub fn Not(T: type) type {
    Filter.ensure_filter(T);

    return struct {
        pub fn archetype_filter(a: *const ecs.ArchetypeStorage) bool {
            return Filter.access_archetype_filter(T).?(a);
        }
        pub fn entity_filter(a: *const ecs.ArchetypeStorage, row_id: ecs.RowId, data: FilterData) bool {
            return !Filter.access_entity_filter(T).?(a, row_id, data);
        }
    };
}
