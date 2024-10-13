const std = @import("std");
const ecs = @import("lib.zig");
const utils = @import("utils");
const hashset = @import("hashset");

pub const ArchetypeId = u64;
pub const RowId = usize;
pub const void_archetype_id = std.math.maxInt(ArchetypeId);

pub const ArchetypeRegistry = struct {
    curr_tick: ecs.Tick = ecs.Tick.new(0, 0),
    alloc: std.mem.Allocator,
    archetypes: std.AutoArrayHashMapUnmanaged(ArchetypeId, ArchetypeStorage) = .{},
    entity_counter: ecs.Entity = 0,
    entity_map: std.AutoArrayHashMapUnmanaged(ecs.Entity, ecs.EntityIndex) = .{},
    added_components: std.AutoHashMapUnmanaged(utils.TypeId, hashset.HashSetUnmanaged(ecs.EntityIndex)) = .{},
    removed_components: std.AutoHashMapUnmanaged(utils.TypeId, hashset.HashSetUnmanaged(ecs.Entity)) = .{},

    const Self = @This();

    pub fn init_ptr(alloc: std.mem.Allocator) *Self {
        var self = alloc.create(Self) catch unreachable;
        self.* = Self{
            .alloc = alloc,
        };

        self.archetypes.put(alloc, void_archetype_id, ArchetypeStorage.init(self.alloc, self)) catch unreachable;

        return self;
    }
    pub fn deinit(self: *Self) void {
        var it = self.archetypes.iterator();
        while (it.next()) |x| {
            x.value_ptr.deinit();
        }
        self.archetypes.deinit(self.alloc);
    }
    pub fn inc_tick(self: *Self) void {
        self.curr_tick.inc_tick();
        self.added_components.clearAndFree(self.alloc);
        self.removed_components.clearAndFree(self.alloc);
    }
    pub fn inc_subtick(self: *Self) void {
        self.curr_tick.inc_subtick();
    }
    pub fn get_entity_handle(self: *Self, entity: ecs.Entity) ?ecs.EntityHandle {
        const index = self.entity_map.get(entity) orelse return null;
        return ecs.EntityHandle{
            .created = self.curr_tick,
            .registry = self,
            .archetype = index.archetype,
            .row = index.row,
        };
    }
    pub fn new_entity(self: *Self) ecs.Entity {
        const new_id = self.entity_counter;
        self.entity_counter += 1;

        var void_archetype = self.archetypes.getPtr(void_archetype_id) orelse std.debug.panic("Archetype Registry wasnt inited", .{});
        const new_row = void_archetype.new_row(new_id);

        self.entity_map.put(self.alloc, new_id, ecs.EntityIndex{ .archetype = void_archetype_id, .row = new_row }) catch unreachable;
        return new_id;
    }
    pub fn add_component(self: *Self, entity: ecs.Entity, comp: anytype) ?void {
        const type_id = utils.type_id(@TypeOf(comp));

        const entity_handle = self.get_entity_handle(entity) orelse std.debug.panic("tried to use an unregistered Enitty, entities must only be created using ArchtypeRegistry.new_entity", .{});

        const curr_archetype = self.archetypes.getEntry(entity_handle.archetype).?.value_ptr;
        curr_archetype.registry = self;
        if (curr_archetype.components.contains(type_id)) {
            return null;
        }
        var archetype_ids = std.ArrayList(utils.TypeId).init(self.alloc);
        //archetype_ids.insertSlice(0, curr_archetype.components.keys()) catch unreachable;

        for (curr_archetype.components.keys()) |comp_id| {
            archetype_ids.append(comp_id) catch unreachable;
        }
        archetype_ids.append(utils.type_id(@TypeOf(comp))) catch unreachable;

        const archetype_id = ArchetypeStorage.calc_id_arr(archetype_ids.items);
        var archetype: *ArchetypeStorage = undefined;
        var storage: *ComponentStorage(@TypeOf(comp)) = undefined;

        // setting up the layout of the archetypes, not acutally putting value to them
        {
            if (!self.archetypes.contains(archetype_id)) {
                //
                archetype = ArchetypeStorage.init_ptr(self.alloc, self);
                for (curr_archetype.components.keys(), curr_archetype.components.values()) |comp_id, *_curr_storage| {
                    const new_storage = self.alloc.create(GenericComponentStorage) catch unreachable;
                    const curr_storage: *GenericComponentStorage = _curr_storage;

                    curr_storage.clone_type(new_storage);

                    archetype.components.put(self.alloc, comp_id, new_storage.*) catch unreachable;
                }
                storage = ComponentStorage(@TypeOf(comp)).init_ptr(self.alloc, self);
                archetype.components.put(self.alloc, type_id, storage.generic()) catch unreachable;
            } else {
                const a = self.archetypes.getEntry(archetype_id).?.value_ptr;
                a.registry = self;
                archetype = a;
                storage = a.components.getEntry(type_id).?.value_ptr.cast(@TypeOf(comp));
            }
        }

        //const generic_storage: *GenericComponentStorage = archetype.components.getEntry(type_id).?.value_ptr;

        //const storage = new_archetype.components.get(type_id) orelse unreachable;
        const new_row_id = archetype.push_empty_row();
        storage.set(new_row_id, comp);
        //std.debug.assert(new_row_id == 0);
        archetype.id = archetype_id;
        //std.debug.print("{}\n", .{self.entity_map.keys().len});
        curr_archetype.move_row(
            entity_handle.row,
            archetype,
            new_row_id,
        );
        const index = self.entity_map.getPtr(entity).?;
        index.archetype = archetype_id;
        index.row = new_row_id;
        self.archetypes.put(self.alloc, archetype_id, archetype.*) catch unreachable;

        const res = self.added_components.getOrPut(self.alloc, type_id) catch unreachable;
        if (!res.found_existing) {
            res.value_ptr.* = hashset.HashSetUnmanaged(ecs.EntityIndex).init();
        }
        _ = res.value_ptr.add(
            self.alloc,
            ecs.EntityIndex{ .row = new_row_id, .archetype = archetype.id },
        ) catch unreachable;
    }
    pub fn remove_component(self: *Self, entity: ecs.Entity, comp: type) void {
        const type_id = utils.type_id(comp);
        const entity_handle = self.entity_map.getPtr(entity).?.*;
        {
            const curr_archetype = self.archetypes.getEntry(entity_handle.archetype).?.value_ptr;

            var archetype_ids = std.ArrayList(utils.TypeId).init(self.alloc);
            for (curr_archetype.components.keys()) |id| {
                if (id != type_id) {
                    archetype_ids.append(id) catch unreachable;
                }
            }

            for (curr_archetype.components.keys()) |comp_id| {
                archetype_ids.append(comp_id) catch unreachable;
            }

            const archetype_id = ArchetypeStorage.calc_id_arr(archetype_ids.items);
            var archetype: *ArchetypeStorage = undefined;

            // setting up the layout of the archetypes, not acutally putting value to them
            {
                if (!self.archetypes.contains(archetype_id)) {
                    //
                    archetype = ArchetypeStorage.init_ptr(self.alloc, self);
                    for (curr_archetype.components.keys(), curr_archetype.components.values()) |comp_id, *_curr_storage| {
                        if (comp_id == type_id) {
                            continue;
                        }
                        const new_storage = self.alloc.create(GenericComponentStorage) catch unreachable;
                        const curr_storage: *GenericComponentStorage = _curr_storage;

                        curr_storage.clone_type(new_storage);

                        archetype.components.put(self.alloc, comp_id, new_storage.*) catch unreachable;
                    }
                } else {
                    const a = self.archetypes.getEntry(archetype_id).?.value_ptr;
                    archetype = a;
                }
            }

            const new_row_id = archetype.push_empty_row();
            //std.debug.assert(new_row_id == 0);
            archetype.id = archetype_id;
            curr_archetype.move_row_except(entity_handle.row, archetype, new_row_id, type_id);
            const index = self.entity_map.getPtr(entity).?;
            index.archetype = archetype_id;
            index.row = new_row_id;
            self.archetypes.put(self.alloc, archetype_id, archetype.*) catch unreachable;
        }

        {
            const res = self.removed_components.getOrPut(self.alloc, type_id) catch unreachable;
            if (!res.found_existing) {
                res.value_ptr.* = hashset.HashSetUnmanaged(ecs.Entity).init();
            }
            _ = res.value_ptr.add(
                self.alloc,
                entity,
            ) catch unreachable;
        }
    }
};
pub fn TypeArrFromTuple(value: anytype) []const type {
    const info = @typeInfo(@TypeOf(value));

    var arr: [info.Struct.fields.len]type = undefined;
    inline for (info.Struct.fields, 0..) |field, i| {
        const t = field.type;
        arr[i] = t;
    }
    return arr[0..];
}
pub const ArchetypeStorage = struct {
    alloc: std.mem.Allocator,
    components: std.AutoArrayHashMapUnmanaged(
        utils.TypeId,
        GenericComponentStorage,
    ),
    entity_ids: std.ArrayListUnmanaged(ecs.Entity) = .{},
    registry: *ecs.ArchetypeRegistry,
    id: ArchetypeId,

    const Self = @This();

    pub fn init(alloc: std.mem.Allocator, reg: *ArchetypeRegistry) Self {
        return Self{
            .alloc = alloc,
            .components = .{},
            .entity_ids = .{},
            .id = void_archetype_id,
            .registry = reg,
        };
    }
    pub fn init_ptr(alloc: std.mem.Allocator, reg: *ArchetypeRegistry) *Self {
        const mem = alloc.create(Self) catch unreachable;
        mem.* = Self.init(alloc, reg);
        return mem;
    }
    pub fn deinit(self: *Self) void {
        for (self.components.values()) |x| {
            var comp_storage: GenericComponentStorage = x;
            comp_storage.deinit();
        }
        self.components.deinit(self.alloc);
        self.entity_ids.deinit(self.alloc);
    }
    pub fn new_row(self: *Self, entity: ecs.Entity) ecs.RowId {
        const new_row_id = self.entity_ids.items.len;
        self.entity_ids.append(self.alloc, entity) catch unreachable;

        return new_row_id;
    }
    // Wrappers should handle change detection
    pub fn get_row(self: *Self, types: []const type, row: ecs.RowId) ?std.meta.Tuple(types) {
        var tuple: std.meta.Tuple(types) = undefined;

        inline for (types, 0..) |t_ptr, i| {
            const info = @typeInfo(t_ptr);
            const t = info.Pointer.child;
            const type_id = utils.type_id(t);

            const entry = self.components.getEntry(type_id) orelse {
                std.debug.print("did not find column with type name: {s}, the archetype contained the following columns:\n", .{@typeName(t)});

                inline for (types) |_t_ptr| {
                    _ = _t_ptr;
                    //const _info = @typeInfo(_t_ptr);
                    //const _t = _info.Pointer.child;
                    //std.debug.print("{s}\n", .{@typeName(_t)});
                }

                return null;
            };
            const storage = entry.value_ptr.cast(t);

            tuple[i] = storage.get_ptr(row);
        }

        return tuple;
    }
    pub fn copy_row(self: *const Self, soruce_row: ecs.RowId, dst: *Self, dst_row: ecs.RowId) void {
        for (self.components.keys(), self.components.values()) |column_id, *src_column| {
            src_column.copy(soruce_row, dst.components.getEntry(column_id).?.value_ptr, dst_row);
        }
        dst.entity_ids.items[dst_row] = self.entity_ids.items[soruce_row];
    }
    pub fn move_row(self: *Self, soruce_row: ecs.RowId, dst: *Self, dst_row: ecs.RowId) void {
        //std.debug.print("{}\n", .{self.registry.entity_map.keys().len});
        for (self.components.keys(), self.components.values()) |column_id, *src_column| {
            src_column.copy(soruce_row, dst.components.getEntry(column_id).?.value_ptr, dst_row);
        }
        dst.entity_ids.items[dst_row] = self.entity_ids.items[soruce_row];

        self.remove_row(soruce_row);
    }
    pub fn move_row_except(self: *Self, soruce_row: ecs.RowId, dst: *Self, dst_row: ecs.RowId, comp: utils.TypeId) void {
        for (self.components.keys(), self.components.values()) |column_id, *src_column| {
            if (column_id == comp) {
                continue;
            }
            src_column.copy(soruce_row, dst.components.getEntry(column_id).?.value_ptr, dst_row);
        }
        dst.entity_ids.items[dst_row] = self.entity_ids.items[soruce_row];

        self.remove_row(soruce_row);
    }
    pub fn push_empty_row(self: *Self) ecs.RowId {
        for (self.components.values()) |*column| {
            _ = column.push_empty();
        }
        self.entity_ids.append(self.alloc, undefined) catch unreachable;
        return self.entity_ids.items.len - 1;
    }
    pub fn remove_row(self: *Self, row: ecs.RowId) void {
        const moved_entity = self.entity_ids.items[self.entity_ids.items.len - 1];
        for (self.components.values()) |*column| {
            column.remove(row);
        }
        _ = self.entity_ids.swapRemove(row);
        const handle = self.registry.entity_map.getPtr(moved_entity) orelse {
            std.debug.panic("handle was not there for entity: {}", .{moved_entity});
        };
        handle.row = row;
    }
    pub fn calc_id(self: *const Self) ecs.ArchetypeId {
        var hasher = std.hash.Wyhash.init(0);
        const type_ids = utils.clone_slice(utils.TypeId, self.components.keys(), self.alloc) catch unreachable;

        const ctx: u1 = 0;
        std.sort.insertion(u64, type_ids, ctx, sort_fn);

        for (type_ids) |x| {
            std.hash.autoHash(&hasher, x);
        }
        return hasher.final();
    }
    pub fn calc_id_arr(type_ids: []utils.TypeId) ecs.ArchetypeId {
        var hasher = std.hash.Wyhash.init(0);

        const ctx: u1 = 0;
        std.sort.insertion(u64, type_ids, ctx, sort_fn);

        for (type_ids) |x| {
            std.hash.autoHash(&hasher, x);
        }
        return hasher.final();
    }
};
fn sort_fn(_: u1, lhs: u64, rhs: u64) bool {
    return lhs < rhs;
}

