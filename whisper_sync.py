"""
Ultra-Accurate Whisper Word-Level Subtitle Generator.
Transcribes audio directly from the video slice using faster-whisper with frame-perfect timestamps.
"""
import os
import logging
from pathlib import Path
from typing import List, Dict, Any, Optional
from faster_whisper import WhisperModel
from config import WHISPER_MODEL

logger = logging.getLogger(__name__)

# Global cached model instance
_whisper_model = None

def get_whisper_model():
    global _whisper_model
    if _whisper_model is None:
        logger.info(f"Loading faster-whisper model ({WHISPER_MODEL})...")
        _whisper_model = WhisperModel(WHISPER_MODEL, device="cpu", compute_type="int8")
    return _whisper_model

def extract_word_timestamps_from_audio(
    video_or_audio_path: Path,
    language: Optional[str] = None
) -> List[Dict[str, Any]]:
    """
    Transcribes the media file directly and extracts exact millisecond word timestamps.
    """
    model = get_whisper_model()
    segments, info = model.transcribe(
        str(video_or_audio_path),
        language=language,
        word_timestamps=True,
        vad_filter=True,
        vad_parameters=dict(min_silence_duration_ms=300)
    )
    
    words = []
    for segment in segments:
        for w in segment.words:
            clean_w = w.word.strip()
            if clean_w:
                words.append({
                    "word": clean_w,
                    "start": float(w.start),
                    "end": float(w.end),
                    "probability": float(w.probability)
                })
    return words

def format_ass_time(seconds: float) -> str:
    """Converts seconds into ASS timestamp: H:MM:SS.cs"""
    hrs = int(seconds // 3600)
    mins = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    centis = int(round((seconds - int(seconds)) * 100))
    if centis >= 100:
        centis = 99
    return f"{hrs:d}:{mins:02d}:{secs:02d}.{centis:02d}"

def build_karaoke_ass(
    words: List[Dict[str, Any]],
    output_ass_path: Path,
    words_per_card: int = 3,
    header_tag: str = "",
    font_name: str = "Arial Black",
    font_size: int = 74,
    highlight_color_hex: str = "00E6FF",  # Yellow in BGR (&H0000E6FF)
    primary_color_hex: str = "FFFFFF",    # White in BGR (&H00FFFFFF)
    margin_v: int = 740
) -> Path:
    """
    Builds a high-impact, perfectly audio-synced Gen-Z / Alex Hormozi style subtitle file.
    Displays 2-3 uppercase words per card, highlighting the EXACT active word in real time.
    """
    # ASS format uses &HAABBGGRR
    primary_ass = f"&H00{primary_color_hex}&"
    highlight_ass = f"&H00{highlight_color_hex}&"

    ass_header = f"""[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: MainStyle,{font_name},{font_size},{primary_ass},&H000000FF,&H00000000,&H90000000,-1,0,0,0,100,100,1.5,0,1,6.5,2.5,2,40,40,{margin_v},1
Style: HeaderTag,{font_name},38,{highlight_ass},&H00FFFFFF,&H00000000,&H90000000,-1,0,0,0,100,100,1,0,1,4.0,1.5,8,40,40,320,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""
    events = []
    
    if header_tag:
        events.append(f"Dialogue: 0,0:00:00.00,1:00:00.00,HeaderTag,,0,0,0,,{header_tag}")

    # Group words into chunks of 2-3 words
    chunks = []
    for i in range(0, len(words), words_per_card):
        chunk = words[i:i + words_per_card]
        if chunk:
            chunks.append(chunk)

    # For each chunk, generate sequential events highlighting each word as it is spoken
    for chunk in chunks:
        chunk_words_text = [w["word"].upper().strip(",.!?\"'") for w in chunk]
        
        for active_idx, active_word in enumerate(chunk):
            w_start = active_word["start"]
            w_end = active_word["end"]
            
            # Next word start or current word end
            if active_idx < len(chunk) - 1:
                next_start = chunk[active_idx + 1]["start"]
                event_end = max(w_end, next_start)
            else:
                event_end = w_end + 0.15
                
            start_str = format_ass_time(w_start)
            end_str = format_ass_time(event_end)
            
            # Render line with active word highlighted
            styled_line_parts = []
            for idx, text in enumerate(chunk_words_text):
                if idx == active_idx:
                    styled_line_parts.append(r"{\c" + highlight_ass + r"}" + text + r"{\c" + primary_ass + r"}")
                else:
                    styled_line_parts.append(text)
                    
            line_text = " ".join(styled_line_parts)
            events.append(f"Dialogue: 1,{start_str},{end_str},MainStyle,,0,0,0,,{line_text}")

    full_ass = ass_header + "\n".join(events) + "\n"
    with open(output_ass_path, "w", encoding="utf-8") as f:
        f.write(full_ass)
        
    return output_ass_path
