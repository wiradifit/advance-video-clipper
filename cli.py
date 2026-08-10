"""
Command-line orchestrator for the Advance Video Clipper Engine.
"""
import sys
import os
import argparse
import logging
import tempfile
import shutil
from pathlib import Path

# Add current module directory to sys.path
CURRENT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(CURRENT_DIR))

from config import (
    LLM_MODEL,
    OUTPUT_DIR
)
from transcript import (
    extract_video_id,
    fetch_transcript,
    build_llm_transcript_text,
    timestamp_to_seconds
)
from viral_brain import analyze_transcript_for_viral_clips
from whisper_sync import extract_word_timestamps_from_audio, build_karaoke_ass
from renderer import download_video_slice, render_vertical_short

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%H:%M:%S"
)
logger = logging.getLogger("advance-clipper")

# Subtitle presets
PRESETS = {
    "hormozi": {
        "font_name": "Arial Black",
        "font_size": 74,
        "highlight_color": "00E6FF",  # Yellow in BGR
        "primary_color": "FFFFFF",
        "words_per_card": 3,
        "margin_v": 740
    },
    "cyber_cyan": {
        "font_name": "Trebuchet MS",
        "font_size": 68,
        "highlight_color": "FFFF00",  # Cyan in BGR
        "primary_color": "FFFFFF",
        "words_per_card": 3,
        "margin_v": 740
    },
    "neon_green": {
        "font_name": "Impact",
        "font_size": 76,
        "highlight_color": "66FF00",  # Lime in BGR
        "primary_color": "FFFFFF",
        "words_per_card": 2,
        "margin_v": 740
    }
}

