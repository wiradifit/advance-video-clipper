"""
Video Rendering Engine using native FFmpeg.
Supports Full-Screen Vertical Center-Crop (1080x1920) and Blurred Background Padding.
"""
import sys
import os
import shutil
import subprocess
import logging
from pathlib import Path
from typing import Optional
from config import OUTPUT_WIDTH, OUTPUT_HEIGHT, FPS

logger = logging.getLogger(__name__)

def get_yt_dlp_cmd() -> list:
    """Returns command prefix to invoke yt-dlp."""
    yt_bin = shutil.which("yt-dlp")
    if yt_bin:
        return [yt_bin]
    venv_yt = Path(sys.executable).parent / "yt-dlp"
    if venv_yt.exists() and os.access(venv_yt, os.X_OK):
        return [str(venv_yt)]
    return [sys.executable, "-m", "yt_dlp"]

def get_ffmpeg_binary() -> str:
    """Finds the best available FFmpeg binary (imageio-ffmpeg or system PATH)."""
    try:
        import imageio_ffmpeg
        exe = imageio_ffmpeg.get_ffmpeg_exe()
        if os.path.exists(exe) and os.access(exe, os.X_OK):
            return exe
    except Exception:
        pass
        
    system_ffmpeg = shutil.which("ffmpeg")
    if system_ffmpeg:
        return system_ffmpeg
        
    for p in ("/usr/bin/ffmpeg", "/usr/local/bin/ffmpeg", "/opt/homebrew/bin/ffmpeg"):
        if os.path.exists(p) and os.access(p, os.X_OK):
            return p
            
    raise RuntimeError("No working FFmpeg executable found. Please install ffmpeg on your system.")

def download_video_slice(
    video_url: str,
    start_time_str: str,
    end_time_str: str,
    output_path: Path
) -> Path:
    """
    Downloads only the specified time section of a YouTube video using yt-dlp.
    """
    ffmpeg_exe = get_ffmpeg_binary()
    yt_cmd_base = get_yt_dlp_cmd()
    
    cmd = yt_cmd_base + [
        "--ffmpeg-location", ffmpeg_exe,
        "--download-sections", f"*{start_time_str}-{end_time_str}",
        "-f", "bestvideo[ext=mp4][height<=1080]+bestaudio[ext=m4a]/best[ext=mp4]/best",
        "--force-keyframes-at-cuts",
        "-o", str(output_path),
        "--no-playlist",
        video_url
    ]
    
    logger.info(f"Downloading slice [{start_time_str} -> {end_time_str}] to {output_path.name}...")
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        logger.warning("Retrying slice download with generic best format...")
        fallback_cmd = yt_cmd_base + [
            "--ffmpeg-location", ffmpeg_exe,
            "--download-sections", f"*{start_time_str}-{end_time_str}",
            "-f", "best",
            "--force-keyframes-at-cuts",
            "-o", str(output_path),
            "--no-playlist",
            video_url
        ]
        res_fb = subprocess.run(fallback_cmd, capture_output=True, text=True)
        if res_fb.returncode != 0:
            raise RuntimeError(f"Failed to download video slice: {res_fb.stderr}")
            
    return output_path

def render_vertical_short(
    input_video_path: Path,
    output_video_path: Path,
    ass_subtitle_path: Optional[Path] = None,
    crop_mode: str = "center_crop"  # "center_crop" or "blur_pad"
) -> Path:
    """
    Executes FFmpeg with 1080x1920 9:16 layout and burns subtitles.
    crop_mode:
      - 'center_crop': Full-screen 9:16 zoom-to-fill, centered on speaker (no blur borders).
      - 'blur_pad': 16:9 in middle with blurred expanded background at top/bottom.
    """
    ffmpeg_exe = get_ffmpeg_binary()
    
    sub_filter = ""
    if ass_subtitle_path and ass_subtitle_path.exists():
        escaped_ass = str(ass_subtitle_path).replace("\\", "/").replace(":", "\\:")
        sub_filter = f",subtitles='{escaped_ass}'"
        
    if crop_mode == "center_crop":
        filter_complex = f"[0:v]scale={OUTPUT_WIDTH}:{OUTPUT_HEIGHT}:force_original_aspect_ratio=increase,crop={OUTPUT_WIDTH}:{OUTPUT_HEIGHT}{sub_filter}[v]"
    else:
        filter_complex = (
            f"[0:v]scale={OUTPUT_WIDTH}:{OUTPUT_HEIGHT}:force_original_aspect_ratio=increase,"
            f"crop={OUTPUT_WIDTH}:{OUTPUT_HEIGHT},boxblur=25:25[bg];"
            f"[0:v]scale={OUTPUT_WIDTH}:-1[fg];"
            f"[bg][fg]overlay=(W-w)/2:(H-h)/2{sub_filter}[v]"
        )

    cmd = [
        ffmpeg_exe, "-y",
        "-i", str(input_video_path),
        "-filter_complex", filter_complex,
        "-map", "[v]",
        "-map", "0:a?",
        "-c:v", "libx264",
        "-preset", "veryfast",
        "-crf", "22",
        "-c:a", "aac",
        "-b:a", "192k",
        "-threads", "2",
        "-r", str(FPS),
        str(output_video_path)
    ]

    logger.info(f"Rendering 9:16 vertical video ({crop_mode}) to {output_video_path.name}...")
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        raise RuntimeError(f"FFmpeg rendering failed: {res.stderr}")

    return output_video_path
