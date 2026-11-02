//! JNI names for `com.meowmoji.sdk.Meowmoji`. Each one borrows the `ByteArray`
//! bytes, calls the plain C function, and releases the array.

use std::ffi::c_void;

use jni_sys::{JNI_ABORT, JNIEnv, jbyte, jbyteArray, jint, jlong, jobject};

use crate::{
    STATUS_NULL_POINTER, STATUS_RUNTIME_FAILED, meowmoji_close, meowmoji_open, meowmoji_run,
};

/// Compile one model. Returns the handle, or `0` when `meowmoji_open` fails.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeOpen(
    env: *mut JNIEnv,
    _thiz: jobject,
    input: jbyteArray,
) -> jlong {
    let Ok(input_bytes) = locked_bytes(env, input, JNI_ABORT) else {
        return 0;
    };
    let mut status = 0;
    let handle = meowmoji_open(input_bytes.pointer(), input_bytes.length, &mut status);
    if handle.is_null() {
        0
    } else {
        handle as usize as jlong
    }
}

/// Score one window. Returns the `meowmoji_run` status code.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeRun(
    env: *mut JNIEnv,
    _thiz: jobject,
    handle: jlong,
    input: jbyteArray,
    output: jbyteArray,
) -> jint {
    let Ok(input_bytes) = locked_bytes(env, input, JNI_ABORT) else {
        return STATUS_NULL_POINTER;
    };
    let Ok(mut output_bytes) = locked_bytes(env, output, 0) else {
        return STATUS_NULL_POINTER;
    };
    meowmoji_run(
        handle as usize as *mut c_void,
        input_bytes.pointer(),
        input_bytes.length,
        output_bytes.pointer_mut(),
        output_bytes.length,
    )
}

/// Free a handle from `nativeOpen`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_meowmoji_sdk_Meowmoji_nativeClose(
    _env: *mut JNIEnv,
    _thiz: jobject,
    handle: jlong,
) {
    meowmoji_close(handle as usize as *mut c_void);
}

struct LockedBytes {
    env: *mut JNIEnv,
    array: jbyteArray,
    elements: *mut jbyte,
    length: usize,
    release_mode: jint,
}

fn locked_bytes(
    env: *mut JNIEnv,
    array: jbyteArray,
    release_mode: jint,
) -> Result<LockedBytes, i32> {
    if env.is_null() || array.is_null() {
        return Err(STATUS_NULL_POINTER);
    }
    unsafe {
        let table = &**env;
        let Some(array_length) = table.GetArrayLength else {
            return Err(STATUS_RUNTIME_FAILED);
        };
        let Some(get_elements) = table.GetByteArrayElements else {
            return Err(STATUS_RUNTIME_FAILED);
        };
        let length = array_length(env, array);
        if length < 0 {
            return Err(STATUS_NULL_POINTER);
        }
        let elements = get_elements(env, array, std::ptr::null_mut());
        if elements.is_null() {
            return Err(STATUS_NULL_POINTER);
        }
        Ok(LockedBytes {
            env,
            array,
            elements,
            length: length as usize,
            release_mode,
        })
    }
}

impl LockedBytes {
    fn pointer(&self) -> *const u8 {
        self.elements as *const u8
    }

    fn pointer_mut(&mut self) -> *mut u8 {
        self.elements as *mut u8
    }
}

impl Drop for LockedBytes {
    fn drop(&mut self) {
        unsafe {
            let table = &**self.env;
            if let Some(release) = table.ReleaseByteArrayElements {
                release(self.env, self.array, self.elements, self.release_mode);
            }
        }
    }
}
