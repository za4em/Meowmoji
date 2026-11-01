use std::sync::Mutex;

use crate::EmojiScores;
use crate::TOKEN_SLOTS;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::apple::AppleRuntime;
#[cfg(any(target_os = "android", target_os = "linux", target_os = "windows"))]
use crate::litert::LiteRtRuntime;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Accelerator {
    Cpu,
    Gpu,
}

impl Accelerator {
    pub(crate) fn from_u32(raw_value: u32) -> Result<Self, RunError> {
        match raw_value {
            0 => Ok(Self::Cpu),
            1 => Ok(Self::Gpu),
            _ => Err(RunError::MalformedWindow),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub(crate) enum RunError {
    NullPointer,
    MalformedWindow,
    OutputTooSmall,
    ModelFileMissing,
    RuntimeNotConnected,
    RuntimeFailed,
}

/// LiteRT and the Apple runtime.
pub(crate) trait ModelRuntime: Send {
    fn compile_model(&mut self, model_path: &str, accelerator: Accelerator)
    -> Result<(), RunError>;
    fn score_tokens(&mut self, token_ids: &[i32; TOKEN_SLOTS]) -> Result<EmojiScores, RunError>;
}

struct LoadedModel {
    runtime: Box<dyn ModelRuntime>,
}

/// One compiled model. The pointer returned by `meowmoji_open` is this value.
pub(crate) struct ModelHandle {
    model: Mutex<LoadedModel>,
}

impl ModelHandle {
    pub(crate) fn score(&self, token_ids: &[i32; TOKEN_SLOTS]) -> Result<EmojiScores, RunError> {
        let mut loaded = self
            .model
            .lock()
            .unwrap_or_else(|crashed| crashed.into_inner());
        loaded.runtime.score_tokens(token_ids)
    }
}

/// Compile one model for one accelerator. The caller frees it with `meowmoji_close`.
pub(crate) fn open_model(
    model_path: &str,
    accelerator: Accelerator,
) -> Result<ModelHandle, RunError> {
    model_file_must_exist(model_path)?;
    let mut loaded = model_for_this_build();
    loaded.runtime.compile_model(model_path, accelerator)?;
    Ok(ModelHandle {
        model: Mutex::new(loaded),
    })
}

fn model_file_must_exist(model_path: &str) -> Result<(), RunError> {
    if std::path::Path::new(model_path).is_file() {
        Ok(())
    } else {
        Err(RunError::ModelFileMissing)
    }
}

fn model_for_this_build() -> LoadedModel {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    {
        LoadedModel {
            runtime: Box::new(AppleRuntime),
        }
    }
    #[cfg(any(target_os = "android", target_os = "linux", target_os = "windows"))]
    {
        LoadedModel {
            runtime: Box::new(LiteRtRuntime::new()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_model_file_fails_before_compile() {
        let error = open_model("no/such/model.tflite", Accelerator::Cpu);
        assert!(matches!(error, Err(RunError::ModelFileMissing)));
    }
}
