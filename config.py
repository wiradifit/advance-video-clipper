"""
Configuration for the Advance Video Clipper Engine.
Loads settings from environment variables or .env file.
"""
import os
from pathlib import Path
from dotenv import load_dotenv

# Base Paths
BASE_DIR = Path(__file__).resolve().parent
load_dotenv(BASE_DIR / ".env")

# Output Directory
OUTPUT_DIR = Path(os.getenv("OUTPUT_DIR", BASE_DIR / "output_clips"))
OUTPUT_DIR.mkdir(parents=True, exist_ok=True)

# LLM API Provider (OpenAI Compatible: OpenAI, DeepSeek, Groq, OpenRouter, Ollama, etc.)
LLM_BASE_URL = os.getenv("LLM_BASE_URL", os.getenv("OPENAI_BASE_URL", "https://api.openai.com/v1"))
LLM_API_KEY = os.getenv("LLM_API_KEY", os.getenv("OPENAI_API_KEY", ""))
LLM_MODEL = os.getenv("LLM_MODEL", os.getenv("OPENAI_MODEL", "gpt-4o-mini"))

# Video Specifications
OUTPUT_WIDTH = int(os.getenv("OUTPUT_WIDTH", "1080"))
OUTPUT_HEIGHT = int(os.getenv("OUTPUT_HEIGHT", "1920"))
FPS = int(os.getenv("FPS", "30"))
MAX_CLIP_DURATION_SEC = int(os.getenv("MAX_CLIP_DURATION_SEC", "60"))
MIN_CLIP_DURATION_SEC = int(os.getenv("MIN_CLIP_DURATION_SEC", "20"))
WHISPER_MODEL = os.getenv("WHISPER_MODEL", "base")
