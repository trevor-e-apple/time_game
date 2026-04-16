const std = @import("std");
const ArenaAllocator = std.heap.ArenaAllocator;

pub fn build(b: *std.Build) !void {
    var arena = ArenaAllocator.init(std.heap.page_allocator);
    defer arena.deinit();
    defer {
        _ = arena.reset(.retain_capacity);
    }

    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});

    const sdl3 = b.dependency(
        "sdl3",
        .{
            .target = target,
            .optimize = optimize,
            // Lib options.
            .ext_image = false,
            .ext_net = false,
            .ext_ttf = false,
            .log_message_stack_size = 1024,
            .main = false,
            .renderer_debug_text_stack_size = 1024,
        },
    );

    var root_module = b.createModule(.{
        .root_source_file = b.path("main.zig"),
        .target = target,
    });
    root_module.addImport("sdl3", sdl3.module("sdl3"));

    const exe = b.addExecutable(.{ .name = "time", .root_module = root_module });
    b.installArtifact(exe);
}
