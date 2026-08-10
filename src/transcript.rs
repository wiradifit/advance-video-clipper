use anyhow::{Context, Result, anyhow};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptSnippet {
    pub text: String,
    pub start: f64,
    pub duration: f64,
}

pub fn find_python_binary() -> String {
    for candidate in [
        ".venv/bin/python",
        "../.venv/bin/python",
        "../charming-curie/.venv/bin/python",
        "/opt/homebrew/bin/python3",
        "/usr/bin/python3",
    ] {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "python3".to_string()
}

pub fn find_ytdl_binary() -> String {
    for candidate in [
        ".venv/bin/yt-dlp",
        "../.venv/bin/yt-dlp",
        "../charming-curie/.venv/bin/yt-dlp",
        "/opt/homebrew/bin/yt-dlp",
        "/usr/local/bin/yt-dlp",
        "/usr/bin/yt-dlp",
    ] {
        if Path::new(candidate).exists() {
            return candidate.to_string();
        }
    }
    "yt-dlp".to_string()
}

pub fn extract_video_id(input: &str) -> Result<String> {
    if input.len() == 11 && !input.contains('/') && !input.contains('?') {
        return Ok(input.to_string());
    }

    let patterns = [
        r"(?:v=|\/)([0-9A-Za-z_-]{11}).*",
        r"(?:youtu\.be\/)([0-9A-Za-z_-]{11})",
        r"(?:embed\/)([0-9A-Za-z_-]{11})",
        r"(?:shorts\/)([0-9A-Za-z_-]{11})",
    ];

    for pat in patterns {
        let re = Regex::new(pat)?;
        if let Some(caps) = re.captures(input) {
            if let Some(m) = caps.get(1) {
                return Ok(m.as_str().to_string());
            }
        }
    }

    Err(anyhow!("Could not extract a valid YouTube video ID from: {}", input))
}

pub fn fetch_transcript(video_id: &str) -> Result<Vec<TranscriptSnippet>> {
    let python_bin = find_python_binary();
    let py_script = format!(
        r#"
import json, sys
try:
    from youtube_transcript_api import YouTubeTranscriptApi
    api = YouTubeTranscriptApi()
    transcripts = api.list('{}')
    t_obj = None
    for t in transcripts:
        if t.language_code in ['id', 'en', 'es', 'pt', 'fr']:
            t_obj = t
            break
    if not t_obj:
        for t in transcripts:
            t_obj = t
            break
    if t_obj:
        data = t_obj.fetch()
        out = []
        for s in data:
            if hasattr(s, 'text'):
                out.append({{'text': s.text, 'start': float(s.start), 'duration': float(s.duration)}})
            elif isinstance(s, dict):
                out.append({{'text': s.get('text', ''), 'start': float(s.get('start', 0.0)), 'duration': float(s.get('duration', 0.0))}})
        print(json.dumps(out))
    else:
        print("[]")
except Exception as e:
    print(f"Error: {{e}}", file=sys.stderr)
    print("[]")
"#,
        video_id
    );

    let output = Command::new(&python_bin)
        .arg("-c")
        .arg(&py_script)
        .output()
        .context("Failed to run transcript extractor")?;

    let stdout_str = String::from_utf8_lossy(&output.stdout);
    let snippets: Vec<TranscriptSnippet> = serde_json::from_str(stdout_str.trim())
        .unwrap_or_default();

    if !snippets.is_empty() {
        return Ok(snippets);
    }

    Err(anyhow!("No transcript found for video {}", video_id))
}

pub fn seconds_to_timestamp(seconds: f64) -> String {
    let total_secs = seconds.round() as u64;
    let hrs = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;

    if hrs > 0 {
        format!("{:02}:{:02}:{:02}", hrs, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

pub fn timestamp_to_seconds(ts_str: &str) -> f64 {
    let parts: Vec<&str> = ts_str.trim().split(':').collect();
    match parts.len() {
        3 => {
            let h: f64 = parts[0].parse().unwrap_or(0.0);
            let m: f64 = parts[1].parse().unwrap_or(0.0);
            let s: f64 = parts[2].parse().unwrap_or(0.0);
            h * 3600.0 + m * 60.0 + s
        }
        2 => {
            let m: f64 = parts[0].parse().unwrap_or(0.0);
            let s: f64 = parts[1].parse().unwrap_or(0.0);
            m * 60.0 + s
        }
        1 => parts[0].parse().unwrap_or(0.0),
        _ => 0.0,
    }
}

pub fn build_llm_transcript_text(snippets: &[TranscriptSnippet], group_interval_sec: f64) -> String {
    let mut blocks = Vec::new();
    let mut current_block_start = 0.0;
    let mut current_block_text = Vec::new();

    for s in snippets {
        let text = s.text.replace('\n', " ").trim().to_string();
        if text.is_empty() {
            continue;
        }

        if current_block_text.is_empty() {
            current_block_start = s.start;
        }

        if s.start - current_block_start >= group_interval_sec {
            let ts_label = seconds_to_timestamp(current_block_start);
            blocks.push(format!("[{}] {}", ts_label, current_block_text.join(" ")));
            current_block_start = s.start;
            current_block_text = vec![text];
        } else {
            current_block_text.push(text);
        }
    }

    if !current_block_text.is_empty() {
        let ts_label = seconds_to_timestamp(current_block_start);
        blocks.push(format!("[{}] {}", ts_label, current_block_text.join(" ")));
    }

    blocks.join("\n")
}
