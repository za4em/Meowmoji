use crate::runtime::RunError;
use crate::{EMOJI_COUNT, EmojiScores, TOKEN_SLOTS};

use super::c_api::{
    FIRST_SIGNATURE, INPUT_BUFFER_COUNT, LITERT_OK, LOCK_FOR_READ, LOCK_FOR_WRITE,
    LiteRtCompiledModel, LiteRtGetTensorBufferSize, LiteRtLockTensorBuffer, LiteRtRunCompiledModel,
    LiteRtTensorBuffer, LiteRtUnlockTensorBuffer, OUTPUT_BUFFER_COUNT,
};

pub(super) fn write_token_ids(
    token_buffer: LiteRtTensorBuffer,
    token_ids: &[i32; TOKEN_SLOTS],
) -> Result<(), RunError> {
    let bytes = std::mem::size_of_val(token_ids);
    with_locked_buffer(token_buffer, LOCK_FOR_WRITE, bytes, |host| unsafe {
        std::ptr::copy_nonoverlapping(token_ids.as_ptr() as *const u8, host, bytes);
    })
}

pub(super) fn read_emoji_scores(score_buffer: LiteRtTensorBuffer) -> Result<EmojiScores, RunError> {
    let mut emoji_scores = [0.0; EMOJI_COUNT];
    let bytes = std::mem::size_of_val(&emoji_scores);
    with_locked_buffer(score_buffer, LOCK_FOR_READ, bytes, |host| unsafe {
        std::ptr::copy_nonoverlapping(
            host as *const u8,
            emoji_scores.as_mut_ptr() as *mut u8,
            bytes,
        );
    })?;
    Ok(emoji_scores)
}

fn with_locked_buffer(
    buffer: LiteRtTensorBuffer,
    lock_mode: i32,
    byte_count: usize,
    copy: impl FnOnce(*mut u8),
) -> Result<(), RunError> {
    let mut host = std::ptr::null_mut();
    let locked = unsafe { LiteRtLockTensorBuffer(buffer, &mut host, lock_mode) };
    if locked != LITERT_OK || host.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    let mut buffer_size = 0;
    let sized = unsafe { LiteRtGetTensorBufferSize(buffer, &mut buffer_size) };
    if sized != LITERT_OK || buffer_size < byte_count {
        unsafe { LiteRtUnlockTensorBuffer(buffer) };
        return Err(RunError::RuntimeFailed);
    }
    copy(host as *mut u8);
    let unlocked = unsafe { LiteRtUnlockTensorBuffer(buffer) };
    if unlocked != LITERT_OK {
        return Err(RunError::RuntimeFailed);
    }
    Ok(())
}

pub(super) fn run_compiled_model(
    compiled_model: LiteRtCompiledModel,
    token_buffer: &mut LiteRtTensorBuffer,
    score_buffer: &mut LiteRtTensorBuffer,
) -> Result<(), RunError> {
    let status = unsafe {
        LiteRtRunCompiledModel(
            compiled_model,
            FIRST_SIGNATURE,
            INPUT_BUFFER_COUNT,
            token_buffer,
            OUTPUT_BUFFER_COUNT,
            score_buffer,
        )
    };
    if status != LITERT_OK {
        return Err(RunError::RuntimeFailed);
    }
    Ok(())
}
