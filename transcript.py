"""
YouTube Transcript Ingestion and Formatting Module.
"""
import re
import logging
from typing import List, Dict, Any
from youtube_transcript_api import YouTubeTranscriptApi, TranscriptsDisabled, NoTranscriptFound

logger = logging.getLogger(__name__)

def extract_video_id(url_or_id: str) -> str:
    """
    Extracts 11-character YouTube video ID from various URL formats.
    """
    if len(url_or_id) == 11 and not ("/" in url_or_id or "?" in url_or_id):
        return url_or_id
        
    patterns = [
        r'(?:v=|\/)([0-9A-Za-z_-]{11}).*',
        r'(?:youtu\.be\/)([0-9A-Za-z_-]{11})',
        r'(?:embed\/)([0-9A-Za-z_-]{11})',
        r'(?:shorts\/)([0-9A-Za-z_-]{11})'
    ]
    for pattern in patterns:
        match = re.search(pattern, url_or_id)
        if match:
            return match.group(1)
            
    raise ValueError(f"Could not extract a valid YouTube video ID from: {url_or_id}")

def fetch_transcript(video_id: str, preferred_languages: List[str] = None) -> List[Dict[str, Any]]:
    """
    Fetches transcript snippets from YouTube for the given video ID.
    Attempts preferred languages first, then falls back to any available or auto-generated transcript.
    """
    if preferred_languages is None:
        preferred_languages = ['en', 'id', 'es', 'pt', 'de', 'fr', 'ja', 'ko']
        
    try:
        transcript_list = YouTubeTranscriptApi.list_transcripts(video_id)
        
        # Try manual transcripts
        for lang in preferred_languages:
            try:
                transcript = transcript_list.find_manually_created_transcript([lang])
                return transcript.fetch()
            except Exception:
                pass
                
        # Try generated transcripts
        for lang in preferred_languages:
            try:
                transcript = transcript_list.find_generated_transcript([lang])
                return transcript.fetch()
            except Exception:
                pass
                
        # Fallback to the first available transcript
        for transcript in transcript_list:
            return transcript.fetch()
            
    except (TranscriptsDisabled, NoTranscriptFound) as e:
        logger.error(f"No transcript available for video {video_id}: {e}")
        raise e
    except Exception as e:
        logger.error(f"Error fetching transcript for {video_id}: {e}")
        raise e
        
    raise RuntimeError(f"Unable to retrieve transcript for video ID: {video_id}")

def seconds_to_timestamp(seconds: float) -> str:
    """Converts seconds into MM:SS or HH:MM:SS format."""
    hrs = int(seconds // 3600)
    mins = int((seconds % 3600) // 60)
    secs = int(seconds % 60)
    if hrs > 0:
        return f"{hrs:02d}:{mins:02d}:{secs:02d}"
    return f"{mins:02d}:{secs:02d}"

def timestamp_to_seconds(ts_str: str) -> float:
    """Converts MM:SS or HH:MM:SS format into seconds."""
    parts = list(map(float, ts_str.strip().split(':')))
    if len(parts) == 3:
        return parts[0] * 3600 + parts[1] * 60 + parts[2]
    elif len(parts) == 2:
        return parts[0] * 60 + parts[1]
    elif len(parts) == 1:
        return parts[0]
    return 0.0

def build_llm_transcript_text(snippets: List[Dict[str, Any]], group_interval_sec: float = 15.0) -> str:
    """
    Aggregates granular transcript snippets into timestamped blocks for the LLM.
    """
    blocks = []
    current_block_start = 0.0
    current_block_text = []
    
    for s in snippets:
        start = s.get('start', 0.0)
        text = s.get('text', '').replace('\n', ' ').strip()
        if not text:
            continue
            
        if not current_block_text:
            current_block_start = start
            
        if start - current_block_start >= group_interval_sec:
            ts_label = seconds_to_timestamp(current_block_start)
            blocks.append(f"[{ts_label}] {' '.join(current_block_text)}")
            current_block_start = start
            current_block_text = [text]
        else:
            current_block_text.append(text)
            
    if current_block_text:
        ts_label = seconds_to_timestamp(current_block_start)
        blocks.append(f"[{ts_label}] {' '.join(current_block_text)}")
        
    return "\n".join(blocks)
