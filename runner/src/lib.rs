#[cfg(any(target_os = "ios", target_os = "macos"))]
mod apple;
mod buffer;
#[cfg(any(target_os = "android", target_os = "linux", target_os = "windows"))]
mod litert;
mod runtime;

use std::ffi::c_void;

use buffer::{read_model_request, read_token_ids, write_emoji_scores};
use runtime::{ModelHandle, RunError, open_model};

/// Four token ids, the same width as `model/train.py`.
pub(crate) const TOKEN_SLOTS: usize = 4;
/// One score per emoji in `model/labels.txt`.
pub(crate) const EMOJI_COUNT: usize = 100;

pub(crate) type EmojiScores = [f32; EMOJI_COUNT];

const STATUS_OK: i32 = 0;
const STATUS_NULL_POINTER: i32 = 1;
const STATUS_MALFORMED_WINDOW: i32 = 2;
const STATUS_OUTPUT_TOO_SMALL: i32 = 3;
const STATUS_MODEL_FILE_MISSING: i32 = 4;
const STATUS_RUNTIME_NOT_CONNECTED: i32 = 5;
const STATUS_RUNTIME_FAILED: i32 = 6;

/// Compile one model.
///
/// `input` is little-endian: a `u32` path length, the UTF-8 path to the model
/// file, then a `u32` accelerator (`0` CPU, `1` GPU).
/// Returns the handle, or null when compilation fails. `status_ptr` receives
/// the status code when it is not null.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_open(
    input_ptr: *const u8,
    input_len: usize,
    status_ptr: *mut i32,
) -> *mut c_void {
    match open_from_buffer(input_ptr, input_len) {
        Ok(handle) => {
            write_status(status_ptr, STATUS_OK);
            Box::into_raw(Box::new(handle)) as *mut c_void
        }
        Err(error) => {
            write_status(status_ptr, status_for_error(error));
            std::ptr::null_mut()
        }
    }
}

/// Score one window on a handle from `meowmoji_open`.
///
/// `input` is four little-endian `i32` token ids. `output` receives one hundred
/// little-endian `f32` scores, in label order. Returns `0` when the scores are
/// written.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_run(
    handle: *mut c_void,
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
) -> i32 {
    match run_buffers(handle, input_ptr, input_len, output_ptr, output_len) {
        Ok(()) => STATUS_OK,
        Err(error) => status_for_error(error),
    }
}

/// Free a handle from `meowmoji_open`. Null does nothing. The pointer must not
/// be used again, and must not be closed twice.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_close(handle: *mut c_void) {
    if handle.is_null() {
        return;
    }
    drop(unsafe { Box::from_raw(handle as *mut ModelHandle) });
}

fn open_from_buffer(input_ptr: *const u8, input_len: usize) -> Result<ModelHandle, RunError> {
    let input_bytes = bytes_from_pointer(input_ptr, input_len)?;
    let request = read_model_request(input_bytes)?;
    open_model(&request.model_path, request.accelerator)
}

fn run_buffers(
    handle: *mut c_void,
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
) -> Result<(), RunError> {
    if handle.is_null() {
        return Err(RunError::NullPointer);
    }
    let input_bytes = bytes_from_pointer(input_ptr, input_len)?;
    let output_bytes = writable_bytes_from_pointer(output_ptr, output_len)?;
    let token_ids = read_token_ids(input_bytes)?;
    let model = unsafe { &*(handle as *const ModelHandle) };
    let emoji_scores = model.score(&token_ids)?;
    write_emoji_scores(&emoji_scores, output_bytes)
}

fn bytes_from_pointer<'a>(pointer: *const u8, byte_count: usize) -> Result<&'a [u8], RunError> {
    if pointer.is_null() {
        return Err(RunError::NullPointer);
    }
    byte_count_fits_slice(byte_count)?;
    Ok(unsafe { std::slice::from_raw_parts(pointer, byte_count) })
}

fn writable_bytes_from_pointer<'a>(
    pointer: *mut u8,
    byte_count: usize,
) -> Result<&'a mut [u8], RunError> {
    if pointer.is_null() {
        return Err(RunError::NullPointer);
    }
    byte_count_fits_slice(byte_count)?;
    Ok(unsafe { std::slice::from_raw_parts_mut(pointer, byte_count) })
}

/// Reject a byte count larger than `isize::MAX`.
fn byte_count_fits_slice(byte_count: usize) -> Result<(), RunError> {
    if byte_count > isize::MAX as usize {
        Err(RunError::MalformedWindow)
    } else {
        Ok(())
    }
}

fn write_status(status_ptr: *mut i32, status: i32) {
    if !status_ptr.is_null() {
        unsafe { *status_ptr = status }
    }
}

fn status_for_error(error: RunError) -> i32 {
    match error {
        RunError::NullPointer => STATUS_NULL_POINTER,
        RunError::MalformedWindow => STATUS_MALFORMED_WINDOW,
        RunError::OutputTooSmall => STATUS_OUTPUT_TOO_SMALL,
        RunError::ModelFileMissing => STATUS_MODEL_FILE_MISSING,
        RunError::RuntimeNotConnected => STATUS_RUNTIME_NOT_CONNECTED,
        RunError::RuntimeFailed => STATUS_RUNTIME_FAILED,
    }
}
