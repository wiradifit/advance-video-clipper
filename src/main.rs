mod config;
mod transcript;
mod viral_brain;
mod whisper;
mod filter;
mod subtitles;
mod renderer;

use std::path::PathBuf;
use anyhow::{Result, Context};
use clap::Parser;
use colored::*;
use tempfile::tempdir;

use config::Config;
use transcript::{extract_video_id, fetch_transcript, build_llm_transcript_text, timestamp_to_seconds};
use viral_brain::analyze_transcript_for_viral_clips;
use whisper::extract_word_timestamps;
use subtitles::{get_preset_by_name, build_karaoke_ass};
use renderer::{download_video_slice, render_vertical_short};

#[derive(Parser, Debug)]
#[command(name = "advance-clipper")]
#[command(author = "wiradifit <https://github.com/wiradifit>")]
#[command(version = "0.1.0")]
#[command(about = "High-Performance AI Video Clipper for TikTok & YouTube Shorts", long_about = None)]
struct Cli {
    /// YouTube Video URL or 11-char Video ID
    #[arg(short, long)]
    url: String,

    /// Number of top viral clips to render
    #[arg(short, long, default_value_t = 2)]
    top: usize,

    /// Subtitle preset: hormozi, cyber_cyan, neon_green
    #[arg(short, long, default_value = "hormozi")]
    preset: String,

    /// Framing mode: center_crop (full-screen 9:16) or blur_pad
    #[arg(short, long, default_value = "center_crop")]
    crop_mode: String,

    /// Custom output directory
    #[arg(short, long)]
    out_dir: Option<PathBuf>,

    /// Language code for Whisper STT (e.g. en, id, es; default: id for local podcasts)
    #[arg(short, long)]
    lang: Option<String>,

    /// Only run AI scorecard analysis without downloading/rendering video
    #[arg(long, default_value_t = false)]
    analyze_only: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    let config = Config::load();

    let video_id = extract_video_id(&args.url).context("Failed to parse YouTube Video ID")?;
    let video_url = format!("https://www.youtube.com/watch?v={}", video_id);

    println!("{}", "\n======================================================================".cyan().bold());
    println!("{} [{}]", "🎬 ADVANCE AI VIDEO CLIPPER (RUST ENGINE)".yellow().bold(), video_id.white().bold());
    println!("{}", "======================================================================\n".cyan().bold());

    // 1. Ingest Transcript
    println!("{} Fetching video transcript...", "📥 [1/5]".cyan().bold());
    let raw_snippets = fetch_transcript(&video_id).context("Failed to retrieve video transcript")?;
    println!("   Retrieved {} transcript segments.", raw_snippets.len().to_string().green().bold());

    let formatted_transcript = build_llm_transcript_text(&raw_snippets, 15.0);

    // 2. Virality Brain Analysis
    println!("{} Analyzing transcript with AI ({})", "🧠 [2/5]".cyan().bold(), config.llm_model.yellow().bold());
    let analysis = analyze_transcript_for_viral_clips(
        &config,
        &formatted_transcript,
        &format!("YouTube Video {}", video_id),
        args.top.max(2),
        args.top.max(4) + 1,
    )
    .await
    .context("Virality AI analysis failed")?;

    if analysis.clips.is_empty() {
        println!("{}", "❌ No viral clips were extracted by the AI model.".red().bold());
        return Ok(());
    }

    println!("\n{}", "──────────────────────────────────────────────────────────────────────".blue());
    println!("{}", "📊 2026 AI VIRAL BRAIN SCORECARD & CANDIDATES".yellow().bold());
    println!("{}", "──────────────────────────────────────────────────────────────────────".blue());

    for (idx, clip) in analysis.clips.iter().enumerate() {
        println!(
            "\n🔥 [{}] {} | Score: {}/10 | {}",
            format!("Clip #{}", idx + 1).yellow().bold(),
            clip.clip_id.cyan(),
            format!("{:.1}", clip.virality_score).green().bold(),
            clip.category.magenta()
        );
        println!("   ⏱️  Timestamp: {} -> {}", clip.start_time.white(), clip.end_time.white());
        println!("   🚨 Top Clickbait Header: {}", clip.top_clickbait_title.red().bold());
        println!("   🎯 Hook (0-3s): \"{}\"", clip.hook_quote.italic());
        println!("   📺 YouTube Shorts Title: {}", clip.youtube_title.bold());
        println!("   📱 TikTok Caption: {}", clip.tiktok_caption);
        println!("   💡 Virality Rationale: {}", clip.shareability_rationale.dimmed());
    }

    if args.analyze_only {
        println!("\n{}", "✅ Analysis complete (--analyze-only mode). Exiting.".green().bold());
        return Ok(());
    }

    // 3. Render Top N Clips
    let selected_clips: Vec<_> = analysis.clips.into_iter().take(args.top).collect();
    let out_dir = args.out_dir.unwrap_or(config.output_dir);
    std::fs::create_dir_all(&out_dir)?;

    let preset = get_preset_by_name(&args.preset);

