const std = @import("std");

// Although this function looks imperative, it does not perform the build
// directly and instead it mutates the build graph (`b`) that will be then
// executed by an external runner. The functions in `std.Build` implement a DSL
// for defining build steps and express dependencies between them, allowing the
// build runner to parallelize the build automatically (and the cache system to
// know when a step doesn't need to be re-run).
pub fn build(b: *std.Build) void {
    // Standard target options allow the person running `zig build` to choose
    // what target to build for. Here we do not override the defaults, which
    // means any target is allowed, and the default is native. Other options
    // for restricting supported target set are available.
    const target = b.standardTargetOptions(.{});
    // Standard optimization options allow the person running `zig build` to select
    // between Debug, ReleaseSafe, ReleaseFast, and ReleaseSmall. Here we do not
    // set a preferred release mode, allowing the user to decide how to optimize.
    const optimize = b.standardOptimizeOption(.{});
    const sdl3 = b.dependency("sdl3", .{
        .target = target,
        .optimize = optimize,
        .main = false,
        .ext_image = false,
        .ext_mixer = false,
        .ext_net = false,
        .ext_ttf = false,
        .ext_shadercross = false,
        .ext_shadercross_dxc = false,
    });
    const nri = b.dependency("nri", .{ .target = target, .optimize = optimize });

    const mod = b.addModule("krajc", .{
        .root_source_file = b.path("src/engine/root.zig"),
        // Later on we'll use this module as the root module of a test executable
        // which requires us to specify a target.
        .target = target,
        .optimize = optimize,
    });
    mod.addImport("sdl", sdl3.module("sdl3"));
    mod.addImport("nri", nri.module("nri"));

    const exe_module = b.createModule(.{
        // b.createModule defines a new module just like b.addModule but,
        // unlike b.addModule, it does not expose the module to consumers of
        // this package, which is why in this case we don't have to give it a name.
        .root_source_file = b.path("src/runtime/main.zig"),
        // Target and optimization levels must be explicitly wired in when
        // defining an executable or library (in the root module), and you
        // can also hardcode a specific target for an executable or library
        // definition if desireable (e.g. firmware for embedded devices).
        .target = target,
        .optimize = optimize,
        .link_libc = true,
        .link_libcpp = true,
        // List of modules available for import in source files part of the
        // root module.
        .imports = &.{
            // Provide all modules that the `krajc` module depends on so
            // they are available to the executable's root module at
            // compile time. Omitting transitive modules can cause the
            // compiler to complain that a module depends on a
            // non-existent module (see `nri` below).
            .{ .name = "krajc", .module = mod },
            .{ .name = "sdl", .module = sdl3.module("sdl3") },
            .{ .name = "nri", .module = nri.module("nri") },
        },
    });
    const nri_so_path = nri.path("_NRI_SDK/Lib/libNRI.so");
    exe_module.addLibraryPath(nri_so_path.dirname());
    exe_module.linkSystemLibrary("NRI", .{ .needed = true });
    exe_module.addRPath(b.path("."));

    const exe = b.addExecutable(.{
        .name = "krajc_runtime",
        .root_module = exe_module,
    });
    exe.use_llvm = true;

    const nri_build = nri.builder.top_level_steps.get("nri_build").?;
    exe.step.dependOn(&nri_build.step);

    const copy_lib = b.addInstallFileWithDir(nri_so_path, .bin, "libNRI.so");
    b.getInstallStep().dependOn(&copy_lib.step);

    // This declares intent for the executable to be installed into the
    // install prefix when running `zig build` (i.e. when executing the default
    // step). By default the install prefix is `zig-out/` but can be overridden
    // by passing `--prefix` or `-p`.
    b.installArtifact(exe);

    // This creates a top level step. Top level steps have a name and can be
    // invoked by name when running `zig build` (e.g. `zig build run`).
    // This will evaluate the `run` step rather than the default step.
    // For a top level step to actually do something, it must depend on other
    // steps (e.g. a Run step, as we will see in a moment).
    const run_step = b.step("run", "Run the app");

    // This creates a RunArtifact step in the build graph. A RunArtifact step
    // invokes an executable compiled by Zig. Steps will only be executed by the
    // runner if invoked directly by the user (in the case of top level steps)
    // or if another step depends on it, so it's up to you to define when and
    // how this Run step will be executed. In our case we want to run it when
    // the user runs `zig build run`, so we create a dependency link.
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);

    // By making the run step depend on the default step, it will be run from the
    // installation directory rather than directly from within the cache directory.
    run_cmd.step.dependOn(b.getInstallStep());

    // This allows the user to pass arguments to the application in the build
    // command itself, like this: `zig build run -- arg1 arg2 etc`
    if (b.args) |args| {
        if (std.mem.eql(u8, args[0], "wayland_debug")) {
            run_cmd.setEnvironmentVariable("WAYLAND_DEBUG", "1");
        }
        run_cmd.addArgs(args);
    }

    const lib_check = b.addTest(.{
        .root_module = mod,
    });
    const lib_check_exe = b.addTest(.{
        .root_module = exe_module,
    });

    const check_step = b.step("check", "check code");
    check_step.dependOn(&lib_check.step);
    check_step.dependOn(&lib_check_exe.step);
    b.default_step.dependOn(&lib_check.step);
    b.default_step.dependOn(&lib_check_exe.step);
}
