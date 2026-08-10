use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::Result;
use crate::whisper::WordTimestamp;

#[derive(Debug, Clone)]
pub struct SubtitlePreset {
    pub name: &'static str,
    pub font_name: &'static str,
    pub font_size: u32,
    pub highlight_color_bgr: &'static str, // BGR hex without &H
    pub primary_color_bgr: &'static str,
    pub words_per_card: usize,
    pub margin_v: u32,
}

// Strictly 1-line, 2-word rapid rhythmic bursts with Electric Yellow highlight
pub const PRESET_HORMOZI: SubtitlePreset = SubtitlePreset {
    name: "hormozi",
    font_name: "Arial Black",
    font_size: 66,
    highlight_color_bgr: "00E6FF", // Electric Yellow (&H0000E6FF)
    primary_color_bgr: "FFFFFF",
    words_per_card: 2, // Strictly 2 words per card to guarantee 1 single line
    margin_v: 720,
};

pub const PRESET_CYBER_CYAN: SubtitlePreset = SubtitlePreset {
    name: "cyber_cyan",
    font_name: "Trebuchet MS",
    font_size: 64,
    highlight_color_bgr: "FFFF00", // Cyan
    primary_color_bgr: "FFFFFF",
    words_per_card: 2,
    margin_v: 720,
};

pub const PRESET_NEON_GREEN: SubtitlePreset = SubtitlePreset {
    name: "neon_green",
    font_name: "Impact",
    font_size: 68,
    highlight_color_bgr: "66FF00", // Lime Green
    primary_color_bgr: "FFFFFF",
    words_per_card: 2,
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
    let highlight_ass = format!("&H00{}&", preset.highlight_color_bgr);

    // WrapStyle: 2 strictly prevents line-breaking / stacking across all ASS renderers
    let ass_header = format!(
        r#"[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920
WrapStyle: 2
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: MainStyle,{},{},{},&H000000FF,&H00000000,&H90000000,-1,0,0,0,100,100,1.5,0,1,6.0,2.0,2,30,30,{},1
Style: HeaderTag,{},36,{},&H00FFFFFF,&H00000000,&H90000000,-1,0,0,0,100,100,1,0,1,4.0,1.5,8,30,30,320,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"#,
        preset.font_name,
        preset.font_size,
        primary_ass,
        preset.margin_v,
        preset.font_name,
        highlight_ass
    );

    let mut events = Vec::new();

    if !header_tag.is_empty() {
        events.push(format!(
            "Dialogue: 0,0:00:00.00,1:00:00.00,HeaderTag,,0,0,0,,{}",
            header_tag
        ));
    }

    // Group words into strictly 2-word single-line chunks
    let chunks: Vec<&[WordTimestamp]> = words.chunks(preset.words_per_card).collect();

    for chunk in chunks {
        let chunk_words_text: Vec<String> = chunk
            .iter()
            .map(|w| {
                w.word
                    .to_uppercase()
                    .trim_matches(|c: char| c.is_ascii_punctuation())
                    .to_string()
            })
            .filter(|w| !w.is_empty())
            .collect();

        if chunk_words_text.is_empty() {
            continue;
        }

        for (active_idx, active_word) in chunk.iter().enumerate() {
            let w_start = active_word.start;
            let w_end = active_word.end;

            let event_end = if active_idx < chunk.len() - 1 {
                let next_start = chunk[active_idx + 1].start;
                w_end.max(next_start)
            } else {
                w_end + 0.15
            };

            let start_str = format_ass_time(w_start);
            let end_str = format_ass_time(event_end);

            let mut styled_line_parts = Vec::new();
            for (idx, text) in chunk_words_text.iter().enumerate() {
                if idx == active_idx {
                    styled_line_parts.push(format!(r"{{\c{}}}{}{{\c{}}}", highlight_ass, text, primary_ass));
                } else {
                    styled_line_parts.push(text.clone());
                }
            }

            let line_text = styled_line_parts.join(" ");
            events.push(format!(
                "Dialogue: 1,{},{},MainStyle,,0,0,0,,{}",
                start_str, end_str, line_text
            ));
        }
    }

    let mut file = File::create(output_ass_path)?;
    file.write_all(ass_header.as_bytes())?;
    file.write_all(events.join("\n").as_bytes())?;
    file.write_all(b"\n")?;

    Ok(())
}
