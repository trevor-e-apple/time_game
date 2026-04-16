const builtin = @import("builtin");
const sdl3 = @import("sdl3");
const std = @import("std");

const ArenaAllocator = std.heap.ArenaAllocator;

const fps = 60;
const screen_width = 640;
const screen_height = 480;

pub fn main() !void {
    const parent_allocator: std.mem.Allocator = allocator: switch (builtin.mode) {
        .Debug => {
            var debug_allocator = std.heap.DebugAllocator(.{}).init;
            break :allocator debug_allocator.allocator();
        },
        else => break :allocator std.heap.page_allocator,
    };

    var arena = ArenaAllocator.init(parent_allocator);
    defer arena.deinit();

    defer sdl3.shutdown();

    // Initialize SDL with needed subsystems
    const init_flags = sdl3.InitFlags{ .video = true };
    try sdl3.init(init_flags);
    defer sdl3.quit(init_flags);

    // Initial window setup
    const window = try sdl3.video.Window.init("Time", screen_width, screen_height, sdl3.video.Window.Flags{});
    defer window.deinit();

    // Renderer setup
    const gpu_device = try sdl3.gpu.Device.init(sdl3.gpu.ShaderFormatFlags{ .spirv = true, .msl = true }, builtin.mode == .Debug, null);
    try gpu_device.claimWindow(window);

    const vertex_shader = vertex_shader_setup: {
        // Read shader source
        const env_map = try std.process.getEnvMap(arena.allocator());
        const path_to_shader_dir = env_map.get("SHADER_DIR");

        const result = try gpu_device.createShader(sdl3.gpu.ShaderCreateInfo{
            .code,
            .entry_point,
            .format,
            .stage,
            .num_samplers,
        });

        break :vertex_shader_setup result;
    };

    // Load BMP

    var fps_capper = sdl3.extras.FramerateCapper(f32){ .mode = .{ .limited = fps } };
    var quit = false;

    while (!quit) {
        // Delay to limit he FPS, returned delta time not needed
        _ = fps_capper.delay();

        const surface = try window.getSurface();
        try surface.fillRect(null, surface.mapRgb(128, 30, 255));
        try window.updateSurface();

        while (sdl3.events.poll()) |event| {
            switch (event) {
                .quit => quit = true,
                .terminating => quit = true,
                else => {},
            }
        }

        _ = arena.reset(.retain_capacity);
    }
}
