const nri_c = @import("nri.zig");
const api = @import("root.zig").api;

pub const Result = error{
    DeviceLost,
    OutOfDate,
    InvalidSdk,
    DriverFailure,
    InvalidArgument,
    OutOfMemory,
    MaximumNumber,
    UnsupportedFeature,
};

pub fn checkResult(result: api.Result) Result!void {
    return switch (result) {
        api.Result.SUCCESS => {},
        api.Result.DEVICE_LOST => Result.DeviceLost,
        api.Result.OUT_OF_DATE => Result.OutOfDate,
        api.Result.INVALID_SDK => Result.InvalidSdk,
        api.Result.FAILURE => Result.DriverFailure,
        api.Result.INVALID_ARGUMENT => Result.InvalidArgument,
        api.Result.OUT_OF_MEMORY => Result.OutOfMemory,
        api.Result.MAX_NUM => Result.MaximumNumber,
        api.Result.UNSUPPORTED => Result.UnsupportedFeature,
    };
}
pub fn checkResultC(result: i8) Result!void {
    return switch (@as(api.Result, @enumFromInt(result))) {
        api.Result.SUCCESS => {},
        api.Result.DEVICE_LOST => Result.DeviceLost,
        api.Result.OUT_OF_DATE => Result.OutOfDate,
        api.Result.INVALID_SDK => Result.InvalidSdk,
        api.Result.FAILURE => Result.DriverFailure,
        api.Result.INVALID_ARGUMENT => Result.InvalidArgument,
        api.Result.OUT_OF_MEMORY => Result.OutOfMemory,
        api.Result.MAX_NUM => Result.MaximumNumber,
        api.Result.UNSUPPORTED => Result.UnsupportedFeature,
    };
}
