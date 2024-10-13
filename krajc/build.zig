const std = @import("std");
const gpu_dawn = @import("mach_gpu_dawn");

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

    const gpu_dawn_options = gpu_dawn.Options{
        .from_source = b.option(bool, "dawn-from-source", "Build Dawn from source") orelse false,
        .debug = b.option(bool, "dawn-debug", "Use a debug build of Dawn") orelse false,
    };

    const lib = b.addStaticLibrary(.{
        .name = "krajc",
        // In this case the main source file is merely a path, however, in more
        // complicated build scripts, this could be a generated file.
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // This declares intent for the library to be installed into the standard
    // location when the user invokes the "install" step (the default step when
    // running `zig build`).
    b.installArtifact(lib);

    const exe = b.addExecutable(.{
        .name = "krajc",
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    // This declares intent for the executable to be installed into the
    // standard location when the user invokes the "install" step (the default
    // step when running `zig build`).
    b.installArtifact(exe);

    try link(b, exe, &exe.root_module, .{ .gpu_dawn_options = gpu_dawn_options });
    try link(b, lib, &exe.root_module, .{ .gpu_dawn_options = gpu_dawn_options });

    //exe.root_module.addImport("ziglangSet", ziglangSet.module("ziglangSet"));

    const wgpu_native_dep = b.dependency("krajc_wgpu", .{});
    exe.root_module.addImport("wgpu", wgpu_native_dep.module("mach-gpu"));
    lib.root_module.addImport("wgpu", wgpu_native_dep.module("mach-gpu"));

    const zchan_dep = b.dependency("zchan", .{});
    // Add module to your exe (wgpu-c can also be added like this, just pass in "wgpu-c" instead)
    exe.root_module.addImport("zchan", zchan_dep.module("zchan"));
    lib.root_module.addImport("zchan", zchan_dep.module("zchan"));

    const glfw_dep = b.dependency("krajc_glfw", .{});
    exe.root_module.addImport("glfw", glfw_dep.module("mach-glfw"));
    lib.root_module.addImport("glfw", glfw_dep.module("mach-glfw"));

    const uuid_dep = b.dependency("uuid", .{});
    exe.root_module.addImport("uuid", uuid_dep.module("uuid"));
    lib.root_module.addImport("uuid", uuid_dep.module("uuid"));

    const hashset_dep = b.dependency("hashset", .{});
    exe.root_module.addImport("hashset", hashset_dep.module("ziglangSet"));
    lib.root_module.addImport("hashset", hashset_dep.module("ziglangSet"));

    const krajc_dep = b.dependency("krajc_ecs", .{});
    exe.root_module.addImport("krajc_ecs", krajc_dep.module("ecs"));
    lib.root_module.addImport("krajc_ecs", krajc_dep.module("ecs"));

    const zm_dep = b.dependency("zm", .{});
    exe.root_module.addImport("zm", zm_dep.module("zm"));
    lib.root_module.addImport("zm", zm_dep.module("zm"));

    exe.addCSourceFile(.{ .file = b.path("../krajc-wgpu/src/mach_dawn.cpp"), .flags = &.{"-std=c++17"} });
    lib.addCSourceFile(.{ .file = b.path("../krajc-wgpu/src/mach_dawn.cpp"), .flags = &.{"-std=c++17"} });
    exe.addIncludePath(b.path("../krajc-wgpu/src"));
    lib.addIncludePath(b.path("../krajc-wgpu/src"));

    // This *creates* a Run step in the build graph, to be executed when another
    // step is evaluated that depends on it. The next line below will establish
    // such a dependency.
    const run_cmd = b.addRunArtifact(exe);

    // By making the run step depend on the install step, it will be run from the
    // installation directory rather than directly from within the cache directory.
    // This is not necessary, however, if the application depends on other installed
    // files, this ensures they will be present and in the expected location.
    run_cmd.step.dependOn(b.getInstallStep());

    // This allows the user to pass arguments to the application in the build
    // command itself, like this: `zig build run -- arg1 arg2 etc`
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    // This creates a build step. It will be visible in the `zig build --help` menu,
    // and can be selected like this: `zig build run`
    // This will evaluate the `run` step rather than the default, which is "install".
    const run_step = b.step("run", "Run the app");
    run_step.dependOn(&run_cmd.step);

    const exe_unit_tests = b.addTest(.{
        .root_source_file = b.path("src/main.zig"),
        .target = target,
        .optimize = optimize,
    });

    const run_exe_unit_tests = b.addRunArtifact(exe_unit_tests);

    // Similar to creating the run step earlier, this exposes a `test` step to
    // the `zig build --help` menu, providing a way for the user to request
    // running the unit tests.
    const test_step = b.step("test", "Run unit tests");
    test_step.dependOn(&run_exe_unit_tests.step);
}

pub const Options = struct {
    gpu_dawn_options: gpu_dawn.Options = .{},
};

pub fn link(b: *std.Build, step: *std.Build.Step.Compile, mod: *std.Build.Module, options: Options) !void {
    if (step.rootModuleTarget().cpu.arch != .wasm32) {
        gpu_dawn.link(
            b.dependency("mach_gpu_dawn", .{
                .target = step.root_module.resolved_target.?,
                .optimize = step.root_module.optimize.?,
            }).builder,
            step,
            mod,
            options.gpu_dawn_options,
        );
        step.addCSourceFile(.{ .file = b.path("../krajc-wgpu/src/mach_dawn.cpp"), .flags = &.{"-std=c++17"} });
        step.addIncludePath(b.path("../krajc-wgpu/src"));
    }
}

fn sdkPath(comptime suffix: []const u8) []const u8 {
    if (suffix[0] != '/') @compileError("suffix must be an absolute path");
    return comptime blk: {
        const root_dir = std.fs.path.dirname(@src().file) orelse ".";
        break :blk root_dir ++ suffix;
    };
}
