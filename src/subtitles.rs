use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use crate::whisper::WordTimestamp;
use crate::filter::filter_sensitive_text;

#[derive(Debug, Clone)]
pub struct SubtitlePreset {
    pub name: &'static str,
    pub font_name: &'static str,
    pub font_size: u32,
    pub primary_color_bgr: &'static str,
    pub words_per_card: usize,
    pub margin_v: u32,
}

// STRICT SINGLE-WORD PRESETS (Zero line wrapping, zero vertical stacking)
pub const PRESET_HORMOZI: SubtitlePreset = SubtitlePreset {
    name: "hormozi",
    font_name: "Arial Black",
    font_size: 84,
    primary_color_bgr: "00E6FF", // Electric Yellow (&H0000E6FF)
    words_per_card: 1, // Strictly 1 single word per event card
    margin_v: 720,
};

pub const PRESET_CYBER_CYAN: SubtitlePreset = SubtitlePreset {
    name: "cyber_cyan",
    font_name: "Trebuchet MS",
    font_size: 80,
    primary_color_bgr: "FFFF00", // Cyan
    words_per_card: 1,
    margin_v: 720,
};

pub const PRESET_NEON_GREEN: SubtitlePreset = SubtitlePreset {
    name: "neon_green",
    font_name: "Impact",
    font_size: 88,
    primary_color_bgr: "66FF00", // Lime Green
    words_per_card: 1,
    margin_v: 720,
};

pub fn get_preset_by_name(name: &str) -> SubtitlePreset {
    match name {
        "cyber_cyan" => PRESET_CYBER_CYAN,
        "neon_green" => PRESET_NEON_GREEN,
        _ => PRESET_HORMOZI,
    }
}

pub fn format_ass_time(seconds: f64) -> String {
    let hrs = (seconds / 3600.0).floor() as u64;
    let mins = ((seconds % 3600.0) / 60.0).floor() as u64;
    let secs = (seconds % 60.0).floor() as u64;
    let mut centis = ((seconds - seconds.floor()) * 100.0).round() as u64;
    if centis >= 100 {
        centis = 99;
    }
    format!("{}:{:02}:{:02}.{:02}", hrs, mins, secs, centis)
}

pub fn build_karaoke_ass(
    words: &[WordTimestamp],
    output_ass_path: &Path,
    preset: &SubtitlePreset,
    header_tag: &str,
) -> Result<()> {
    let primary_ass = format!("&H00{}&", preset.primary_color_bgr);

    // WrapStyle: 2 strictly prevents line-breaking / stacking across all ASS renderers
    // HeaderTag: Top-Center alignment (8), MarginV: 240, Size: 40pt with thick black outline for clickbait title
    let ass_header = format!(
        r#"[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920
WrapStyle: 2
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: MainStyle,{},{},{},&H00FFFFFF,&H00000000,&H90000000,-1,0,0,0,100,100,2.0,0,1,7.5,2.5,2,30,30,{},1
Style: HeaderTag,{},40,&H00FFFFFF,&H0000E6FF,&H00000000,&H90000000,-1,0,0,0,100,100,1.2,0,1,5.0,2.0,8,30,30,240,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
        preset.font_name,
        preset.font_size,
        primary_ass,
        preset.margin_v,
        preset.font_name
    );

    let mut events = Vec::new();

    if !header_tag.is_empty() {
        // Apply Social Media Word Filter to top clickbait title header
        let clean_header = filter_sensitive_text(&header_tag.to_uppercase());
        events.push(format!(
            "Dialogue: 0,0:00:00.00,1:00:00.00,HeaderTag,,0,0,0,,{}",
            clean_header
        ));
    }

    // Render STRICTLY 1 single word per event card with Social Media Word Filtering
    for (i, w) in words.iter().enumerate() {
        let raw_word = w
            .word
            .to_uppercase()
            .trim_matches(|c: char| c.is_ascii_punctuation())
            .to_string();

        if raw_word.is_empty() {
            continue;
        }

        // Apply Social Media Trigger & Profanity Filter (e.g. MATI -> M*TI, DARAH -> D*RAH, KILL -> K*LL)
        let filtered_word = filter_sensitive_text(&raw_word);

        let start_sec = w.start;
        let mut end_sec = w.end;

        // Ensure start_sec < end_sec and zero overlap with next word
        if i < words.len() - 1 {
            let next_start = words[i + 1].start;
            if end_sec > next_start {
                end_sec = next_start;
            }
            if end_sec <= start_sec {
                end_sec = start_sec + 0.12;
            }
        } else {
            end_sec += 0.15;
        }

        let start_str = format_ass_time(start_sec);
        let end_str = format_ass_time(end_sec);

        events.push(format!(
            "Dialogue: 1,{},{},MainStyle,,0,0,0,,{}",
            start_str, end_str, filtered_word
        ));
    }

    let mut file = File::create(output_ass_path)?;
    file.write_all(ass_header.as_bytes())?;
    file.write_all(events.join("\n").as_bytes())?;
    file.write_all(b"\n")?;

    Ok(())
}
