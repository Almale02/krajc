const std = @import("std");
const zchan = @import("lib.zig");

const BufferedChan = zchan.BufferedChan;
const Chan = zchan.Chan;
const ChannelError = zchan.ChanError;

pub fn main() !void {
    var c = BufferedChan(u8, 10){};
    std.debug.print("Capacity: {}\n", .{c.capacity()});
    std.debug.print("Len: {}\n", .{c.len()});
}

test "unbufferedChan" {
    // create channel of u8
    const T = Chan(u8);
    var chan = T.init(std.testing.allocator);
    defer chan.deinit();

    // spawn thread that immediately waits on channel
    const thread = struct {
        fn func(c: *T) !void {
            const val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
        }
    };
    const t = try std.Thread.spawn(.{}, thread.func, .{&chan});
    defer t.join();

    // let thread wait a bit before sending value
    std.time.sleep(1_000_000_000);

    const val: u8 = 10;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
}

test "bidirectional unbufferedChan" {
    std.debug.print("\n", .{});

    const T = Chan(u8);
    var chan = T.init(std.testing.allocator);
    defer chan.deinit();

    const thread = struct {
        fn func(c: *T) !void {
            std.time.sleep(2_000_000_000);
            const val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
            std.time.sleep(1_000_000_000);
            std.debug.print("{d} Thread Sending {d}\n", .{ std.time.milliTimestamp(), val + 1 });
            try c.send(val + 1);
            std.time.sleep(2_000_000_000);
            std.debug.print("{d} Thread Sending {d}\n", .{ std.time.milliTimestamp(), val + 100 });
            try c.send(val + 100);
            std.debug.print("{d} Thread Exit\n", .{std.time.milliTimestamp()});
        }
    };

    const t = try std.Thread.spawn(.{}, thread.func, .{&chan});
    defer t.join();

    std.time.sleep(1_000_000_000);
    var val: u8 = 10;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
    val = try chan.recv();
    std.debug.print("{d} Main Received {d}\n", .{ std.time.milliTimestamp(), val });
    val = try chan.recv();
    std.debug.print("{d} Main Received {d}\n", .{ std.time.milliTimestamp(), val });
}

test "buffered Chan" {
    std.debug.print("\n", .{});

    const T = BufferedChan(u8, 3);
    var chan = T.init(std.testing.allocator);
    defer chan.deinit();

    const thread = struct {
        fn func(c: *T) !void {
            std.time.sleep(2_000_000_000);
            std.debug.print("{d} Thread Receiving\n", .{std.time.milliTimestamp()});
            var val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
            std.time.sleep(1_000_000_000);
            std.debug.print("{d} Thread Receiving\n", .{std.time.milliTimestamp()});
            val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
            std.time.sleep(1_000_000_000);
            std.debug.print("{d} Thread Receiving\n", .{std.time.milliTimestamp()});
            val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
            std.time.sleep(1_000_000_000);
            std.debug.print("{d} Thread Receiving\n", .{std.time.milliTimestamp()});
            val = try c.recv();
            std.debug.print("{d} Thread Received {d}\n", .{ std.time.milliTimestamp(), val });
        }
    };

    const t = try std.Thread.spawn(.{}, thread.func, .{&chan});
    defer t.join();

    std.time.sleep(1_000_000_000);
    var val: u8 = 10;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
    std.debug.print("{d} Main Sent {d}\n", .{ std.time.milliTimestamp(), val });

    val = 11;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
    std.debug.print("{d} Main Sent {d}\n", .{ std.time.milliTimestamp(), val });

    val = 12;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
    std.debug.print("{d} Main Sent {d}\n", .{ std.time.milliTimestamp(), val });

    val = 13;
    std.debug.print("{d} Main Sending {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);
    std.debug.print("{d} Main Sent {d}\n", .{ std.time.milliTimestamp(), val });
}

test "chan of chan" {
    std.debug.print("\n", .{});

    const T = BufferedChan(u8, 3);
    const TofT = Chan(T);
    var chanOfChan = TofT.init(std.testing.allocator);
    defer chanOfChan.deinit();

    const thread = struct {
        fn func(cOC: *TofT) !void {
            std.time.sleep(2_000_000_000);
            std.debug.print("{d} Thread Receiving\n", .{std.time.milliTimestamp()});
            var c = try cOC.recv();
            std.debug.print("{d} Thread Received chan of chan: {any}\n", .{ std.time.milliTimestamp(), cOC });
            std.debug.print("{d} Thread pulling from chan buffer\n", .{std.time.milliTimestamp()});
            const val = try c.recv(); // should have value on buffer
            std.debug.print("{d} Thread received from chan: {d}\n", .{ std.time.milliTimestamp(), val });
        }
    };

    const t = try std.Thread.spawn(.{}, thread.func, .{&chanOfChan});
    defer t.join();

    std.time.sleep(1_000_000_000);
    const val: u8 = 10;
    var chan = T.init(std.testing.allocator);
    defer chan.deinit();
    std.debug.print("{d} Main sending u8 to chan {d}\n", .{ std.time.milliTimestamp(), val });
    try chan.send(val);

    std.debug.print("{d} Main sending chan across chanOfChan\n", .{std.time.milliTimestamp()});
    try chanOfChan.send(chan);
}
