use crate::runtime::{Accelerator, ModelRuntime, RunError};
use crate::{EmojiScores, TOKEN_SLOTS};

pub(crate) struct LiteRtRuntime;

impl ModelRuntime for LiteRtRuntime {
    fn compile_model(
        &mut self,
        _model_path: &str,
        _accelerator: Accelerator,
    ) -> Result<(), RunError> {
        Err(RunError::RuntimeNotConnected)
    }

    fn score_tokens(&mut self, _token_ids: &[i32; TOKEN_SLOTS]) -> Result<EmojiScores, RunError> {
        Err(RunError::RuntimeNotConnected)
    }
}
