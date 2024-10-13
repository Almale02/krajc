const std = @import("std");

// Although this function looks imperative, note that its job is to
// declaratively construct a build graph that will be executed by an external
// runner.
pub fn build(b: *std.Build) void {
    // Standard target options allows the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});

    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});

    const utils_dep = b.dependency("utils", .{});
    // Add module to your exe (wgpu-c can also be added like this, just pass in "wgpu-c" instead)

    const hashset_dep = b.dependency("hashset", .{});

    var lib = b.addModule("ecs", .{
        .root_source_file = b.path("src/lib.zig"),
        .target = target,
        .optimize = optimize,
    });
    // const lib = b.addStaticLibrary(.{
    //     .name = "ecs",
    //     .root_source_file = b.path("src/lib.zig"),
    //     .target = target,
    //     .optimize = optimize,
    // });
    lib.addImport("utils", utils_dep.module("utils"));
    lib.addImport("hashset", hashset_dep.module("ziglangSet"));

    //b.installArtifact(lib);
    // C:\Users\gaspa\Desktop\zig\ecs
    const examples_dir = std.fs.cwd().openDir("src/examples", std.fs.Dir.OpenDirOptions{ .iterate = true, .access_sub_paths = true }) catch return;

    var it = examples_dir.iterate();

    while (it.next() catch unreachable) |x| {
        const entry: std.fs.Dir.Entry = x;

        switch (entry.kind) {
            std.fs.Dir.Entry.Kind.file => {
                const abs_name = examples_dir.realpathAlloc(b.allocator, entry.name) catch unreachable;

                if (std.mem.eql(u8, std.fs.path.extension(entry.name), ".zig")) {
                    const example = b.addExecutable(.{ .name = std.fmt.allocPrint(b.allocator, "example-{s}", .{std.fs.path.basename(entry.name)}) catch unreachable, .root_source_file = b.path(std.fs.path.relative(b.allocator, std.fs.cwd().realpathAlloc(b.allocator, ".") catch unreachable, abs_name) catch unreachable), .target = target, .optimize = optimize });
                    example.root_module.addImport("ecs", lib);

                    example.root_module.addImport("hashset", hashset_dep.module("ziglangSet"));
                    example.root_module.addImport("utils", utils_dep.module("utils"));
                    b.installArtifact(example);
                    const run_example_cmd = b.addRunArtifact(example);
                    run_example_cmd.step.dependOn(b.getInstallStep());
                    const run_example_step = b.step(std.fmt.allocPrint(b.allocator, "run-example-{s}", .{stripExtension(std.fs.path.basename(entry.name))}) catch unreachable, "Run an example");
                    run_example_step.dependOn(&run_example_cmd.step);
                }
            },
            else => {},
        }
    }

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
}
pub fn stripExtension(file_name: []const u8) []const u8 {
    // Find the last occurrence of '.' in the filename to remove the extension
    const dot_pos = std.mem.lastIndexOf(u8, file_name, ".");

    // Return the filename without the extension
    return if (dot_pos != null) file_name[0..dot_pos.?] else file_name;
}
