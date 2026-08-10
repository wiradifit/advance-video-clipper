# Contributing to Advance Video Clipper

Thank you for your interest in contributing to **Advance Video Clipper**! We welcome bug reports, feature requests, code improvements, and new visual presets.

---

## 🛠️ Development Setup

1. **Fork and Clone the Repository:**
   ```bash
   git clone https://github.com/your-username/advance-video-clipper.git
   cd advance-video-clipper
   ```

2. **Set up Virtual Environment:**
   ```bash
   python3 -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

3. **Configure Environment:**
   ```bash
   cp .env.example .env
   # Add your LLM API Key (OpenAI, DeepSeek, Groq, Ollama)
   ```

4. **Run a Test Clip:**
   ```bash
   python cli.py --url "https://www.youtube.com/watch?v=RS077KUy93g" --top 1
   ```

---

## 🚀 How to Contribute

1. **Create a Feature Branch:**
   ```bash
   git checkout -b feature/awesome-new-feature
   ```
2. **Commit Your Changes:**
   ```bash
   git commit -m "feat: add support for dynamic emoji subtitle animations"
   ```
3. **Push to Your Fork:**
   ```bash
   git push origin feature/awesome-new-feature
   ```
4. **Open a Pull Request** with a clear explanation of what was changed and why.

---

## 💡 Contribution Ideas
* Support for additional subtitle styles (MrBeast fonts, glowing drop shadows, word bouncing animations).
* Smart face detection auto-reframe for multi-speaker podcasts.
* Direct upload integration with YouTube Data API and TikTok Webhook API.
