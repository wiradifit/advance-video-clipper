"""
AI Virality Brain Module.
Evaluates long-form transcripts and scores segment virality using a high-arousal emotional framework.
"""
import json
import logging
from typing import Dict, Any, List
from openai import OpenAI
from config import LLM_BASE_URL, LLM_API_KEY, LLM_MODEL, MAX_CLIP_DURATION_SEC, MIN_CLIP_DURATION_SEC

logger = logging.getLogger(__name__)

SYSTEM_PROMPT = """You are an elite short-form video editor and viral content strategist for TikTok, YouTube Shorts, and Instagram Reels.

Your task is to analyze timestamped video transcripts and extract the most captivating, high-retention segments (20-60 seconds) that maximize watch time, shares, and comments.

### Virality Evaluation Framework:
1. **3-Second Hook Rule:** The opening line must stop users mid-scroll through controversy, curiosity gap, shocking revelation, or intense emotion.
2. **High-Arousal Triggers:** Prioritize content that evokes awe, righteous indignation, humor, curiosity, or unexpected truth.
3. **Stand-Alone Clarity:** The extracted segment must make complete sense without requiring the rest of the video.
4. **Loop Mechanics:** The segment should conclude with a punchy conclusion or mind-bending thought that seamlessly loops back to the start.

### JSON Output Schema:
Return ONLY a valid JSON object matching this exact structure:
{
  "analysis_summary": "Short overview of overall video content",
  "clips": [
    {
      "clip_id": "clip_1",
      "start_time": "MM:SS",
      "end_time": "MM:SS",
      "virality_score": 9.5,
      "category": "Controversial / Insight / Humor / Story",
      "hook_quote": "Exact punchy quote in the first 3 seconds",
      "youtube_title": "High CTR YouTube Shorts Title (Under 60 chars)",
      "tiktok_caption": "Engaging TikTok caption with 3-5 trending hashtags",
      "shareability_rationale": "Why this specific clip will generate shares and saves"
    }
  ]
}
"""

def get_openai_client() -> OpenAI:
    if not LLM_API_KEY:
        raise ValueError(
            "LLM API Key is missing. Please set LLM_API_KEY or OPENAI_API_KEY in your .env file or environment variables."
        )
    return OpenAI(base_url=LLM_BASE_URL, api_key=LLM_API_KEY)

def analyze_transcript_for_viral_clips(
    transcript_text: str,
    video_title: str = "Video",
    min_clips: int = 2,
    max_clips: int = 5
) -> Dict[str, Any]:
    """
    Sends the formatted timestamped transcript to the LLM to identify and rank top viral clips.
    """
    client = get_openai_client()
    
    user_prompt = f"""Video Title: {video_title}
Desired Number of Clips: Between {min_clips} and {max_clips}
Minimum Clip Duration: {MIN_CLIP_DURATION_SEC} seconds
Maximum Clip Duration: {MAX_CLIP_DURATION_SEC} seconds

Timestamped Transcript:
{transcript_text}

Extract the top viral clips that will perform best on TikTok and YouTube Shorts.
Return ONLY valid JSON matching the schema."""

    logger.info(f"Analyzing transcript virality with LLM ({LLM_MODEL})...")
    
    try:
        response = client.chat.completions.create(
            model=LLM_MODEL,
            messages=[
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": user_prompt}
            ],
            response_format={"type": "json_object"},
            temperature=0.4
        )
        
        content = response.choices[0].message.content
        data = json.loads(content)
        
        # Sort clips by virality_score descending
        if "clips" in data:
            data["clips"] = sorted(
                data["clips"],
                key=lambda x: x.get("virality_score", 0),
                reverse=True
            )
        return data
        
    except Exception as e:
        logger.error(f"Error during AI virality analysis: {e}")
        # Fallback JSON parser if markdown blocks exist
        if 'content' in locals() and content:
            cleaned = content.strip()
            if cleaned.startswith("```json"):
                cleaned = cleaned[7:]
            if cleaned.startswith("```"):
                cleaned = cleaned[3:]
            if cleaned.endswith("```"):
                cleaned = cleaned[:-3]
            try:
                return json.loads(cleaned.strip())
            except Exception:
                pass
        raise e
