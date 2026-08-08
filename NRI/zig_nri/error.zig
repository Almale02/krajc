const nri_c = @import("nri.zig");

pub const NriError = error{
    DeviceLost,
    OutOfDate,
    InvalidSdk,
    DriverFailure,
    InvalidArgument,
    OutOfMemory,
    UnsupportedFeature,
    MaximumNumber,
    UnknownError,
};

pub fn checkResult(result: anytype) NriError!void {
    const res_val: c_int = @intCast(result);

    return switch (res_val) {
        nri_c.Result_SUCCESS => {},

        nri_c.Result_DEVICE_LOST => NriError.DeviceLost,
        nri_c.Result_OUT_OF_DATE => NriError.OutOfDate,
        nri_c.Result_INVALID_SDK => NriError.InvalidSdk,
        nri_c.Result_FAILURE => NriError.DriverFailure,
        nri_c.Result_INVALID_ARGUMENT => NriError.InvalidArgument,
        nri_c.Result_OUT_OF_MEMORY => NriError.OutOfMemory,
        nri_c.Result_MAX_NUM => NriError.MaximumNumber,
        nri_c.Result_UNSUPPORTED => NriError.UnsupportedFeature,

        else => error.UnknownError,
    };
}
