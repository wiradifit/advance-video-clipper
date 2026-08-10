"""
Gen-Z Dynamic Subtitle Engine with AI Transcript Polishing and Multiple Style Presets.
"""
import json
import logging
from pathlib import Path
from typing import List, Dict, Any, Optional
from openai import OpenAI
from config import HCNSEC_BASE_URL, HCNSEC_API_KEY, HCNSEC_MODEL

logger = logging.getLogger(__name__)

# Style Presets for Gen-Z Short-Form Content
PRESETS = {
    "hormozi": {
        "name": "Hormozi Bold Yellow",
        "font": "Arial Black",
        "fontsize": 72,
        "primary_color": "&H00FFFFFF",      # Crisp White
        "highlight_color": "&H0000E6FF",    # Electric Yellow (&H00BBGGRR -> R=FF, G=E6, B=00)
        "outline_color": "&H00000000",      # Pure Black Outline
        "outline_width": 6.5,
        "shadow": 2.0,
        "margin_v": 720,                   # Chest/Mid focal zone (Y=1200)
        "alignment": 2,                    # Centered
        "uppercase": True,
        "words_per_line": 3
    },
    "cyber_cyan": {
        "name": "Cyber Neon Cyan",
        "font": "Trebuchet MS",
        "fontsize": 68,
        "primary_color": "&H00FFFFFF",
        "highlight_color": "&H00FFFF00",    # Vivid Cyan (&H00FFFF00 -> R=00, G=FF, B=FF)
        "outline_color": "&H00000000",
        "outline_width": 5.5,
        "shadow": 0.0,
        "margin_v": 700,
        "alignment": 2,
        "uppercase": True,
        "words_per_line": 3
    },
    "neon_green": {
        "name": "TikTok Lime Pop",
        "font": "Impact",
        "fontsize": 76,
        "primary_color": "&H00FFFFFF",
        "highlight_color": "&H0000FF66",    # Neon Lime Green (&H0066FF00)
        "outline_color": "&H00000000",
        "outline_width": 7.0,
        "shadow": 3.0,
        "margin_v": 740,
        "alignment": 2,
        "uppercase": True,
        "words_per_line": 2
    }
}

def polish_transcript_with_ai(
    raw_snippets: List[Any],
    clip_start_sec: float,
    clip_end_sec: float
) -> List[Dict[str, Any]]:
    """
    Uses hcnsec-custom (DeepSeek-V4-Flash) to clean up raw YouTube auto-captions,
    correct phonetic/Indonesian grammar errors, and chunk into punchy 2-4 word Gen-Z subtitle cards.
    """
    # Extract overlapping raw snippets
    relevant = []
    for s in raw_snippets:
        s_start = s.start if hasattr(s, 'start') else s.get('start', 0.0)
        s_dur = s.duration if hasattr(s, 'duration') else s.get('duration', 2.0)
        s_text = (s.text if hasattr(s, 'text') else s.get('text', '')).strip()
        s_end = s_start + s_dur
        
        if s_end > clip_start_sec and s_start < clip_end_sec:
            rel_start = max(0.0, s_start - clip_start_sec)
            rel_end = min(clip_end_sec - clip_start_sec, s_end - clip_start_sec)
            relevant.append({"start": round(rel_start, 2), "end": round(rel_end, 2), "text": s_text})
            
    if not relevant:
        return []

    client = OpenAI(base_url=HCNSEC_BASE_URL, api_key=HCNSEC_API_KEY)
    
    prompt = f"""
You are a professional Gen-Z subtitle editor.
Given these raw YouTube auto-generated speech snippets (which may have typos, phonetic misspellings, or weird word breaks):

Raw Snippets:
{json.dumps(relevant, ensure_ascii=False, indent=2)}

TASK:
1. Fix any Indonesian grammar mistakes, typo words, or misheard words.
2. Split into SHORT, PUNCHY subtitle cards (strictly 2 to 4 words per card).
3. Ensure relative start/end timestamps are perfectly sequential and fit between 0.00s and {round(clip_end_sec - clip_start_sec, 2)}s.
4. Mark 1 high-impact keyword per card to highlight.

Return a STRICT JSON array of objects:
[
  {{
    "start": 0.0,
    "end": 1.2,
    "text": "BANYAK ORANG BERPIKIR",
    "highlight_word": "BERPIKIR"
  }}
]
"""

    try:
        response = client.chat.completions.create(
            model=HCNSEC_MODEL,
            messages=[
                {"role": "system", "content": "You are a professional video subtitle editor. Output strictly JSON array."},
                {"role": "user", "content": prompt}
            ],
            response_format={"type": "json_object"},
            temperature=0.2
        )
        data = json.loads(response.choices[0].message.content)
        cards = data.get("subtitles", data.get("cards", data if isinstance(data, list) else []))
        if isinstance(data, dict) and not cards:
            for v in data.values():
                if isinstance(v, list):
                    cards = v
                    break
        return cards
    except Exception as e:
        logger.warning(f"AI transcript polish fallback due to: {e}")
        # Fallback to local chunker if LLM fails
        return fallback_chunker(relevant)

