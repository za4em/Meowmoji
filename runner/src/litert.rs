mod c_api;
mod compile;
mod score;

use crate::runtime::{Accelerator, ModelRuntime, RunError};
use crate::{EmojiScores, TOKEN_SLOTS};
use c_api::{
    LiteRtCompiledModel, LiteRtEnvironment, LiteRtModel, LiteRtOptions, LiteRtTensorBuffer,
    destroy_compiled_model, destroy_model, destroy_options, destroy_tensor_buffer,
};

pub(crate) struct LiteRtRuntime {
    compiled: Option<CompiledLiteRtModel>,
}

impl LiteRtRuntime {
    pub(crate) fn new() -> Self {
        Self { compiled: None }
    }
}

struct CompiledLiteRtModel {
    environment: LiteRtEnvironment,
    model: LiteRtModel,
    options: LiteRtOptions,
    compiled_model: LiteRtCompiledModel,
    token_buffer: LiteRtTensorBuffer,
    score_buffer: LiteRtTensorBuffer,
}

unsafe impl Send for CompiledLiteRtModel {}

impl CompiledLiteRtModel {
    fn empty() -> Self {
        Self {
            environment: std::ptr::null_mut(),
            model: std::ptr::null_mut(),
            options: std::ptr::null_mut(),
            compiled_model: std::ptr::null_mut(),
            token_buffer: std::ptr::null_mut(),
            score_buffer: std::ptr::null_mut(),
        }
    }
}

impl Drop for CompiledLiteRtModel {
    fn drop(&mut self) {
        if !self.score_buffer.is_null() {
            destroy_tensor_buffer(self.score_buffer);
        }
        if !self.token_buffer.is_null() {
            destroy_tensor_buffer(self.token_buffer);
        }
        if !self.compiled_model.is_null() {
            destroy_compiled_model(self.compiled_model);
        }
        if !self.options.is_null() {
            destroy_options(self.options);
        }
        if !self.model.is_null() {
            destroy_model(self.model);
        }
    }
}

impl ModelRuntime for LiteRtRuntime {
    fn compile_model(
        &mut self,
        model_path: &str,
        accelerator: Accelerator,
    ) -> Result<(), RunError> {
        self.compiled = None;
        self.compiled = Some(compile::compile_litert_model(model_path, accelerator)?);
        Ok(())
    }

    fn score_tokens(&mut self, token_ids: &[i32; TOKEN_SLOTS]) -> Result<EmojiScores, RunError> {
        let Some(compiled) = self.compiled.as_mut() else {
            return Err(RunError::RuntimeNotConnected);
        };
        score::write_token_ids(compiled.token_buffer, token_ids)?;
        score::run_compiled_model(
            compiled.compiled_model,
            &mut compiled.token_buffer,
            &mut compiled.score_buffer,
        )?;
        score::read_emoji_scores(compiled.score_buffer)
    }
}