def run_pipeline(
    url: str,
    top_n: int = 2,
    preset: str = "hormozi",
    crop_mode: str = "center_crop",
    output_directory: Path = OUTPUT_DIR,
    language: str = None,
    analyze_only: bool = False
):
    video_id = extract_video_id(url)
    video_url = f"https://www.youtube.com/watch?v={video_id}"
    
    print("\n" + "="*70)
    print(f"🎬 ADVANCE AI VIDEO CLIPPER: Processing Video ID [{video_id}]")
    print("="*70 + "\n")
    
    # 1. Ingest Transcript
    logger.info("📥 Step 1: Fetching video transcript...")
    raw_snippets = fetch_transcript(video_id)
    logger.info(f"Retrieved {len(raw_snippets)} transcript snippets.")
    
    formatted_transcript = build_llm_transcript_text(raw_snippets, group_interval_sec=15.0)
    
    # 2. Viral Brain AI Analysis
    logger.info(f"🧠 Step 2: Evaluating virality with LLM ({LLM_MODEL})...")
    brain_analysis = analyze_transcript_for_viral_clips(
        transcript_text=formatted_transcript,
        video_title=f"YouTube Video {video_id}",
        min_clips=max(2, top_n),
        max_clips=max(5, top_n + 2)
    )
    
    clips = brain_analysis.get("clips", [])
    if not clips:
        logger.error("No clips were identified by the AI model.")
        return []
        
    print("\n" + "─"*70)
    print("📊 AI VIRAL BRAIN SCORECARD & CANDIDATES")
    print("─"*70)
    for idx, clip in enumerate(clips, 1):
        score = clip.get("virality_score", 0.0)
        cat = clip.get("category", "General")
        hook = clip.get("hook_quote", "")
        start = clip.get("start_time", "00:00")
        end = clip.get("end_time", "00:00")
        yt_title = clip.get("youtube_title", "")
        tt_caption = clip.get("tiktok_caption", "")
        
        print(f"\n🔥 [Clip #{idx}] {clip.get('clip_id', f'clip_{idx}')} | Score: {score}/10 | {cat}")
        print(f"   ⏱️ Timestamp: {start} -> {end}")
        print(f"   🎯 Hook (0-3s): \"{hook}\"")
        print(f"   📺 YouTube Shorts Title: {yt_title}")
        print(f"   📱 TikTok Caption: {tt_caption}")
        print(f"   💡 Virality Rationale: {clip.get('shareability_rationale', '')}")
        
    if analyze_only:
        print("\n✅ Analysis complete (--analyze-only mode).")
        return clips

    # Select top N clips
    selected_clips = clips[:top_n]
    print("\n" + "─"*70)
    print(f"⚙️ Rendering Top {len(selected_clips)} Viral Clips (Framing: {crop_mode} | Preset: {preset})...")
    print("─"*70 + "\n")
    
    output_directory.mkdir(parents=True, exist_ok=True)
    results = []
    
    cfg_preset = PRESETS.get(preset, PRESETS["hormozi"])
    
    for idx, clip in enumerate(selected_clips, 1):
        clip_id = clip.get("clip_id", f"clip_{idx}")
        start_ts = clip.get("start_time")
        end_ts = clip.get("end_time")
        start_sec = timestamp_to_seconds(start_ts)
        end_sec = timestamp_to_seconds(end_ts)
        duration = end_sec - start_sec
        category = clip.get("category", "VIRAL")
        
        clean_filename = f"{video_id}_{clip_id}_{int(start_sec)}s_{int(end_sec)}s.mp4"
        final_output_path = output_directory / clean_filename
        
        logger.info(f"Processing Clip #{idx} ({start_ts} to {end_ts}, {duration:.1f}s)...")
        
        # Ephemeral processing workspace
        with tempfile.TemporaryDirectory(prefix="clipper_render_") as tmpdir:
            tmp_path = Path(tmpdir)
            raw_slice = tmp_path / "raw_slice.mp4"
            ass_file = tmp_path / "subtitles.ass"
            rendered_temp = tmp_path / clean_filename
            
            # Step A: Download slice
            download_video_slice(video_url, start_ts, end_ts, raw_slice)
            
            # Step B: Direct audio waveform speech recognition
            logger.info("🎙️ Transcribing audio slice for frame-perfect word sync...")
            words = extract_word_timestamps_from_audio(raw_slice, language=language)
            
            # Step C: Build high-impact active word karaoke subtitles
            header_tag = f"🔥 {category.upper()}"
            build_karaoke_ass(
                words=words,
                output_ass_path=ass_file,
                words_per_card=cfg_preset["words_per_card"],
                header_tag=header_tag,
                font_name=cfg_preset["font_name"],
                font_size=cfg_preset["font_size"],
                highlight_color_hex=cfg_preset["highlight_color"],
                primary_color_hex=cfg_preset["primary_color"],
                margin_v=cfg_preset["margin_v"]
            )
            
            # Step D: Render 1080x1920 9:16 vertical video
            render_vertical_short(raw_slice, rendered_temp, ass_file, crop_mode=crop_mode)
            
            # Step E: Save to final destination
            shutil.copy2(rendered_temp, final_output_path)
            
        results.append({
            "clip_id": clip_id,
            "title": clip.get("youtube_title"),
            "caption": clip.get("tiktok_caption"),
            "output_file": str(final_output_path),
            "score": clip.get("virality_score"),
            "timestamp": f"{start_ts} -> {end_ts}"
        })
        
    print("\n" + "="*70)
    print("🎉 ALL CLIPS SUCCESSFULLY RENDERED!")
    print("="*70)
    for r in results:
        print(f"\n🎬 {r['clip_id']} ({r['timestamp']}) | Score: {r['score']}/10")
        print(f"   📌 Title: {r['title']}")
        print(f"   📁 Local Video Path: {r['output_file']}")
        
    return results

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Advance AI Video Clipper for TikTok & YouTube Shorts")
    parser.add_argument("--url", type=str, required=True, help="YouTube Video URL or ID")
    parser.add_argument("--top", type=int, default=2, help="Number of top clips to render (default: 2)")
    parser.add_argument("--preset", type=str, default="hormozi", choices=["hormozi", "cyber_cyan", "neon_green"], help="Subtitle style preset (default: hormozi)")
    parser.add_argument("--crop-mode", type=str, default="center_crop", choices=["center_crop", "blur_pad"], help="Framing mode: center_crop (full-screen 9:16) or blur_pad")
    parser.add_argument("--lang", type=str, default=None, help="Spoken language code for Whisper (e.g. en, id, es, fr; default: auto-detect)")
    parser.add_argument("--out-dir", type=str, default=None, help="Custom output directory")
    parser.add_argument("--analyze-only", action="store_true", help="Only run AI scorecard analysis without rendering")
    args = parser.parse_args()
    
    out_dir = Path(args.out_dir) if args.out_dir else OUTPUT_DIR
    run_pipeline(
        url=args.url,
        top_n=args.top,
        preset=args.preset,
        crop_mode=args.crop_mode,
        output_directory=out_dir,
        language=args.lang,
        analyze_only=args.analyze_only
    )
