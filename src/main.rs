use dirs::config_dir;
use ollama_rs::{
    generation::completion::{request::GenerationRequest, GenerationResponseStream},
    Ollama,
};
use serde::{Deserialize, Serialize};
use std::{env, error::Error, fs, path::PathBuf};
use termimad::crossterm::style::Color::Magenta;
use termimad::crossterm::style::Color::Yellow;
use termimad::MadSkin;
use termimad::{crossterm::style::Attribute::Underlined, rgb};
use tokio::io::{self, AsyncWriteExt};
use tokio_stream::StreamExt;

/// Configuration structure for storing the selected model.
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    model: String,
}

/// Language model abstraction using Ollama.
struct OllamaLanguageModel {
    client: Ollama,
    model: String,
}

impl OllamaLanguageModel {
    /// Initializes a new instance of the language model with the provided configuration.
    fn new(config: Config) -> Result<Self, Box<dyn Error>> {
        let client = Ollama::default();
        Ok(Self {
            client,
            model: config.model,
        })
    }

    /// Generates a streamed response for the given prompt.
    async fn generate_stream(
        &self,
        prompt: &str,
    ) -> Result<GenerationResponseStream, Box<dyn Error>> {
        let request = GenerationRequest::new(self.model.clone(), prompt.to_string());
        let stream = self.client.generate_stream(request).await?;
        Ok(stream)
    }
}

async fn tell_command(config: Config, prompt: String) -> Result<(), Box<dyn Error>> {
    let language_model = OllamaLanguageModel::new(config)?;

    let mut stream = language_model.generate_stream(&prompt).await?;

    // Stream the responses as they arrive
    while let Some(response_result) = stream.next().await {
        match response_result {
            Ok(response) => {
                for part in response {
                    let s = &part.response;
                    let mut skin = MadSkin::default();
                    skin.bold.set_fg(Yellow);
                    skin.print_inline(s);
                    skin.paragraph.set_fgbg(Magenta, rgb(30, 30, 40));
                    skin.italic.add_attr(Underlined);
                    io::stdout().flush().await?;

                    if let Some(ctx) = part.context {
                        // Update context if provided
                        // This can be used to maintain conversation state
                        // For simplicity, we're not storing it here
                        let _context = ctx;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error during generation: {}", e);
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: tell <prompt> | tell --switch <model>");
        return Ok(());
    }

    let config_path = get_config_path()?;
    let mut config = load_config(&config_path)?;

    if args[1] == "--switch" && args.len() > 2 {
        config.model = args[2].clone();
        save_config(&config_path, &config)?;
        println!("Switched to model: {}", config.model);
        return Ok(()); // Exit after switching
    } else if args.len() > 1 {
        let prompt = args[1..].join(" ");
        tell_command(config, prompt).await?;
    }

    Ok(())
}

fn get_config_path() -> Result<PathBuf, Box<dyn Error>> {
    let config_dir = config_dir().ok_or("Failed to get config directory")?;
    let config_file = config_dir.join("tell.toml");
    Ok(config_file)
}

fn load_config(config_path: &PathBuf) -> Result<Config, Box<dyn Error>> {
    if !config_path.exists() {
        let default_config = Config {
            model: "gemma2:2b".to_string(),
        };
        save_config(config_path, &default_config)?;
        return Ok(default_config);
    }

    let config_string = fs::read_to_string(config_path)?;
    let config: Config = toml::from_str(&config_string)?;
    Ok(config)
}

fn save_config(config_path: &PathBuf, config: &Config) -> Result<(), Box<dyn Error>> {
    let config_string = toml::to_string(config)?;
    fs::write(config_path, config_string)?;
    Ok(())
}
