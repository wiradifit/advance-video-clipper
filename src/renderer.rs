use anyhow::{Context, Result, anyhow};
use std::path::Path;
use std::process::Command;
use crate::transcript::{find_ytdl_binary, find_python_binary};

pub fn find_ffmpeg_binary() -> String {
    // 1. Check system PATH
    if let Ok(output) = Command::new("which").arg("ffmpeg").output() {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && Path::new(&path).exists() {
                return path;
            }
        }
    }

    // 2. Check standard installation directories
    for p in [
        "/usr/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/bin/ffmpeg",
    ] {
        if Path::new(p).exists() {
            return p.to_string();
        }
    }

    // 3. Check imageio_ffmpeg via python
    let python_bin = find_python_binary();
    if let Ok(output) = Command::new(&python_bin)
        .arg("-c")
        .arg("import imageio_ffmpeg; print(imageio_ffmpeg.get_ffmpeg_exe())")
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() && Path::new(&path).exists() {
                return path;
            }
        }
    }

    "ffmpeg".to_string()
}

pub fn download_video_slice(
    video_url: &str,
    start_time_str: &str,
    end_time_str: &str,
    output_path: &Path,
) -> Result<()> {
    let ffmpeg_exe = find_ffmpeg_binary();
    let ytdl_bin = find_ytdl_binary();
    let section_arg = format!("*{}", format!("{}-{}", start_time_str, end_time_str));

    let status = Command::new(&ytdl_bin)
        .args([
            "--ffmpeg-location",
            &ffmpeg_exe,
            "--download-sections",
            &section_arg,
            "-f",
            "bestvideo[ext=mp4][height<=1080]+bestaudio[ext=m4a]/best[ext=mp4]/best",
            "--force-keyframes-at-cuts",
            "-o",
            output_path.to_str().unwrap(),
            "--no-playlist",
            video_url,
        ])
        .status()
        .context("Failed to invoke yt-dlp slice download")?;

    if !status.success() {
        let fb_status = Command::new(&ytdl_bin)
            .args([
                "--ffmpeg-location",
                &ffmpeg_exe,
                "--download-sections",
                &section_arg,
                "-f",
                "best",
                "--force-keyframes-at-cuts",
                "-o",
                output_path.to_str().unwrap(),
                "--no-playlist",
                video_url,
            ])
            .status()
            .context("Failed to run yt-dlp fallback download")?;

        if !fb_status.success() {
            return Err(anyhow!("Failed to download video slice from YouTube"));
        }
    }

    Ok(())
}

pub fn render_vertical_short(
    input_path: &Path,
    output_path: &Path,
    ass_subtitle_path: Option<&Path>,
    crop_mode: &str,
) -> Result<()> {
    let ffmpeg_exe = find_ffmpeg_binary();

    let sub_filter = if let Some(ass) = ass_subtitle_path {
        let escaped = ass.to_str().unwrap().replace('\\', "/").replace(':', "\\:");
        format!(",subtitles='{}'", escaped)
    } else {
        String::new()
    };

    let filter_complex = if crop_mode == "center_crop" {
        format!(
            "[0:v]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920{}[v]",
            sub_filter
        )
    } else {
        format!(
            "[0:v]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,boxblur=25:25[bg];[0:v]scale=1080:-1[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2{}[v]",
            sub_filter
        )
    };

    let mut cmd = Command::new(&ffmpeg_exe);
    cmd.args([
        "-y",
        "-i",
        input_path.to_str().unwrap(),
        "-filter_complex",
        &filter_complex,
        "-map",
        "[v]",
        "-map",
        "0:a?",
        "-c:v",
        "libx264",
        "-preset",
        "veryfast",
        "-crf",
        "22",
        "-c:a",
        "aac",
        "-b:a",
        "192k",
        "-threads",
        "2",
        "-r",
        "30",
        output_path.to_str().unwrap(),
    ]);

    let output = cmd.output().context("Failed to execute FFmpeg rendering")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("FFmpeg rendering failed: {}", err));
    }

    Ok(())
}
