use std::sync::{Mutex, OnceLock};

use crate::EmojiScores;
use crate::TOKEN_SLOTS;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use crate::apple::AppleRuntime;
use crate::buffer::Window;
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
    compiled_model_path: Option<String>,
    compiled_accelerator: Option<Accelerator>,
    runtime: Box<dyn ModelRuntime>,
}

impl LoadedModel {
    fn new(runtime: Box<dyn ModelRuntime>) -> Self {
        Self {
            compiled_model_path: None,
            compiled_accelerator: None,
            runtime,
        }
    }

    fn already_compiled(&self, model_path: &str, accelerator: Accelerator) -> bool {
        self.compiled_model_path.as_deref() == Some(model_path)
            && self.compiled_accelerator == Some(accelerator)
    }

    fn compile_if_needed(
        &mut self,
        model_path: &str,
        accelerator: Accelerator,
    ) -> Result<(), RunError> {
        if self.already_compiled(model_path, accelerator) {
            return Ok(());
        }
        model_file_must_exist(model_path)?;
        self.runtime.compile_model(model_path, accelerator)?;
        self.compiled_model_path = Some(model_path.to_owned());
        self.compiled_accelerator = Some(accelerator);
        Ok(())
    }

    fn score_window(&mut self, window: &Window) -> Result<EmojiScores, RunError> {
        self.compile_if_needed(&window.model_path, window.accelerator)?;
        self.runtime.score_tokens(&window.token_ids)
    }
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
        LoadedModel::new(Box::new(AppleRuntime))
    }
    #[cfg(any(target_os = "android", target_os = "linux", target_os = "windows"))]
    {
        LoadedModel::new(Box::new(LiteRtRuntime::new()))
    }
}

fn shared_loaded_model() -> &'static Mutex<LoadedModel> {
    static LOADED_MODEL: OnceLock<Mutex<LoadedModel>> = OnceLock::new();
    LOADED_MODEL.get_or_init(|| Mutex::new(model_for_this_build()))
}

/// Score one window on the runtime compiled into this build.
pub(crate) fn run_window(window: &Window) -> Result<EmojiScores, RunError> {
    let mut loaded_model = shared_loaded_model()
        .lock()
        .unwrap_or_else(|crashed| crashed.into_inner());
    loaded_model.score_window(window)
}
