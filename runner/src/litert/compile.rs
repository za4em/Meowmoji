use std::ffi::CString;
use std::sync::Mutex;

use crate::runtime::{Accelerator, RunError};

use super::CompiledLiteRtModel;
use super::c_api::{
    FIRST_SIGNATURE, FIRST_TENSOR, LITERT_CPU, LITERT_GPU, LITERT_OK, LiteRtCompiledModel,
    LiteRtCreateCompiledModel, LiteRtCreateEnvironment,
    LiteRtCreateManagedTensorBufferFromRequirements, LiteRtCreateModelFromFile,
    LiteRtCreateOptions, LiteRtEnvironment, LiteRtGetCompiledModelInputBufferRequirements,
    LiteRtGetCompiledModelOutputBufferRequirements, LiteRtGetModelSignature,
    LiteRtGetRankedTensorType, LiteRtGetSignatureInputTensorByIndex,
    LiteRtGetSignatureOutputTensorByIndex, LiteRtModel, LiteRtOptions,
    LiteRtSetOptionsHardwareAccelerators, LiteRtSignature, LiteRtTensor, LiteRtTensorBuffer,
    LiteRtTensorBufferRequirements, RankedTensorType, destroy_options,
};

static ENVIRONMENT: Mutex<usize> = Mutex::new(0);

#[derive(Clone, Copy)]
enum TensorEnd {
    TokenIds,
    EmojiScores,
}

pub(super) fn compile_litert_model(
    model_path: &str,
    accelerator: Accelerator,
) -> Result<CompiledLiteRtModel, RunError> {
    let mut compiled = CompiledLiteRtModel::empty();
    compiled.environment = shared_environment()?;
    compiled.model = model_from_file(compiled.environment, model_path)?;
    compiled.options = options_for_accelerator(accelerator)?;
    compiled.compiled_model =
        create_compiled_model(compiled.environment, compiled.model, compiled.options)?;
    compiled.token_buffer = tensor_buffer(
        compiled.environment,
        compiled.model,
        compiled.compiled_model,
        TensorEnd::TokenIds,
    )?;
    compiled.score_buffer = tensor_buffer(
        compiled.environment,
        compiled.model,
        compiled.compiled_model,
        TensorEnd::EmojiScores,
    )?;
    Ok(compiled)
}

fn shared_environment() -> Result<LiteRtEnvironment, RunError> {
    let mut slot = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if *slot != 0 {
        return Ok(*slot as LiteRtEnvironment);
    }
    let created = create_environment()?;
    *slot = created as usize;
    Ok(created)
}

fn create_environment() -> Result<LiteRtEnvironment, RunError> {
    let mut environment = std::ptr::null_mut();
    let status = unsafe { LiteRtCreateEnvironment(0, std::ptr::null(), &mut environment) };
    if status != LITERT_OK || environment.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(environment)
}

fn model_from_file(
    environment: LiteRtEnvironment,
    model_path: &str,
) -> Result<LiteRtModel, RunError> {
    let path = CString::new(model_path).map_err(|_| RunError::RuntimeFailed)?;
    let mut model = std::ptr::null_mut();
    let status = unsafe { LiteRtCreateModelFromFile(environment, path.as_ptr(), &mut model) };
    if status != LITERT_OK || model.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(model)
}

fn options_for_accelerator(accelerator: Accelerator) -> Result<LiteRtOptions, RunError> {
    let mut options = std::ptr::null_mut();
    let created = unsafe { LiteRtCreateOptions(&mut options) };
    if created != LITERT_OK || options.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    let chip = match accelerator {
        Accelerator::Cpu => LITERT_CPU,
        Accelerator::Gpu => LITERT_GPU | LITERT_CPU,
    };
    let set = unsafe { LiteRtSetOptionsHardwareAccelerators(options, chip) };
    if set != LITERT_OK {
        destroy_options(options);
        return Err(RunError::RuntimeFailed);
    }
    Ok(options)
}

fn create_compiled_model(
    environment: LiteRtEnvironment,
    model: LiteRtModel,
    options: LiteRtOptions,
) -> Result<LiteRtCompiledModel, RunError> {
    let mut compiled_model = std::ptr::null_mut();
    let status =
        unsafe { LiteRtCreateCompiledModel(environment, model, options, &mut compiled_model) };
    if status != LITERT_OK || compiled_model.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(compiled_model)
}

fn tensor_buffer(
    environment: LiteRtEnvironment,
    model: LiteRtModel,
    compiled_model: LiteRtCompiledModel,
    end: TensorEnd,
) -> Result<LiteRtTensorBuffer, RunError> {
    let signature = model_signature(model)?;
    let tensor = signature_tensor(signature, end)?;
    let tensor_type = ranked_tensor_type(tensor)?;
    let requirements = buffer_requirements(compiled_model, end)?;
    managed_tensor_buffer(environment, &tensor_type, requirements)
}

fn model_signature(model: LiteRtModel) -> Result<LiteRtSignature, RunError> {
    let mut signature = std::ptr::null_mut();
    let status = unsafe { LiteRtGetModelSignature(model, FIRST_SIGNATURE, &mut signature) };
    if status != LITERT_OK || signature.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(signature)
}

fn signature_tensor(signature: LiteRtSignature, end: TensorEnd) -> Result<LiteRtTensor, RunError> {
    let mut tensor = std::ptr::null_mut();
    let status = unsafe {
        match end {
            TensorEnd::TokenIds => {
                LiteRtGetSignatureInputTensorByIndex(signature, FIRST_TENSOR, &mut tensor)
            }
            TensorEnd::EmojiScores => {
                LiteRtGetSignatureOutputTensorByIndex(signature, FIRST_TENSOR, &mut tensor)
            }
        }
    };
    if status != LITERT_OK || tensor.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(tensor)
}

fn ranked_tensor_type(tensor: LiteRtTensor) -> Result<RankedTensorType, RunError> {
    let mut tensor_type = unsafe { std::mem::zeroed() };
    let status = unsafe { LiteRtGetRankedTensorType(tensor, &mut tensor_type) };
    if status != LITERT_OK {
        return Err(RunError::RuntimeFailed);
    }
    Ok(tensor_type)
}

fn buffer_requirements(
    compiled_model: LiteRtCompiledModel,
    end: TensorEnd,
) -> Result<LiteRtTensorBufferRequirements, RunError> {
    let mut requirements = std::ptr::null_mut();
    let status = unsafe {
        match end {
            TensorEnd::TokenIds => LiteRtGetCompiledModelInputBufferRequirements(
                compiled_model,
                FIRST_SIGNATURE,
                FIRST_TENSOR,
                &mut requirements,
            ),
            TensorEnd::EmojiScores => LiteRtGetCompiledModelOutputBufferRequirements(
                compiled_model,
                FIRST_SIGNATURE,
                FIRST_TENSOR,
                &mut requirements,
            ),
        }
    };
    if status != LITERT_OK || requirements.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(requirements)
}

fn managed_tensor_buffer(
    environment: LiteRtEnvironment,
    tensor_type: &RankedTensorType,
    requirements: LiteRtTensorBufferRequirements,
) -> Result<LiteRtTensorBuffer, RunError> {
    let mut buffer = std::ptr::null_mut();
    let status = unsafe {
        LiteRtCreateManagedTensorBufferFromRequirements(
            environment,
            tensor_type,
            requirements,
            &mut buffer,
        )
    };
    if status != LITERT_OK || buffer.is_null() {
        return Err(RunError::RuntimeFailed);
    }
    Ok(buffer)
}
