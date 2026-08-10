use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub llm_base_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
    pub whisper_model: String,
    pub output_dir: PathBuf,
    pub output_width: u32,
    pub output_height: u32,
    pub fps: u32,
    pub max_clip_duration: u32,
    pub min_clip_duration: u32,
}

impl Config {
    pub fn load() -> Self {
        // Attempt loading .env
        let _ = dotenvy::dotenv();

        let llm_base_url = env::var("LLM_BASE_URL")
            .or_else(|_| env::var("OPENAI_BASE_URL"))
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        let llm_api_key = env::var("LLM_API_KEY")
            .or_else(|_| env::var("OPENAI_API_KEY"))
            .or_else(|_| env::var("HCNSEC_API_KEY"))
            .unwrap_or_default();

        let llm_model = env::var("LLM_MODEL")
            .or_else(|_| env::var("OPENAI_MODEL"))
            .or_else(|_| env::var("HCNSEC_MODEL"))
            .unwrap_or_else(|_| "gpt-4o-mini".to_string());

        let whisper_model = env::var("WHISPER_MODEL").unwrap_or_else(|_| "base".to_string());
        
        let output_dir = env::var("OUTPUT_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("./output_clips"));

        Self {
            llm_base_url,
            llm_api_key,
            llm_model,
            whisper_model,
            output_dir,
            output_width: 1080,
            output_height: 1920,
            fps: 30,
            max_clip_duration: 45, // Best-practice viral short-form duration (30-45s sweet spot)
            min_clip_duration: 30,
        }
    }
}