pub fn ComponentStorageData(T: type) type {
    return struct {
        data: T,
        change_tick: ecs.Tick,

        const Self = @This();

        pub fn new(data: T, tick: ecs.Tick) Self {
            return Self{ .data = data, .change_tick = tick };
        }
    };
}
pub fn ComponentStorage(T: type) type {
    return struct {
        alloc: std.mem.Allocator,
        data: std.ArrayListUnmanaged(ComponentStorageData(T)),
        registry: *ecs.ArchetypeRegistry,

        const Self = @This();

        pub fn init(alloc: std.mem.Allocator, reg: *ecs.ArchetypeRegistry) Self {
            return Self{
                .alloc = alloc,
                .data = std.ArrayListUnmanaged(ComponentStorageData(T)).initCapacity(alloc, 8) catch unreachable,
                .registry = reg,
            };
        }
        pub fn init_ptr(alloc: std.mem.Allocator, reg: *ecs.ArchetypeRegistry) *Self {
            const mem = alloc.create(Self) catch unreachable;
            mem.* = Self.init(alloc, reg);
            return mem;
        }
        pub fn deinit(_: *Self) void {

            //self.data.deinit(self.alloc);
        }
        pub fn generic(self: *Self) GenericComponentStorage {
            return GenericComponentStorage{ .storage = self, .deinit_ptr = deinit_wrapper, .clone_type_ptr = clone_type_wrapper, .copy_ptr = copy_wrapper, .remove_ptr = remove_wrapper, .push_empty_ptr = push_empty_wrapper, .get_len_ptr = get_len_wrapper };
        }
        pub fn deinit_wrapper(ptr: *anyopaque) void {
            const self: *Self = @ptrCast(@alignCast(ptr));

            self.deinit();
        }
        pub inline fn get(self: *Self, i: ecs.RowId) T {
            return self.data.items[i].data;
        }
        pub inline fn get_change_tick(self: *const Self, i: ecs.RowId) ecs.Tick {
            return self.data.items[i].change_tick;
        }
        // Wrappers should handle change tick changes
        pub inline fn get_ptr(self: *Self, i: ecs.RowId) *T {
            return &self.data.items[i].data;
        }
        pub inline fn get_const(self: *Self, i: ecs.RowId) *const T {
            return &self.data.items[i].data;
        }
        // Wrappers should handle change tick changes
        pub inline fn set(self: *Self, i: ecs.RowId, value: T) void {
            self.data.items[i] = ComponentStorageData(T).new(value, .{});
        }
        pub inline fn set_change_tick(self: *Self, i: ecs.RowId, tick: ecs.Tick) void {
            self.data.items[i].change_tick = tick;
        }
        pub fn clone_type(self: *const Self, to: *GenericComponentStorage) void {
            var mem = self.alloc.create(ComponentStorage(T)) catch unreachable;
            mem.* = ComponentStorage(T).init(self.alloc, self.registry);
            to.* = mem.generic();
        }
        pub fn clone_type_wrapper(ptr: *const anyopaque, to: *GenericComponentStorage) void {
            const self: *const Self = @ptrCast(@alignCast(ptr));

            self.clone_type(to);
        }
        pub fn copy(self: *const Self, source_row: ecs.RowId, to: *Self, dst_row: ecs.RowId) ?void {
            if (to.data.items.len <= dst_row) {
                return null;
            }
            to.data.items[dst_row] = self.data.items[source_row];
        }
        pub fn copy_wrapper(ptr: *const anyopaque, source_row: ecs.RowId, to_ptr: *GenericComponentStorage, dst_row: ecs.RowId) void {
            const self: *const Self = @ptrCast(@alignCast(ptr));
            const to: *Self = @ptrCast(@alignCast(to_ptr.storage));

            self.copy(source_row, to, dst_row) orelse unreachable;
        }
        pub fn remove(self: *Self, row: ecs.RowId) void {
            _ = self.data.swapRemove(row);
        }
        pub fn remove_wrapper(ptr: *anyopaque, row: ecs.RowId) void {
            const self: *Self = @ptrCast(@alignCast(ptr));

            self.remove(row);
        }
        pub fn push_empty(self: *Self) ecs.RowId {
            _ = self.data.addOne(self.alloc) catch unreachable;
            return self.data.items.len - 1;
        }
        pub fn push_empty_wrapper(ptr: *anyopaque) ecs.RowId {
            const self: *Self = @ptrCast(@alignCast(ptr));

            return self.push_empty();
        }
        pub fn get_len(self: *const Self) usize {
            return self.data.items.len;
        }
        pub fn get_len_wrapper(ptr: *const anyopaque) ecs.RowId {
            const self: *const Self = @ptrCast(@alignCast(ptr));

            return self.get_len();
        }
    };
}
pub const GenericComponentStorage = struct {
    storage: *anyopaque,
    deinit_ptr: *const fn (*anyopaque) void,
    clone_type_ptr: *const fn (*const anyopaque, *GenericComponentStorage) void,
    copy_ptr: *const fn (*const anyopaque, ecs.RowId, *GenericComponentStorage, ecs.RowId) void,
    remove_ptr: *const fn (*anyopaque, ecs.RowId) void,
    push_empty_ptr: *const fn (*anyopaque) ecs.RowId,
    get_len_ptr: *const fn (*const anyopaque) usize,
    //push_empty: *const fn (*anyopaque) ecs.RowId,

    const Self = @This();

    pub fn cast(self: *Self, comptime T: type) *ComponentStorage(T) {
        return @ptrCast(@alignCast(self.storage));
    }
    pub fn deinit(self: *Self) void {
        self.deinit_ptr(self.storage);
    }
    pub fn clone_type(from: *const Self, to: *GenericComponentStorage) void {
        from.clone_type_ptr(from.storage, to);
    }
    pub fn copy(from: *const Self, source_row: ecs.RowId, to: *GenericComponentStorage, dst_row: ecs.RowId) void {
        from.copy_ptr(from.storage, source_row, to, dst_row);
    }
    pub fn remove(self: *Self, row: ecs.RowId) void {
        self.remove_ptr(self.storage, row);
    }
    pub fn push_empty(self: *Self) ecs.RowId {
        return self.push_empty_ptr(self.storage);
    }
    pub fn get_len(self: *const Self) usize {
        return self.get_len_ptr(self.storage);
    }
};
