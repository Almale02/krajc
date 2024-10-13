const std = @import("std");
pub const zm = @import("zm");

pub const Vec2 = struct {
    vec: zm.Vec2f = zm.vec.zero(2, f32),

    const Self = @This();

    pub fn new(_x: f32, _y: f32) Self {
        return Self{ .vec = zm.Vec2f{ _x, _y } };
        //zm.vec.
    }
    pub fn new_f64(_x: f64, _y: f64) Self {
        return Self{ .vec = zm.Vec2f{ @floatCast(_x), @floatCast(_y) } };
    }
    pub fn from_vec(vec: zm.Vec2f) Self {
        return Self{ .vec = vec };
    }
    pub fn x(self: Self) f32 {
        return self.vec[0];
    }
    pub fn y(self: Self) f32 {
        return self.vec[1];
    }

    pub fn dot(self: Self, other: Self) f32 {
        return zm.vec.dot(self.vec, other.vec);
    }

    pub fn lenSq(self: Self) f32 {
        return .vec.lenSq(self.vec);
    }

    pub fn len(self: Self) f32 {
        return zm.vec.len(self.vec);
    }

    pub fn normalize(self: Self) Self {
        return from_vec(zm.vec.normalize(self.vec));
    }
    pub fn normaize_mut(self: *Self) void {
        self.* = self.normalize();
    }

    pub fn cross(self: Self, other: Self) Self {
        return from_vec(zm.vec.cross(self.vec, other.vec));
    }

    /// Returns the distance between two points.
    pub fn distance(self: Self, other: Self) f32 {
        return zm.vec.distance(self.vec, other.vec);
    }

    /// Returns the angle between two vectors in radians.
    pub fn angle(self: Self, other: Self) f32 {
        return zm.vec.angle(self.vec, other.vec);
    }

    pub fn lerp(a: Self, b: Self, t: f32) Self {
        return from_vec(zm.vec.lerp(a.vec, b.vec, t));
    }

    /// Reflects `self` along `normal`. `normal` must be normalized.
    pub fn reflect(self: Self, normal: Self) Self {
        return from_vec(zm.vec.reflect(self.vec, normal.vec));
    }
};
pub const Vec3 = struct {
    vec: zm.Vec3f = zm.vec.zero(3, f32),

    const Self = @This();

    pub fn new(_x: f32, _y: f32, _z: f32) Self {
        return Self{ .vec = zm.Vec3f{ _x, _y, _z } };
    }
    pub fn new_f64(_x: f32, _y: f32, _z: f32) Self {
        return Self{ .vec = zm.Vec3f{ @floatCast(_x), @floatCast(_y), @floatCast(_z) } };
    }
    pub fn from_vec(vec: zm.Vec3f) Self {
        return Self{ .vec = vec };
    }
    pub fn x(self: Self) f32 {
        return self.vec[0];
    }
    pub fn y(self: Self) f32 {
        return self.vec[1];
    }
    pub fn z(self: Self) f32 {
        return self.vec[2];
    }

    pub fn dot(self: Self, other: Self) f32 {
        return zm.vec.dot(self.vec, other.vec);
    }

    pub fn lenSq(self: Self) f32 {
        return zm.vec.lenSq(self.vec);
    }

    pub fn len(self: Self) f32 {
        return zm.vec.len(self.vec);
    }

    pub fn normalize(self: Self) Self {
        return from_vec(zm.vec.normalize(self.vec));
    }
    pub fn normaize_mut(self: *Self) void {
        self.* = self.normalize();
    }

    pub fn cross(self: Self, other: Self) Self {
        return from_vec(zm.vec.cross(self.vec, other.vec));
    }

    /// Returns the distance between two points.
    pub fn distance(self: Self, other: Self) f32 {
        return zm.vec.distance(self.vec, other.vec);
    }

    /// Returns the angle between two vectors in radians.
    pub fn angle(self: Self, other: Self) f32 {
        return zm.vec.angle(self.vec, other.vec);
    }

    pub fn lerp(a: Self, b: Self, t: f32) Self {
        return from_vec(zm.vec.lerp(a.vec, b.vec, t));
    }

    /// Reflects `self` along `normal`. `normal` must be normalized.
    pub fn reflect(self: Self, normal: Self) Self {
        return from_vec(zm.vec.reflect(self.vec, normal.vec));
    }
};
