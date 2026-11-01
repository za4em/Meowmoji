use crate::runtime::{Accelerator, RunError};
use crate::{EMOJI_COUNT, EmojiScores, TOKEN_SLOTS};

pub(crate) struct ModelRequest {
    pub model_path: String,
    pub accelerator: Accelerator,
}

struct ByteReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn is_at_end(&self) -> bool {
        self.position == self.bytes.len()
    }

    fn read_bytes(&mut self, byte_count: usize) -> Result<&'a [u8], RunError> {
        let end = self
            .position
            .checked_add(byte_count)
            .ok_or(RunError::MalformedWindow)?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or(RunError::MalformedWindow)?;
        self.position = end;
        Ok(slice)
    }

    fn read_four_bytes(&mut self) -> Result<[u8; 4], RunError> {
        let bytes = self.read_bytes(4)?;
        let mut four_bytes = [0; 4];
        four_bytes.copy_from_slice(bytes);
        Ok(four_bytes)
    }

    fn read_u32_le(&mut self) -> Result<u32, RunError> {
        Ok(u32::from_le_bytes(self.read_four_bytes()?))
    }

    fn read_i32_le(&mut self) -> Result<i32, RunError> {
        Ok(i32::from_le_bytes(self.read_four_bytes()?))
    }
}

/// Read the model file and accelerator that `meowmoji_open` compiles.
pub(crate) fn read_model_request(bytes: &[u8]) -> Result<ModelRequest, RunError> {
    let mut reader = ByteReader::new(bytes);
    let path_byte_count = reader.read_u32_le()? as usize;
    let path_bytes = reader.read_bytes(path_byte_count)?;
    let model_path = std::str::from_utf8(path_bytes)
        .map_err(|_| RunError::MalformedWindow)?
        .to_owned();
    if model_path.is_empty() {
        return Err(RunError::MalformedWindow);
    }
    let accelerator = Accelerator::from_u32(reader.read_u32_le()?)?;
    if !reader.is_at_end() {
        return Err(RunError::MalformedWindow);
    }
    Ok(ModelRequest {
        model_path,
        accelerator,
    })
}

/// Read the four token ids that `meowmoji_run` scores.
pub(crate) fn read_token_ids(bytes: &[u8]) -> Result<[i32; TOKEN_SLOTS], RunError> {
    let mut reader = ByteReader::new(bytes);
    let mut token_ids = [0; TOKEN_SLOTS];
    for token_id in &mut token_ids {
        *token_id = reader.read_i32_le()?;
    }
    if !reader.is_at_end() {
        return Err(RunError::MalformedWindow);
    }
    Ok(token_ids)
}

/// Write the one hundred emoji scores into the output buffer.
pub(crate) fn write_emoji_scores(
    emoji_scores: &EmojiScores,
    output: &mut [u8],
) -> Result<(), RunError> {
    let required_bytes = EMOJI_COUNT * 4;
    if output.len() < required_bytes {
        return Err(RunError::OutputTooSmall);
    }
    for (emoji_index, score) in emoji_scores.iter().enumerate() {
        let byte_offset = emoji_index * 4;
        output[byte_offset..byte_offset + 4].copy_from_slice(&score.to_le_bytes());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32_le(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_i32_le(bytes: &mut Vec<u8>, value: i32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn model_request_bytes(path: &str, accelerator: u32) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_u32_le(&mut bytes, path.len() as u32);
        bytes.extend_from_slice(path.as_bytes());
        push_u32_le(&mut bytes, accelerator);
        bytes
    }

    #[test]
    fn reads_a_cpu_model_request() {
        let bytes = model_request_bytes("model/meowmoji.tflite", 0);
        let request = read_model_request(&bytes).unwrap();
        assert_eq!(request.model_path, "model/meowmoji.tflite");
        assert_eq!(request.accelerator, Accelerator::Cpu);
    }

    #[test]
    fn reads_four_token_ids() {
        let mut bytes = Vec::new();
        for token_id in [7, 8, 0, 0] {
            push_i32_le(&mut bytes, token_id);
        }
        assert_eq!(read_token_ids(&bytes).unwrap(), [7, 8, 0, 0]);
    }

    #[test]
    fn rejects_a_short_buffer() {
        assert!(matches!(
            read_model_request(&[1, 0, 0, 0]),
            Err(RunError::MalformedWindow)
        ));
        assert!(matches!(
            read_token_ids(&[1, 0, 0, 0]),
            Err(RunError::MalformedWindow)
        ));
    }

    #[test]
    fn writes_scores_little_endian() {
        let mut emoji_scores = [0.0; EMOJI_COUNT];
        emoji_scores[0] = 1.0;
        emoji_scores[99] = 0.5;
        let mut output = vec![0; EMOJI_COUNT * 4];
        write_emoji_scores(&emoji_scores, &mut output).unwrap();
        assert_eq!(&output[0..4], &1.0f32.to_le_bytes());
        assert_eq!(&output[99 * 4..100 * 4], &0.5f32.to_le_bytes());
    }
}
