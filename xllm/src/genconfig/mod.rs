use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};
use serde::Deserialize;


pub fn resolve_env_variables(input: &str) -> String {
    let re = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
    re.replace_all(input, |caps: &regex::Captures| {
        std::env::var(&caps[1]).unwrap_or_else(|_| "".to_string())
    })
    .to_string()
}

pub fn get_config_path() -> Result<PathBuf> {
    // Try multiple locations in order of preference
    let possible_paths = vec![
        // 1. Current directory (for development)
        PathBuf::from("config.toml"),
        // 2. XDG config directory (~/.config/xllm/config.toml)
        dirs::config_dir()
            .map(|p| p.join("xllm").join("config.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.config/xllm/config.toml")),
        // 3. Home directory (~/.xllm.toml)
        dirs::home_dir()
            .map(|p| p.join(".xllm.toml"))
            .unwrap_or_else(|| PathBuf::from("~/.xllm.toml")),
    ];

    for path in possible_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    // If no config found, create the default config directory and provide helpful error
    let config_dir = dirs::config_dir()
        .map(|p| p.join("xllm"))
        .unwrap_or_else(|| PathBuf::from("~/.config/xllm"));

    Err(anyhow::anyhow!(
        "Configuration file not found. Please create one at:\n  {}\n\nExample config.toml:\n[models.claude]\nmodel = \"claude-sonnet-4-20250514\"\nmax_tokens = 1024\nanthropic_api_key = \"${{ANTHROPIC_API_KEY}}\"\nurl = \"https://api.anthropic.com/\"",
        config_dir.join("config.toml").display()
    ))
}

pub fn create_default_config() -> Result<()> {
    let config_dir = dirs::config_dir()
        .map(|p| p.join("xllm"))
        .unwrap_or_else(|| PathBuf::from("~/.config/xllm"));

    // Create config directory if it doesn't exist
    fs::create_dir_all(&config_dir).with_context(|| {
        format!(
            "Failed to create config directory: {}",
            config_dir.display()
        )
    })?;

    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        return Err(anyhow::anyhow!(
            "Config file already exists at {}",
            config_path.display()
        ));
    }

    let default_config = r#"[global]
proxy = false
proxy_url = "https://proxy.ai.url"

[models.claude]
model = "claude-sonnet-4-20250514"
max_tokens = 1024
anthropic_api_key = "${ANTHROPIC_API_KEY}"
url = "https://api.anthropic.com/"

# Future models can be added here, e.g.:
# [models.openai]
# model = "gpt-4"
# max_tokens = 1024
# api_key = "${OPENAI_API_KEY}"
# url = "https://api.openai.com/"
"#;

    fs::write(&config_path, default_config)
        .with_context(|| format!("Failed to write config file: {}", config_path.display()))?;

    println!("✅ Created default config at {}", config_path.display());
    println!("📝 Please set your ANTHROPIC_API_KEY environment variable or edit the config file.");

    Ok(())
}
pub fn load_config() -> Result<Config> {
    let config_path = get_config_path()?;

    let config_content = fs::read_to_string(&config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path.display()))?;

    let mut config: Config = toml::from_str(&config_content)
        .with_context(|| format!("Failed to parse config file: {}", config_path.display()))?;

    // Resolve environment variables for all models
    if let Some(claude_config) = &mut config.models.claude {
        claude_config.anthropic_api_key = resolve_env_variables(&claude_config.anthropic_api_key);
    }

    Ok(config)
}

/// Get the appropriate model configuration based on model name
pub fn get_model_config(config: &Config, model_name: &str) -> Result<ModelProvider> {
    match model_name {
        "opus4" | "sonnet4" | "sonnet3" | "haiku3" => {
            if let Some(claude_config) = &config.models.claude {
                Ok(ModelProvider::Claude(claude_config.clone()))
            } else {
                Err(anyhow::anyhow!("Claude configuration not found for model: {}", model_name))
            }
        }
        // Future models can be added here:
        // "gpt-4" | "gpt-3.5" => {
        //     if let Some(openai_config) = &config.models.openai {
        //         Ok(ModelProvider::OpenAI(openai_config.clone()))
        //     } else {
        //         Err(anyhow::anyhow!("OpenAI configuration not found for model: {}", model_name))
        //     }
        // }
        _ => Err(anyhow::anyhow!("Unknown model: {}. Supported models: opus4, sonnet4, sonnet3, haiku3", model_name))
    }
}

/// Enum to represent different model providers
#[derive(Debug, Clone)]
pub enum ModelProvider {
    Claude(ClaudeConfig),
    // Future providers:
    // OpenAI(OpenAIConfig),
}

// Generic Config struct that can hold configurations for multiple AI providers
#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: Option<GlobalConfig>,
    pub models: ModelsConfig,
}

#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    pub proxy: bool,
    pub proxy_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ModelsConfig {
    pub claude: Option<ClaudeConfig>,
    // Future models can be added here:
    // pub openai: Option<OpenAIConfig>,
    // pub anthropic: Option<AnthropicConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClaudeConfig {
    pub model: String,
    pub max_tokens: u32,
    pub url: String,
    pub anthropic_api_key: String,
}
