mod claude;
use anyhow::{Context, Result};
use clap::{Arg, Command};
use claude::{call_claude_api, load_config};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use termimad::crossterm::style::Color::*;
use termimad::*;

fn render_markdown(text: &str) {
    let mut skin = MadSkin::default();

    skin.set_headers_fg(Yellow);
    skin.bold.set_fg(Cyan);
    skin.italic.set_fg(Magenta);
    skin.inline_code.set_fgbg(Green, AnsiValue(236));
    skin.code_block.set_fgbg(White, AnsiValue(235));
    skin.table.align = Alignment::Left;

    // Create area
    let mut area = Area::full_screen();
    area.pad_for_max_width(100);

    let formatted_text = skin.area_text(text, &area);
    print!("{}", formatted_text);
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("xllm")
        .version("1.0")
        .about("CLI tool for Claude API with markdown output")
        .arg(
            Arg::new("prompt")
                .help("The prompt to send to Claude")
                .required(true)
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

    let prompt = matches.get_one::<String>("prompt").unwrap();
    let model_str = matches.get_one::<String>("model").map(|s| s.as_str());
    let max_tokens_override = matches.get_one::<u32>("max-tokens").copied();
    let file_path = matches.get_one::<String>("file");

    let model_override = claude::parse_model(model_str);

    // Build the final prompt
    let mut final_prompt = prompt.clone();

    if let Some(file_path) = file_path {
        let file_content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path))?;

        final_prompt = format!("{}\n\nFile content:\n```\n{}\n```", prompt, file_content);
    }

    let config = load_config().context("Failed to load configuration")?;

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["🤖", "🔧", "⚙️", "🔩", "🤖", "⚡", "💻", "🧠"])
            .template("{spinner} {msg}")
            .unwrap(),
    );
    spinner.set_message("loading...");
    spinner.enable_steady_tick(std::time::Duration::from_millis(200)); // Call Claude API
    match call_claude_api(
        &config.claude,
        &final_prompt,
        model_override,
        max_tokens_override,
    )
    .await
    {
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
