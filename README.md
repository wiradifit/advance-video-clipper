# 🎬 Advance Video Clipper (AI-Powered Shorts & TikTok Engine)

> **High-performance, open-source AI video clipper** that converts long YouTube videos into viral TikToks, YouTube Shorts, and Instagram Reels. Powered by **2026 Viral Brain AI Analysis**, **Frame-Perfect Whisper Word Synchronization**, and **Alex Hormozi Gen-Z Animated Subtitles**.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Python: 3.10+](https://img.shields.io/badge/Python-3.10%2B-blue.svg)](https://www.python.org/)
[![FFmpeg](https://img.shields.io/badge/FFmpeg-6.0%2B-green.svg)](https://ffmpeg.org/)
[![Whisper](https://img.shields.io/badge/Whisper-faster--whisper-red.svg)](https://github.com/SYSTRAN/faster-whisper)

---

## 🌟 Key Highlights

* **🧠 2026 AI Virality Engine:** Analyzes video transcripts with deep LLM reasoning (OpenAI, DeepSeek, Groq, Ollama) evaluating 3-second hooks, high-arousal emotional triggers, controversy, and loopability.
* **🎙️ Frame-Perfect Audio Sync (`faster-whisper`):** Direct audio waveform speech recognition with Voice Activity Detection (VAD) producing millisecond-accurate start and end timestamps for every single spoken word.
* **⚡ Alex Hormozi / Gen-Z Subtitle Animation:** Rapid 2–3 word cards with real-time **Electric Yellow (`#FFE600`)** active-word karaoke glow positioned at chest level to avoid mobile UI obstructions.
* **📱 Full-Bleed 1080x1920 9:16 Vertical Video:** Clean center-cropped vertical framing with zero top/bottom blur bars or letterboxing.
* **⚡ Zero-Memory Overhead:** Ephemeral slice downloading with `yt-dlp` ensures minimal disk and RAM footprint (< 1.5 GB RAM required).
* **🌐 Universal LLM Provider Support:** Works with OpenAI (`gpt-4o-mini`), DeepSeek (`deepseek-chat`), Groq (`llama-3.3-70b-versatile`), OpenRouter, and local Ollama.

---

## 🏗️ Architecture & Pipeline

```
                      ┌────────────────────────────────────────┐
                      │    YouTube Video (URL or Video ID)     │
                      └───────────────────┬────────────────────┘
                                          │
                        1. Ingest Timestamped Transcript
                                          ▼
                      ┌────────────────────────────────────────┐
                      │    YouTube Transcript Ingestion API    │
                      └───────────────────┬────────────────────┘
                                          │ Group into 15s Context Blocks
                                          ▼
                      ┌────────────────────────────────────────┐
                      │  AI Virality Brain (LLM Evaluation)   │
                      │  • 3-Second Hook & Retention Scoring   │
                      │  • Virality Score (0-10) Ranking       │
                      │  • YouTube Titles & TikTok Captions    │
                      └───────────────────┬────────────────────┘
                                          │
                        2. Keyframe-Accurate Slicing (yt-dlp)
                                          ▼
                      ┌────────────────────────────────────────┐
                      │    High-Speed Video Slicer (yt-dlp)    │
                      └───────────────────┬────────────────────┘
                                          │
                        3. Direct Audio Waveform Speech-to-Text
                                          ▼
                      ┌────────────────────────────────────────┐
                      │    faster-whisper (VAD + Timestamps)   │
                      │    Exact ms start/end for each word    │
                      └───────────────────┬────────────────────┘
                                          │
                        4. Build Animated SubStation Alpha (.ass)
                                          ▼
                      ┌────────────────────────────────────────┐
                      │    Hormozi Active-Word Subtitle Engine │
                      │    2-3 Word Bursts + #FFE600 Pop       │
                      └───────────────────┬────────────────────┘
                                          │
                        5. Native FFmpeg 9:16 Center-Crop Render
                                          ▼
                      ┌────────────────────────────────────────┐
                      │  FFmpeg Engine (1080x1920 @ 30 FPS)    │
                      │  Full-Bleed Vertical + Burned Subtitles│
                      └───────────────────┬────────────────────┘
                                          │
                                          ▼
                      ┌────────────────────────────────────────┐
                      │  Output: Ready-to-Publish Viral Shorts │
                      └────────────────────────────────────────┘
```

---

## 💻 Tech Stack & System Requirements

### Technology Stack
* **Core Language:** Python 3.10+
* **AI & LLM Reasoning:** OpenAI Python SDK (compatible with OpenAI, DeepSeek, Groq, OpenRouter, Ollama)
* **Speech-to-Text:** `faster-whisper` (CTranslate2 INT8 engine) with Voice Activity Detection (VAD)
* **Video Rendering:** Native `ffmpeg` 6.0+
* **Stream Extraction:** `yt-dlp`
* **Subtitles:** Advanced SubStation Alpha (`.ass` v4.00+) with vector styling

### Minimum System Requirements
| Component | Minimum Specification | Recommended |
|---|---|---|
| **Operating System** | macOS 12+ or Linux (Ubuntu 20.04+) or Windows WSL2 | Ubuntu 24.04 LTS / macOS Sonoma+ |
| **CPU** | 2 vCPUs / 2 Cores | 4+ Cores (Apple Silicon M-Series, Intel i7, AMD Ryzen) |
| **RAM** | 1.5 GB Free RAM (Whisper `base` INT8 consumes ~150 MB) | 4 GB+ RAM |
| **Disk Space** | 1 GB Free Disk Space | 5 GB+ NVMe SSD |

---

## 🚀 Step-by-Step Installation

### 1. Clone the Repository
```bash
git clone https://github.com/wiradifit/advance-video-clipper.git
cd advance-video-clipper
```

### 2. Set Up Virtual Environment
```bash
python3 -m venv .venv
source .venv/bin/activate
```

### 3. Install FFmpeg
* **macOS (Homebrew):**
  ```bash
  brew install ffmpeg
  ```
* **Ubuntu / Debian Linux:**
  ```bash
  sudo apt-get update && sudo apt-get install -y ffmpeg
  ```

### 4. Install Python Dependencies
```bash
pip install --upgrade pip
pip install -r requirements.txt
```

### 5. Configure Environment Variables
Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

Add your LLM API Key:
```env
LLM_BASE_URL="https://api.openai.com/v1"
LLM_API_KEY="your-api-key-here"
LLM_MODEL="gpt-4o-mini"
WHISPER_MODEL="base"
OUTPUT_DIR="./output_clips"
```

---

## 📖 CLI Usage & Examples

### 1. Clip Top Viral Segments (Full-Screen 9:16)
```bash
# Automatically finds, scores, slices, and renders top 2 clips:
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --top 2
```

### 2. Choose Visual Subtitle Presets
```bash
# Hormozi Bold Yellow (Default):
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --preset hormozi

# Cyber Neon Cyan:
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --preset cyber_cyan

# TikTok Lime Pop:
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --preset neon_green
```

### 3. Inspect AI Virality Scorecard (Zero Video Download / Fast)
```bash
# Quick audit of all viral hooks without rendering video:
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --analyze-only
```

### 4. Custom Output Directory and Language
```bash
python cli.py --url "https://www.youtube.com/watch?v=VIDEO_ID" --top 3 --out-dir ./my_shorts --lang en
```

---

## 🤖 Autonomous AI Agent Setup Prompt

If you are using an AI coding assistant (Antigravity, Claude Code, Cursor, Codex, OpenClaw, Hermes) to set up and operate this repository autonomously, provide this prompt:

```text
You are an autonomous AI media engineer operating on this repository.

Tasks:
1. Environment: Initialize a Python virtual environment in `.venv` and run `pip install -r requirements.txt`.
2. System Checks: Verify that FFmpeg is installed and accessible via system PATH or `imageio-ffmpeg`.
3. Configuration: Ensure `.env` is configured with an OpenAI-compatible API key (OpenAI, DeepSeek, Groq, Ollama) for `LLM_API_KEY` and `LLM_MODEL`.
4. Execution: Run `python cli.py --url "<YOUTUBE_URL>" --top 2 --crop-mode center_crop`.
5. Quality Verification: Confirm that the rendered MP4 video in `output_clips/` has 1080x1920 vertical resolution, accurate Whisper waveform audio sync, and Alex Hormozi active-word karaoke subtitles.
```

---

## 🤝 Contributing
Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for details on opening issues, adding visual presets, or submitting pull requests.

---

## 📄 License
This project is open-sourced under the **[MIT License](LICENSE)**.
