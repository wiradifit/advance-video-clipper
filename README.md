# 🎬 Advance Video Clipper (`advance-clipper`)

A high-performance, open-source **native Rust CLI engine** that converts long-form podcasts and videos (up to 2+ hours) into viral, high-retention short-form clips (TikTok, YouTube Shorts, Instagram Reels).

Built for maximum speed, zero OOM panics, low memory footprint (<35 MB RAM), and millisecond-accurate single-word subtitle synchronization.

---

## 🔥 Features & Virality Standards

* **⚡ Ultra-Low Memory Footprint:** Built in native Rust using `tokio`, `reqwest`, and `clap`. Runs on low-spec VPS servers (under 35 MB RAM usage).
* **🧠 2026 AI Virality Brain:** Evaluates full timestamped transcripts using OpenAI / DeepSeek models to discover 30-45s viral goldmines based on hook strength, emotional arousal, curiosity gaps, and loop potential.
* **🚨 Top Single-Line Clickbait Header:** Generates an irresistible, ALL-CAPS single-line clickbait hook sentence with emojis rendered at the top of the video (`Y=240`) to maximize viewer CTR and curiosity.
* **💬 Strict 1-Word Subtitle Display:** Renders strictly **1 single word per card** (`WrapStyle: 2`, `Y=1200`) in **Electric Yellow (`#FFE600`)** synchronously with speech. Zero line wrapping, zero vertical stacking, zero visual fatigue.
* **📝 Viral Social Media Captions & Source Credit:** Automatically generates copy-paste ready social media captions optimized for viral distribution (Curiosity Hook + Comment-Trigger Question + 5-8 Hashtags) along with explicit YouTube video source attribution (`📌 Source: YouTube Video (https://youtu.be/...)`). Saved as sidecar `.txt` files alongside rendered `.mp4` shorts.
* **🛡️ Social Media Content Filter (250+ Words):** Built-in automatic masking for trigger & profanity words in **Bahasa Indonesia** (`MATI` → `M**I`, `DARAH` → `D***H`, `TUMBAL` → `T****L`, `BUNUH` → `B***H`) and **English** (`DEAD` → `D**D`, `KILL` → `K**L`, `FUCK` → `F**K`, `SUICIDE` → `S*****E`). Censoring shows only first & last letter, middle chars replaced with `*`. Sources: LDNOOBW, Google Profanity, drizki/indonesian-badwords, TikTok & YouTube content policies.
* **🎙️ `stable-ts` Indonesian Word Alignment:** Integrates OpenAI Whisper with Dynamic Time Warping (DTW) and VAD filtering for precise Indonesian speech recognition.
* **✂️ Byte-Range Stream Slicing:** Uses `yt-dlp` stream slicing (`--download-sections`) to download only the 35-second viral segment (~10 MB), bypassing 5+ GB full video downloads.
* **📐 Full-Bleed 1080x1920 Framing:** Center-crops videos to vertical 9:16 aspect ratio without heavy 2D blur padding.

---

## 🛡️ Social Media Trigger & Profanity Filter (250+ Words)

The engine automatically censors sensitive/triggering words across subtitles AND clickbait header titles.

**Masking Rule:** Only the **first and last** letter are visible. All middle characters are replaced with `*`.

> **Dictionary Sources:** [LDNOOBW](https://github.com/LDNOOBW/List-of-Dirty-Naughty-Obscene-and-Otherwise-Bad-Words), [google-profanity-words](https://github.com/coffee-and-fun/google-profanity-words), [indonesian-badwords](https://github.com/drizki/indonesian-badwords), [kasar](https://github.com/asruldev/kasar), TikTok Community Guidelines, YouTube Advertiser-Friendly Content Guidelines.

### 🇮🇩 Bahasa Indonesia Filtered Words

| Category | Examples (Raw → Censored) | Count |
|---|---|---|
| Violence & Death | `MATI` → `M**I`, `BUNUH` → `B***H`, `DARAH` → `D***H`, `MAYAT` → `M***T` | 26 |
| Occult / Superstition | `SANTET` → `S****T`, `SETAN` → `S***N`, `POCONG` → `P****G` | 13 |
| Profanity & Vulgar | `ANJING` → `A****G`, `BANGSAT` → `B*****T`, `KONTOL` → `K****L`, `GOBLOK` → `G****K` | 81 |
| Self-Harm & Drugs | `NARKOBA` → `N*****A`, `PERKOSA` → `P*****A`, `GANJA` → `G***A` | 8 |

### 🇬🇧 English Filtered Words

| Category | Examples (Raw → Censored) | Count |
|---|---|---|
| Violence & Death | `DEAD` → `D**D`, `KILL` → `K**L`, `MURDER` → `M****R`, `SUICIDE` → `S*****E` | 30 |
| Sexual / Adult | `FUCK` → `F**K`, `SHIT` → `S**T`, `BITCH` → `B***H`, `PORN` → `P**N` | 87 |
| Hate Speech / Slurs | `NIGGER` → `N****R`, `FAGGOT` → `F****T`, `RETARD` → `R****D` | 19 |
| Drugs / Self-Harm | `COCAINE` → `C*****E`, `HEROIN` → `H****N`, `OVERDOSE` → `O******E` | 17 |
| Weapons / Extremism | `TERRORIST` → `T*******T`, `BOMB` → `B**B`, `SHOOTING` → `S******G` | 9 |

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

# Run unit tests:
cargo test

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