    println!("\n{}", "──────────────────────────────────────────────────────────────────────".blue());
    println!(
        "⚙️  Rendering Top {} Viral Clips (Preset: {} | Framing: {})...",
        selected_clips.len().to_string().yellow().bold(),
        preset.name.green().bold(),
        args.crop_mode.cyan().bold()
    );
    println!("{}", "──────────────────────────────────────────────────────────────────────\n".blue());

    let mut results = Vec::new();

    for (idx, clip) in selected_clips.iter().enumerate() {
        let start_sec = timestamp_to_seconds(&clip.start_time);
        let end_sec = timestamp_to_seconds(&clip.end_time);
        let duration = end_sec - start_sec;

        let clean_filename = format!("{}_{}_{}s_{}s.mp4", video_id, clip.clip_id, start_sec as u64, end_sec as u64);
        let final_output = out_dir.join(&clean_filename);

        println!(
            "🎬 Processing Clip #{} [{} -> {} ({:.1}s)]...",
            idx + 1,
            clip.start_time.yellow(),
            clip.end_time.yellow(),
            duration
        );

        let tmp = tempdir()?;
        let slice_video = tmp.path().join("slice.mp4");
        let ass_file = tmp.path().join("subtitles.ass");
        let temp_render = tmp.path().join(&clean_filename);

        // Step A: Download slice
        print!("   📥 Slicing video stream via yt-dlp... ");
        download_video_slice(&video_url, &clip.start_time, &clip.end_time, &slice_video)
            .context("Slice download failed")?;
        println!("{}", "Done.".green());

        // Step B: Whisper Speech-to-Text
        print!("   🎙️  Transcribing audio for millisecond word timestamps... ");
        let words = extract_word_timestamps(&slice_video, args.lang.as_deref(), &config.whisper_model)
            .context("Whisper STT failed")?;
        println!("{} words extracted.", words.len().to_string().green());

        // Step C: Build Subtitles with Top Single-Line Clickbait Header & Word Filtering
        let header_tag = if !clip.top_clickbait_title.is_empty() {
            clip.top_clickbait_title.clone()
        } else {
            format!("🔥 {}", clip.category.to_uppercase())
        };
        build_karaoke_ass(&words, &ass_file, &preset, &header_tag)?;

        // Step D: Render 1080x1920 video
        print!("   ⚙️  Encoding 1080x1920 9:16 vertical video with FFmpeg... ");
        render_vertical_short(&slice_video, &temp_render, Some(&ass_file), &args.crop_mode)
            .context("Video rendering failed")?;
        println!("{}", "Done.".green());

        // Step E: Save to final destination
        std::fs::copy(&temp_render, &final_output)?;

        // Step F: Generate Copy-Paste Social Post Metadata File with Source Attribution & Filtered Text
        let meta_filename = format!("{}_{}_{}s_{}s.txt", video_id, clip.clip_id, start_sec as u64, end_sec as u64);
        let meta_output = out_dir.join(&meta_filename);

        let filtered_title = filter::filter_sensitive_text(&clip.youtube_title);
        let filtered_caption = filter::filter_sensitive_text(&clip.tiktok_caption);
        let filtered_header = filter::filter_sensitive_text(&clip.top_clickbait_title);

        let social_post_content = format!(
            "======================================================================\n\
             📱 READY-TO-POST SOCIAL MEDIA CAPTION & METADATA\n\
             ======================================================================\n\n\
             📺 YOUTUBE SHORTS TITLE:\n\
             {}\n\n\
             🚨 ON-SCREEN TOP CLICKBAIT HEADER:\n\
             {}\n\n\
             📱 TIKTOK / REELS / SHORTS CAPTION:\n\
             {}\n\n\
             📌 SOURCE ATTRIBUTION:\n\
             Source Video: {}\n\n\
             📊 VIRALITY SCORECARD:\n\
             Score: {:.1}/10 | Category: {}\n\
             Segment: {} -> {}\n\
             Rationale: {}\n\
             ======================================================================\n",
            filtered_title,
            filtered_header,
            filtered_caption,
            video_url,
            clip.virality_score,
            clip.category,
            clip.start_time,
            clip.end_time,
            clip.shareability_rationale,
        );

        std::fs::write(&meta_output, &social_post_content)?;

        println!("   ✅ Video Clip Saved: {}", final_output.display().to_string().cyan().bold());
        println!("   📝 Metadata & Caption File Saved: {}\n", meta_output.display().to_string().yellow().bold());

        results.push((clip.clone(), final_output, meta_output, social_post_content));
    }

    println!("{}", "======================================================================".cyan().bold());
    println!("{}", "🎉 ALL VIRAL SHORTS & CAPTIONS SUCCESSFULLY GENERATED!".green().bold());
    println!("{}", "======================================================================".cyan().bold());

    for (clip, video_path, meta_path, post_text) in &results {
        println!("\n{}", "──────────────────────────────────────────────────────────────────────".blue());
        println!("🔥 CLIP: {}", clip.clip_id.yellow().bold());
        println!("📁 Video File: {}", video_path.display().to_string().cyan());
        println!("📝 Caption File: {}", meta_path.display().to_string().yellow());
        println!("\n{}", post_text.white());
    }

    Ok(())
}
