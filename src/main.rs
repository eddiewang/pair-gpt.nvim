use clap::{Parser, Subcommand};
use serde_json::{from_str, json, Value};

#[derive(Parser, Debug, Clone)]
struct WriteArgs {
    #[arg(index = 1)]
    prompt: String,
}

#[derive(Subcommand, Debug, Clone)]
enum Action {
    Write(WriteArgs),
    Refactor(WriteArgs),
    Explain(WriteArgs),
    Walkthrough(WriteArgs),
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    action: Action,

    #[arg(short, long)]
    lang: String,

    #[arg(long, env = "OPENAI_API_KEY")]
    api_key: String,

    #[arg(short, long, default_value_t = String::from("text-davinci-001"))]
    model: String,

    #[arg(short = 't', long, default_value_t = 1024)]
    max_tokens: i32,

    #[arg(long, default_value_t = String::from("https://api.openai.com/v1/chat/completions"), env = "OPENAI_ENDPOINT")]
    endpoint: String,
}

pub fn into_comment(text: String, lang: String) -> anyhow::Result<String> {
    match lang.as_str() {
        "rust" | "c" | "javascript" | "typescript" | "solidity" => {
            let regex = regex::Regex::new(r"(?m)^")?;
            Ok(regex.replace_all(&text, "// ").to_string())
        }
        "dockerfile" | "bash" | "zsh" | "sh" | "python" | "ruby" => {
            let regex = regex::Regex::new(r"(?m)^")?;
            Ok(regex.replace_all(&text, "# ").to_string())
        }
        "lua" | "sql" => {
            let regex = regex::Regex::new(r"(?m)^")?;
            Ok(regex.replace_all(&text, "-- ").to_string())
        }
        _ => Ok(text),
    }
}

// similar to into_comment, but adds an extra new line for every line break
pub fn into_formatted(text: String) -> anyhow::Result<String> {
    let regex = regex::Regex::new(r"(?m)^")?;
    // add new line to the vim buffer and an extra breakline
    Ok(regex.replace_all(&text, "\n\r").to_string())
}


fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let prompt_msg = match args.action.clone() {
        Action::Write(WriteArgs { prompt }) => format!(
            "write {}, {}. Don't write explanations or anything else other than code",
            args.lang, prompt
        ),
        Action::Refactor(WriteArgs { prompt }) => {
            format!("refactor this {} code: ```\n{}```", args.lang, prompt)
        }
        Action::Explain(WriteArgs { prompt }) => {
            format!(
                "Don't start the sentence with as an AI language model. As an expert in {}, explain this {} code: ```\n{}```",
                args.lang, args.lang, prompt
            )
        }
        Action::Walkthrough(WriteArgs { prompt }) => {
            format!(
                "Don't start the sentence with as an AI language model. As an expert in {}, walkthrough indepth step by step with explanations on what this {} code does: ```\n{}```",
                args.lang, args.lang, prompt
            )
        }
    };

    let messages = vec![json!({"role": "user", "content": prompt_msg})];

    let body = json!({
        "model": "gpt-3.5-turbo",
        "messages": messages,
        "temperature": 0.8,
        // "max_tokens": args.max_tokens
    });

    let resp: String = ureq::post(&args.endpoint)
        .set("Authorization", format!("Bearer {}", args.api_key).as_str())
        .send_json(&body)?
        .into_string()?;

    let value: Value = from_str(&resp)?;
    let choice = &value["choices"][0]["message"]["content"];
    let mut code = snailquote::unescape(&choice.to_string())
        .unwrap()
        .trim()
        .to_string();

    if let Action::Explain(_) = args.action {
        let lines: Vec<&str> = code.split("\n").collect(); // Split the text into lines
        let mut wrapped_lines = Vec::new();
        for line in lines {
            // Wrap each line to 80 characters and push to the vector of wrapped lines
            wrapped_lines.extend(textwrap::wrap(line, 80));
        }
        code = wrapped_lines.join("\n"); // Join the wrapped lines with the newline character
    }
    if let Action::Walkthrough(_) = args.action {
        let lines: Vec<&str> = code.split("\n").collect(); // Split the text into lines
        let mut wrapped_lines = Vec::new();
        for line in lines {
            // Wrap each line to 80 characters and push to the vector of wrapped lines
            wrapped_lines.extend(textwrap::wrap(line, 80));
        }
        code = wrapped_lines.join("\n"); // Join the wrapped lines with the newline character
    }




    println!("{code}");

    Ok(())
}
