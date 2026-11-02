#[cfg(any(target_os = "ios", target_os = "macos"))]
mod apple;
#[cfg(any(target_os = "android", target_os = "linux", target_os = "windows"))]
mod litert;
mod runtime;

use std::ffi::{CStr, c_char, c_void};

use runtime::{Accelerator, ModelHandle, RunError, open_model};

/// Four token ids, the same width as `model/train.py`.
pub(crate) const TOKEN_SLOTS: usize = 4;
/// One score per emoji in `model/labels.txt`.
pub(crate) const EMOJI_COUNT: usize = 100;

pub(crate) type EmojiScores = [f32; EMOJI_COUNT];

pub(crate) const STATUS_OK: i32 = 0;
pub(crate) const STATUS_NULL_POINTER: i32 = 1;
const STATUS_MALFORMED_WINDOW: i32 = 2;
const STATUS_OUTPUT_TOO_SMALL: i32 = 3;
const STATUS_MODEL_FILE_MISSING: i32 = 4;
const STATUS_RUNTIME_NOT_CONNECTED: i32 = 5;
pub(crate) const STATUS_RUNTIME_FAILED: i32 = 6;

#[cfg(target_os = "android")]
mod android_jni;

/// Compile one model.
///
/// `path` is a null-terminated UTF-8 path to the model file. `accelerator` is
/// `0` for CPU and `1` for GPU. Returns the handle, or null when compilation
/// fails. `status_ptr` receives the status code when it is not null.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_open(
    path: *const c_char,
    accelerator: u32,
    status_ptr: *mut i32,
) -> *mut c_void {
    match open_from_path(path, accelerator) {
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
/// `token_ids` holds `token_count` ids. This model scores exactly four.
/// `scores` receives one hundred `f32` values, in label order. Returns `0`
/// when the scores are written.
#[unsafe(no_mangle)]
pub extern "C" fn meowmoji_run(
    handle: *mut c_void,
    token_ids: *const i32,
    token_count: usize,
    scores: *mut f32,
    score_count: usize,
) -> i32 {
    match score_tokens(handle, token_ids, token_count, scores, score_count) {
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

fn open_from_path(path: *const c_char, accelerator: u32) -> Result<ModelHandle, RunError> {
    if path.is_null() {
        return Err(RunError::NullPointer);
    }
    let path = unsafe { CStr::from_ptr(path) }
        .to_str()
        .map_err(|_| RunError::MalformedWindow)?;
    if path.is_empty() {
        return Err(RunError::MalformedWindow);
    }
    open_model(path, Accelerator::from_u32(accelerator)?)
}

fn score_tokens(
    handle: *mut c_void,
    token_ids: *const i32,
    token_count: usize,
    scores: *mut f32,
    score_count: usize,
) -> Result<(), RunError> {
    if handle.is_null() || token_ids.is_null() || scores.is_null() {
        return Err(RunError::NullPointer);
    }
    if token_count != TOKEN_SLOTS || token_count > isize::MAX as usize {
        return Err(RunError::MalformedWindow);
    }
    if score_count < EMOJI_COUNT || score_count > isize::MAX as usize {
        return Err(RunError::OutputTooSmall);
    }
    let token_ids = unsafe { std::slice::from_raw_parts(token_ids, TOKEN_SLOTS) };
    let mut tokens = [0; TOKEN_SLOTS];
    tokens.copy_from_slice(token_ids);
    let model = unsafe { &*(handle as *const ModelHandle) };
    let emoji_scores = model.score(&tokens)?;
    let scores = unsafe { std::slice::from_raw_parts_mut(scores, EMOJI_COUNT) };
    scores.copy_from_slice(&emoji_scores);
    Ok(())
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
