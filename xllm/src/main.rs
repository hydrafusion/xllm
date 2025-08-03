// mod genconfig;
mod genconfig;
mod models;
mod utils;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use genconfig::{create_default_config, load_config, get_model_config, ModelProvider};
use indicatif::{ProgressBar, ProgressStyle};
use models::claude::call_claude_api;
use std::fs;
use utils::render::render_markdown;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("xllm")
        .version("1.0")
        .about("CLI tool for Claude API with markdown output")
        .arg(
            Arg::new("init")
                .long("init")
                .help("Create a default configuration file")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("prompt")
                .help("The prompt to send to Claude")
                .required_unless_present("init")
                .index(1),
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("Claude model to use: opus4, sonnet4, sonnet3, haiku3"),
        )
        .arg(
            Arg::new("max-tokens")
                .short('t')
                .long("max-tokens")
                .value_name("TOKENS")
                .help("Maximum tokens in response (overrides config default)")
                .value_parser(clap::value_parser!(u32)),
        )
        .arg(
            Arg::new("file")
                .long("file")
                .value_name("FILE")
                .help("File to include in the prompt"),
        )
        .get_matches();

    // Handle --init flag
    if matches.get_flag("init") {
        match create_default_config() {
            Ok(()) => return Ok(()),
            Err(e) => {
                eprintln!("❌ Failed to create config: {}", e);
                std::process::exit(1);
            }
        }
    }

    let prompt = matches.get_one::<String>("prompt").unwrap();
    let model_str = matches.get_one::<String>("model").map(|s| s.as_str());
    let max_tokens_override = matches.get_one::<u32>("max-tokens").copied();
    let file_path = matches.get_one::<String>("file");

    let model_override = models::claude::parse_model(model_str);

    // Build the final prompt
    let mut final_prompt = prompt.clone();

    if let Some(file_path) = file_path {
        let file_content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        final_prompt = format!("{}\n\nFile content:\n```\n{}\n```", prompt, file_content);
    }

    let config = load_config().context("Failed to load configuration")?;
    
    // Determine which model to use - either from command line or default
    let model_name = if let Some(model_str) = model_str {
        model_str
    } else {
        "sonnet4" // Default model
    };
    
    let model_provider = get_model_config(&config, model_name)
        .with_context(|| format!("Failed to get configuration for model: {}", model_name))?;
    
    let claude_config = match model_provider {
        ModelProvider::Claude(config) => config,
        // Future providers can be handled here
    };

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["🤖", "🔧", "⚙️", "🔩", "🤖", "⚡", "💻", "🧠"])
            .template("{spinner} {msg}")
            .unwrap(),
    );
    spinner.set_message("loading...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(200)); 
    
    // Check if proxy is enabled in config and warn user if not available
    if config.global.as_ref().map_or(false, |global| global.proxy) {
        eprintln!("⚠️  Warning: Proxy functionality is only available when building from source.");
        eprintln!("   Using direct Claude API instead.");
    }
    
    // Use direct Claude API (proxy functionality not available in published version)
    let result = call_claude_api(
        &claude_config,
        &final_prompt,
        model_override,
        max_tokens_override,
    ).await;

    match result {
        Ok(response) => {
            spinner.finish_and_clear();

            // Render the response as markdown
            render_markdown(&response);
        }
        Err(e) => {
            spinner.finish_and_clear();
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}
