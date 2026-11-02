//! JNI names for `com.meowmoji.sdk.Meowmoji`. The `jni` crate borrows the string
//! and the arrays, and releases them when the guards drop. `with_env` catches a
//! panic so it cannot unwind into the JVM.

use std::ffi::{CString, c_void};

use jni::errors::Error;
use jni::objects::{JFloatArray, JIntArray, JObject, JString, ReleaseMode};
use jni::sys::{jint, jlong};
use jni::{Env, EnvUnowned, Outcome};

use crate::{
    STATUS_NULL_POINTER, STATUS_RUNTIME_FAILED, meowmoji_close, meowmoji_open, meowmoji_run,
};

/// Compile one model. `path` is the model file. `accelerator` is `0` for CPU
/// and `1` for GPU. Returns the handle, or `0` when compilation fails.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeOpen<'local>(
    mut env: EnvUnowned<'local>,
    _thiz: JObject<'local>,
    path: JString<'local>,
    accelerator: jint,
) -> jlong {
    let outcome = env.with_env(|env| open_model(env, &path, accelerator));
    match outcome.into_outcome() {
        Outcome::Ok(handle) => handle,
        Outcome::Err(_) | Outcome::Panic(_) => 0,
    }
}

/// Score one window. `token_ids` holds four ids. `scores` receives one hundred
/// floats, in label order. Returns the `meowmoji_run` status code.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeRun<'local>(
    mut env: EnvUnowned<'local>,
    _thiz: JObject<'local>,
    handle: jlong,
    token_ids: JIntArray<'local>,
    scores: JFloatArray<'local>,
) -> jint {
    let outcome = env.with_env(|env| score_window(env, handle, &token_ids, &scores));
    match outcome.into_outcome() {
        Outcome::Ok(status) => status,
        Outcome::Err(Error::NullPtr(_)) => STATUS_NULL_POINTER,
        Outcome::Err(_) | Outcome::Panic(_) => STATUS_RUNTIME_FAILED,
    }
}

/// Free a handle from `nativeOpen`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeClose<'local>(
    _env: EnvUnowned<'local>,
    _thiz: JObject<'local>,
    handle: jlong,
) {
    meowmoji_close(handle as usize as *mut c_void);
}

fn open_model(env: &mut Env<'_>, path: &JString<'_>, accelerator: jint) -> Result<jlong, Error> {
    let chars = path.mutf8_chars(env)?;
    let Ok(path) = CString::new(chars.to_str().as_ref()) else {
        return Ok(0);
    };
    let mut status = 0;
    let handle = meowmoji_open(path.as_ptr(), accelerator as u32, &mut status);
    if handle.is_null() {
        Ok(0)
    } else {
        Ok(handle as usize as jlong)
    }
}

fn score_window(
    env: &mut Env<'_>,
    handle: jlong,
    token_ids: &JIntArray<'_>,
    scores: &JFloatArray<'_>,
) -> Result<jint, Error> {
    let tokens = unsafe { token_ids.get_elements(env, ReleaseMode::NoCopyBack)? };
    let scores = unsafe { scores.get_elements(env, ReleaseMode::CopyBack)? };
    Ok(meowmoji_run(
        handle as usize as *mut c_void,
        tokens.as_ptr(),
        tokens.len(),
        scores.as_ptr(),
        scores.len(),
    ))
}
