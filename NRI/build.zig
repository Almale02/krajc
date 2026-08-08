const std = @import("std");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const run_deploy = b.addSystemCommand(&.{"./1-Deploy.sh"});
    const run_build = b.addSystemCommand(&.{"./2-Build.sh"});
    const run_prepare = b.addSystemCommand(&.{"./3-PrepareSDK.sh"});

    run_build.step.dependOn(&run_deploy.step);
    run_prepare.step.dependOn(&run_build.step);

    run_deploy.setCwd(b.path("."));
    run_build.setCwd(b.path("."));
    run_prepare.setCwd(b.path("."));

    const nri_step = b.step("nri_build", "Build NVIDIA NRI via CMake");
    nri_step.dependOn(&run_prepare.step);
    const nri_modul = b.addModule("nri", .{
        .root_source_file = b.path("zig_nri/root.zig"),
        .link_libc = true,
        .link_libcpp = true,
        .target = target,
        .optimize = optimize,
    });
    nri_modul.addLibraryPath(b.path("_NRI_SDK/Lib/"));
    nri_modul.addObjectFile(b.path("_NRI_SDK/Lib/libNRI.so"));

    const exe_module = b.createModule(.{
        .root_source_file = b.path("zig_nri/tools/main.zig"),
        .target = target,
        .optimize = optimize,
    });
    const exe = b.addExecutable(.{ .name = "nri_tools", .root_module = exe_module });
    b.installArtifact(exe);

    const run_step = b.step("run", "run tools");
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const build_step = b.step("build_nri", "Builds nvidia nri");
    build_step.dependOn(&run_prepare.step);

    const lib_check = b.addTest(.{
        .root_module = nri_modul,
    });
    const lib_check_tools = b.addTest(.{
        .root_module = exe_module,
    });

    const check_step = b.step("check", "check code");
    check_step.dependOn(&lib_check.step);
    check_step.dependOn(&lib_check_tools.step);
    b.default_step.dependOn(&lib_check.step);
    b.default_step.dependOn(&lib_check_tools.step);
}
