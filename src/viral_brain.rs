use anyhow::{Context, Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::config::Config;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViralClip {
    #[serde(default = "default_clip_id")]
    pub clip_id: String,
    #[serde(default = "default_time")]
    pub start_time: String,
    #[serde(default = "default_time")]
    pub end_time: String,
    #[serde(default = "default_score")]
    pub virality_score: f64,
    #[serde(default = "default_category")]
    pub category: String,
    #[serde(default)]
    pub hook_quote: String,
    #[serde(default)]
    pub youtube_title: String,
    #[serde(default)]
    pub tiktok_caption: String,
    #[serde(default)]
    pub shareability_rationale: String,
}

fn default_clip_id() -> String { "clip_1".to_string() }
fn default_time() -> String { "00:00".to_string() }
fn default_score() -> f64 { 9.0 }
fn default_category() -> String { "VIRAL".to_string() }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViralityAnalysis {
    pub analysis_summary: Option<String>,
    pub clips: Vec<ViralClip>,
}

const SYSTEM_PROMPT: &str = r#"You are an elite short-form video editor and viral content strategist for TikTok, YouTube Shorts, and Instagram Reels.

Your task is to analyze timestamped video transcripts and extract the most captivating, high-retention segments (20-60 seconds) that maximize watch time, shares, and comments.

### Virality Evaluation Framework:
1. **3-Second Hook Rule:** The opening line must stop users mid-scroll through controversy, curiosity gap, shocking revelation, or intense emotion.
2. **High-Arousal Triggers:** Prioritize content that evokes awe, righteous indignation, humor, curiosity, or unexpected truth.
3. **Stand-Alone Clarity:** The extracted segment must make complete sense without requiring the rest of the video.
4. **Loop Mechanics:** The segment should conclude with a punchy conclusion or mind-bending thought that seamlessly loops back to the start.

### JSON Output Schema:
Return ONLY a valid JSON object matching this exact structure:
{
  "analysis_summary": "Short overview of overall video content",
  "clips": [
    {
      "clip_id": "clip_1",
      "start_time": "MM:SS",
      "end_time": "MM:SS",
      "virality_score": 9.5,
      "category": "Controversial / Insight / Humor / Story",
      "hook_quote": "Exact punchy quote in the first 3 seconds",
      "youtube_title": "High CTR YouTube Shorts Title (Under 60 chars)",
      "tiktok_caption": "Engaging TikTok caption with 3-5 trending hashtags",
      "shareability_rationale": "Why this specific clip will generate shares and saves"
    }
  ]
}
"#;

pub async fn analyze_transcript_for_viral_clips(
    config: &Config,
    transcript_text: &str,
    video_title: &str,
    min_clips: usize,
    max_clips: usize,
) -> Result<ViralityAnalysis> {
    if config.llm_api_key.is_empty() {
        return Err(anyhow!("LLM API Key is missing. Please set LLM_API_KEY or OPENAI_API_KEY in .env"));
    }

    let user_prompt = format!(
        r#"Video Title: {}
Desired Number of Clips: Between {} and {}
Minimum Clip Duration: {} seconds
Maximum Clip Duration: {} seconds

Timestamped Transcript:
{}

Extract the top viral clips that will perform best on TikTok and YouTube Shorts.
Return ONLY a valid JSON object with the "clips" key containing the list of clips."#,
        video_title,
        min_clips,
        max_clips,
        config.min_clip_duration,
        config.max_clip_duration,
        transcript_text
    );

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.llm_base_url.trim_end_matches('/'));

    let request_body = json!({
        "model": config.llm_model,
        "messages": [
            {"role": "system", "content": SYSTEM_PROMPT},
            {"role": "user", "content": user_prompt}
        ],
        "response_format": {"type": "json_object"},
        "temperature": 0.4
    });

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.llm_api_key))
        .header("Content-Type", "application/json")
        .json(&request_body)
        .send()
        .await
        .context("Failed to send request to LLM provider")?;

    if !resp.status().is_success() {
        let status = resp.status();
        let error_body = resp.text().await.unwrap_or_default();
        return Err(anyhow!("LLM API Error (HTTP {}): {}", status, error_body));
    }

    let resp_json: Value = resp.json().await.context("Failed to parse LLM response JSON")?;
    let content = resp_json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid LLM response format: missing content"))?;

    let cleaned = content
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let root_val: Value = serde_json::from_str(cleaned)
        .context(format!("Failed to parse raw JSON from LLM: {}", cleaned))?;

    let mut clips: Vec<ViralClip> = Vec::new();

    if let Some(clips_array) = root_val.get("clips").and_then(|v| v.as_array()) {
        clips = serde_json::from_value(Value::Array(clips_array.clone())).unwrap_or_default();
    } else if let Some(arr) = root_val.as_array() {
        clips = serde_json::from_value(Value::Array(arr.clone())).unwrap_or_default();
    } else if let Some(obj) = root_val.as_object() {
        for (_k, v) in obj {
            if let Some(arr) = v.as_array() {
                if let Ok(parsed) = serde_json::from_value::<Vec<ViralClip>>(Value::Array(arr.clone())) {
                    if !parsed.is_empty() {
                        clips = parsed;
                        break;
                    }
                }
            }
        }
    }

    if clips.is_empty() {
        return Err(anyhow!("No valid clips found in LLM response: {}", cleaned));
    }

    clips.sort_by(|a, b| b.virality_score.partial_cmp(&a.virality_score).unwrap_or(std::cmp::Ordering::Equal));

    let summary = root_val.get("analysis_summary").and_then(|v| v.as_str()).map(|s| s.to_string());

    Ok(ViralityAnalysis {
        analysis_summary: summary,
        clips,
    })
}