def fallback_chunker(snippets: List[Dict[str, Any]]) -> List[Dict[str, Any]]:
    cards = []
    for s in snippets:
        words = s["text"].split()
        if not words:
            continue
        dur = s["end"] - s["start"]
        step = dur / max(1, len(words))
        for i in range(0, len(words), 3):
            chunk = words[i:i+3]
            c_start = s["start"] + i * step
            c_end = min(s["end"], c_start + len(chunk) * step)
            cards.append({
                "start": round(c_start, 2),
                "end": round(c_end, 2),
                "text": " ".join(chunk).upper(),
                "highlight_word": chunk[0].upper()
            })
    return cards

def format_ass_time(seconds: float) -> str:
    """Converts seconds into ASS timestamp: H:MM:SS.cs"""
    hrs = int(seconds // 3600)
    mins = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    centis = int(round((seconds - int(seconds)) * 100))
    if centis >= 100:
        centis = 99
    return f"{hrs:d}:{mins:02d}:{secs:02d}.{centis:02d}"

def generate_genz_ass_file(
    cards: List[Dict[str, Any]],
    output_ass_path: Path,
    preset_name: str = "hormozi",
    header_tag: Optional[str] = None
) -> Path:
    """
    Generates a high-contrast Gen-Z style .ass subtitle file based on the chosen preset.
    """
    cfg = PRESETS.get(preset_name, PRESETS["hormozi"])
    
    ass_header = f"""[Script Info]
ScriptType: v4.00+
PlayResX: 1080
PlayResY: 1920
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: MainStyle,{cfg['font']},{cfg['fontsize']},{cfg['primary_color']},&H000000FF,{cfg['outline_color']},&H80000000,-1,0,0,0,100,100,1.5,0,1,{cfg['outline_width']},{cfg['shadow']},{cfg['alignment']},40,40,{cfg['margin_v']},1
Style: HeaderTag,Arial Black,38,&H0000FFFF,&H00FFFFFF,&H00000000,&H90000000,-1,0,0,0,100,100,1,0,1,4.0,1.5,8,40,40,320,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
"""
    events = []
    
    # Optional static header tag at top
    if header_tag:
        events.append(f"Dialogue: 0,0:00:00.00,1:00:00.00,HeaderTag,,0,0,0,,{header_tag}")

    for card in cards:
        start_sec = card.get("start", 0.0)
        end_sec = card.get("end", 1.0)
        raw_text = card.get("text", "").strip()
        highlight = card.get("highlight_word", "").strip().upper()
        
        if not raw_text or end_sec <= start_sec:
            continue
            
        start_str = format_ass_time(start_sec)
        end_str = format_ass_time(end_sec)
        
        # Upper case formatting
        text = raw_text.upper() if cfg["uppercase"] else raw_text
        
        # Word highlight coloring
        words = text.split()
        styled_words = []
        for w in words:
            # Clean word punctuation for comparison
            clean_w = "".join(ch for ch in w if ch.isalnum()).upper()
            if highlight and (clean_w in highlight or highlight in clean_w):
                styled_words.append(f"{{\\c{cfg['highlight_color']}&}}{w}{{\\c{cfg['primary_color']}&}}")
            else:
                styled_words.append(w)
                
        line_text = " ".join(styled_words)
        events.append(f"Dialogue: 1,{start_str},{end_str},MainStyle,,0,0,0,,{line_text}")

    full_ass = ass_header + "\n".join(events) + "\n"
    
    with open(output_ass_path, "w", encoding="utf-8") as f:
        f.write(full_ass)
        
    return output_ass_path
