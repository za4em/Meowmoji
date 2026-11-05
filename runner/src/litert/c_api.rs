use std::ffi::{c_char, c_int, c_void};

pub(super) const LITERT_OK: i32 = 0;
pub(super) const LITERT_CPU: i32 = 1;
pub(super) const LITERT_GPU: i32 = 2;
pub(super) const LOCK_FOR_READ: i32 = 0;
pub(super) const LOCK_FOR_WRITE: i32 = 1;
pub(super) const FIRST_SIGNATURE: usize = 0;
pub(super) const FIRST_TENSOR: usize = 0;
pub(super) const INPUT_BUFFER_COUNT: usize = 1;
pub(super) const OUTPUT_BUFFER_COUNT: usize = 1;

pub(super) type LiteRtEnvironment = *mut c_void;
pub(super) type LiteRtModel = *mut c_void;
pub(super) type LiteRtOptions = *mut c_void;
pub(super) type LiteRtCompiledModel = *mut c_void;
pub(super) type LiteRtTensorBuffer = *mut c_void;
pub(super) type LiteRtTensorBufferRequirements = *mut c_void;
pub(super) type LiteRtSignature = *mut c_void;
pub(super) type LiteRtTensor = *mut c_void;

#[repr(C)]
pub(super) struct TensorLayout {
    rank_bits: u32,
    dimensions: [i32; 8],
    strides: [u32; 8],
}

#[repr(C)]
pub(super) struct RankedTensorType {
    element_type: i32,
    layout: TensorLayout,
}

const _: () = assert!(std::mem::size_of::<RankedTensorType>() == 72);

pub(super) fn destroy_model(model: LiteRtModel) {
    unsafe { LiteRtDestroyModel(model) }
}

pub(super) fn destroy_options(options: LiteRtOptions) {
    unsafe { LiteRtDestroyOptions(options) }
}

pub(super) fn destroy_compiled_model(compiled_model: LiteRtCompiledModel) {
    unsafe { LiteRtDestroyCompiledModel(compiled_model) }
}

pub(super) fn destroy_tensor_buffer(buffer: LiteRtTensorBuffer) {
    unsafe { LiteRtDestroyTensorBuffer(buffer) }
}

unsafe extern "C" {
    pub(super) fn LiteRtCreateEnvironment(
        num_options: c_int,
        options: *const c_void,
        environment: *mut LiteRtEnvironment,
    ) -> i32;
    pub(super) fn LiteRtCreateModelFromFile(
        environment: LiteRtEnvironment,
        filename: *const c_char,
        model: *mut LiteRtModel,
    ) -> i32;
    fn LiteRtDestroyModel(model: LiteRtModel);
    pub(super) fn LiteRtCreateOptions(options: *mut LiteRtOptions) -> i32;
    fn LiteRtDestroyOptions(options: LiteRtOptions);
    pub(super) fn LiteRtSetOptionsHardwareAccelerators(
        options: LiteRtOptions,
        accelerators: i32,
    ) -> i32;
    pub(super) fn LiteRtCreateCompiledModel(
        environment: LiteRtEnvironment,
        model: LiteRtModel,
        options: LiteRtOptions,
        compiled_model: *mut LiteRtCompiledModel,
    ) -> i32;
    fn LiteRtDestroyCompiledModel(compiled_model: LiteRtCompiledModel);
    pub(super) fn LiteRtGetModelSignature(
        model: LiteRtModel,
        signature_index: usize,
        signature: *mut LiteRtSignature,
    ) -> i32;
    pub(super) fn LiteRtGetSignatureInputTensorByIndex(
        signature: LiteRtSignature,
        tensor_index: usize,
        tensor: *mut LiteRtTensor,
    ) -> i32;
    pub(super) fn LiteRtGetSignatureOutputTensorByIndex(
        signature: LiteRtSignature,
        tensor_index: usize,
        tensor: *mut LiteRtTensor,
    ) -> i32;
    pub(super) fn LiteRtGetRankedTensorType(
        tensor: LiteRtTensor,
        tensor_type: *mut RankedTensorType,
    ) -> i32;
    pub(super) fn LiteRtGetCompiledModelInputBufferRequirements(
        compiled_model: LiteRtCompiledModel,
        signature_index: usize,
        tensor_index: usize,
        requirements: *mut LiteRtTensorBufferRequirements,
    ) -> i32;
    pub(super) fn LiteRtGetCompiledModelOutputBufferRequirements(
        compiled_model: LiteRtCompiledModel,
        signature_index: usize,
        tensor_index: usize,
        requirements: *mut LiteRtTensorBufferRequirements,
    ) -> i32;
    pub(super) fn LiteRtCreateManagedTensorBufferFromRequirements(
        environment: LiteRtEnvironment,
        tensor_type: *const RankedTensorType,
        requirements: LiteRtTensorBufferRequirements,
        buffer: *mut LiteRtTensorBuffer,
    ) -> i32;
    fn LiteRtDestroyTensorBuffer(buffer: LiteRtTensorBuffer);
    pub(super) fn LiteRtGetTensorBufferSize(buffer: LiteRtTensorBuffer, size: *mut usize) -> i32;
    pub(super) fn LiteRtLockTensorBuffer(
        buffer: LiteRtTensorBuffer,
        host_memory: *mut *mut c_void,
        lock_mode: i32,
    ) -> i32;
    pub(super) fn LiteRtUnlockTensorBuffer(buffer: LiteRtTensorBuffer) -> i32;
    pub(super) fn LiteRtRunCompiledModel(
        compiled_model: LiteRtCompiledModel,
        signature_index: usize,
        input_count: usize,
        input_buffers: *mut LiteRtTensorBuffer,
        output_count: usize,
        output_buffers: *mut LiteRtTensorBuffer,
    ) -> i32;
}
