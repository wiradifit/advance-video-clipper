# 🎬 Advance Video Clipper (`advance-clipper`)

A high-performance, open-source **native Rust CLI engine** that converts long-form podcasts and videos (up to 2+ hours) into viral, high-retention short-form clips (TikTok, YouTube Shorts, Instagram Reels).

Built for maximum speed, zero OOM panics, low memory footprint (<35 MB RAM), and millisecond-accurate single-word subtitle synchronization.

---

## 🔥 Features & Virality Standards

* **⚡ Ultra-Low Memory Footprint:** Built in native Rust using `tokio`, `reqwest`, and `clap`. Runs on low-spec VPS servers (under 35 MB RAM usage).
* **🧠 2026 AI Virality Brain:** Evaluates full timestamped transcripts using OpenAI / DeepSeek models to discover 30-45s viral goldmines based on hook strength, emotional arousal, curiosity gaps, and loop potential.
* **🚨 Top Single-Line Clickbait Header:** Generates an irresistible, ALL-CAPS single-line clickbait hook sentence with emojis rendered at the top of the video (`Y=240`) to maximize viewer CTR and curiosity.
* **💬 Strict 1-Word Subtitle Display:** Renders strictly **1 single word per card** (`WrapStyle: 2`, `Y=1200`) in **Electric Yellow (`#FFE600`)** synchronously with speech. Zero line wrapping, zero vertical stacking, zero visual fatigue.
* **🎙️ `stable-ts` Indonesian Word Alignment:** Integrates OpenAI Whisper with Dynamic Time Warping (DTW) and VAD filtering for precise Indonesian speech recognition.
* **✂️ Byte-Range Stream Slicing:** Uses `yt-dlp` stream slicing (`--download-sections`) to download only the 35-second viral segment (~10 MB), bypassing 5+ GB full video downloads.
* **📐 Full-Bleed 1080x1920 Framing:** Center-crops videos to vertical 9:16 aspect ratio without heavy 2D blur padding.

---

## 📊 Virality Best Practice Duration Standards

Short-form algorithms (TikTok, Shorts, Reels) rank videos based on **watch-through completion rate**. 
* **Optimal Short Duration:** **30 to 45 seconds** (Sweet spot: ~35 seconds).
* Clips under 30s lack depth for shares, while clips over 60s suffer algorithm drop-off. The 30-45s window yields optimal 85%+ completion rates and repeat loops.

---

## 🚀 Quick Start

### 1. Requirements
* **Rust Toolchain:** `rustc 1.80+` & `cargo`
* **FFmpeg:** Installed on system PATH or via `imageio-ffmpeg`
* **Python 3.10+:** For `faster-whisper` and `stable-ts` word timestamp extraction

### 2. Installation
```bash
git clone https://github.com/wiradifit/advance-video-clipper.git
cd advance-video-clipper

# Install Python STT helper dependencies:
pip install -r requirements.txt

# Build optimized release binary:
cargo build --release
```

### 3. Environment Setup
Copy `.env.example` to `.env` and provide your API keys:
```env
LLM_BASE_URL=https://api.openai.com/v1
LLM_API_KEY=your_api_key_here
LLM_MODEL=gpt-4o-mini
WHISPER_MODEL=base
OUTPUT_DIR=./output_clips
```

---

## 💻 CLI Usage

```bash
# Render top 2 viral shorts from any video (full-bleed 1080x1920 center-crop):
./target/release/advance-clipper --url "https://www.youtube.com/watch?v=VIDEO_ID" --top 2

# Inspect AI virality scorecard without rendering video:
./target/release/advance-clipper --url "https://www.youtube.com/watch?v=VIDEO_ID" --analyze-only

# Custom subtitle preset & language selection:
./target/release/advance-clipper --url "https://www.youtube.com/watch?v=VIDEO_ID" --preset hormozi --lang id
```

---

## 📜 License

Distributed under the MIT License. See [`LICENSE`](LICENSE) for details.
