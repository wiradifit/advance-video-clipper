use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;
use crate::transcript::find_python_binary;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordTimestamp {
    pub word: String,
    pub start: f64,
    pub end: f64,
    pub probability: f64,
}

pub fn extract_word_timestamps(
    media_path: &Path,
    language: Option<&str>,
    whisper_model: &str,
) -> Result<Vec<WordTimestamp>> {
    let python_bin = find_python_binary();
    let lang_code = language.unwrap_or("id"); // Default to Indonesian for highest accuracy on local podcasts
    let py_script = format!(
        r#"
import json, sys
from faster_whisper import WhisperModel

try:
    model = WhisperModel('{model}', device='cpu', compute_type='int8')
    lang_arg = None if '{lang}' in ['None', 'auto'] else '{lang}'
    segments, info = model.transcribe(
        r'{media_path}',
        language=lang_arg,
        word_timestamps=True,
        vad_filter=True,
        vad_parameters=dict(min_silence_duration_ms=300)
    )
    
    words = []
    for segment in segments:
        for w in segment.words:
            clean_w = w.word.strip()
            if clean_w:
                words.append({{
                    'word': clean_w,
                    'start': float(w.start),
                    'end': float(w.end),
                    'probability': float(w.probability)
                }})
    print(json.dumps(words))
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    sys.exit(1)
"#,
        model = whisper_model,
        lang = lang_code,
        media_path = media_path.display()
    );

    let output = Command::new(&python_bin)
        .arg("-c")
        .arg(&py_script)
        .output()
        .context("Failed to execute whisper transcription engine")?;

    if !output.status.success() {
        let err_str = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Whisper execution error: {}", err_str));
    }

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let words: Vec<WordTimestamp> = serde_json::from_str(stdout_str.trim())
        .context("Failed to parse word timestamps JSON from Whisper")?;

    Ok(words)
}
