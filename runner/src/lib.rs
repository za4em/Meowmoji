#[cfg(target_os = "ios")]
mod apple;
mod buffer;
#[cfg(not(target_os = "ios"))]
mod litert;
mod runtime;

use buffer::{read_window, write_emoji_scores};
use runtime::{RunError, run_window};

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

/// Run one window.
///
/// `input` is little-endian: a `u32` path length, the UTF-8 path to the model
/// file, a `u32` accelerator (`0` CPU, `1` GPU), then four `i32` token ids.
/// `output` receives one hundred little-endian `f32` scores, in label order.
/// Returns `0` when the scores are written.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_run(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
) -> i32 {
    match run_buffers(input_ptr, input_len, output_ptr, output_len) {
        Ok(()) => STATUS_OK,
        Err(error) => status_for_error(error),
    }
}

fn run_buffers(
    input_ptr: *const u8,
    input_len: usize,
    output_ptr: *mut u8,
    output_len: usize,
) -> Result<(), RunError> {
    let input_bytes = bytes_from_pointer(input_ptr, input_len)?;
    let output_bytes = writable_bytes_from_pointer(output_ptr, output_len)?;
    let window = read_window(input_bytes)?;
    let emoji_scores = run_window(&window)?;
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

fn status_for_error(error: RunError) -> i32 {
    match error {
        RunError::NullPointer => STATUS_NULL_POINTER,
        RunError::MalformedWindow => STATUS_MALFORMED_WINDOW,
        RunError::OutputTooSmall => STATUS_OUTPUT_TOO_SMALL,
        RunError::ModelFileMissing => STATUS_MODEL_FILE_MISSING,
        RunError::RuntimeNotConnected => STATUS_RUNTIME_NOT_CONNECTED,
    }
}
