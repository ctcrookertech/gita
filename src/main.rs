use chatgpt::prelude::*;
use std::{env, io, io::Read, fs, process};
use regex;

#[tokio::main]
async fn main() -> Result<()> {
    let exe_file_path = env::current_exe().expect("Executable path not found");
    let cwd_path = exe_file_path.parent().expect("Executable path is empty");
    let prompt_file_path = cwd_path.join("gita.prompt.txt");

    println!("Using prompt from {}", prompt_file_path.to_string_lossy());

    let prompt = fs::read_to_string(prompt_file_path).expect("Prompt file not read");
    let replace_pattern = regex::Regex::new(r"\|> (.*) <\|").unwrap();
    let mut updated_prompt = prompt.clone();

    for replace_capture in replace_pattern.captures_iter(&prompt) {
        let replace_text = replace_capture.get(0).expect("Capture not found").as_str();
        let replace_command = replace_capture.get(1).expect("Replace pattern value not found").as_str();
        let replace_command_output = run_command(replace_command, true).await?;
        let replaced_prompt = updated_prompt.replace(replace_text, &replace_command_output);

        updated_prompt.clear();
        updated_prompt.push_str(&replaced_prompt);
    }

    let key = env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY is not set");

    let client = ChatGPT::new_with_config(
        key,
        ModelConfigurationBuilder::default()
            .engine(ChatGPTEngine::Gpt35Turbo)
            .temperature(0.5)
            .build()
            .unwrap())?;

    let args: Vec<String> = env::args().skip(1).collect();
    let message = format!("{updated_prompt}{}", args.join(" "));

    println!("{}", message);

    let response = client.send_message(message).await?;
    let commands = response.message().content.to_string();

    println!("{}", commands);

    let mut confirmation = String::new();

    println!("Run all these commands now? [Y/n]");
    io::stdin().read_line(&mut confirmation).expect("Failed to read line");
    let confirmation = confirmation.trim();

    if confirmation != "" && confirmation != "y" && confirmation != "Y" {
        process::exit(-1);
    }

    let all_commands = commands.lines();

    for command in all_commands {
        run_command(command, false).await?;
    }

    Ok(())
}

async fn run_command(command: &str, capture: bool) -> Result<String> {
    println!("> {command}");

    let parts: Vec<String> = winsplit::split(command);
    let command_exe = parts.first().expect("Command is empty");
    let mut process = process::Command::new(command_exe);

    process.args(parts.into_iter().skip(1)).stderr(process::Stdio::inherit());

    if capture {
        process.stdout(process::Stdio::piped());
    } else {
        process.stdout(process::Stdio::inherit());
    }

    let mut child = process.spawn()?;
    let mut stdout_text = String::new(); 

    if capture {
        let stdout = child.stdout.as_mut().expect("Failed to open stdout");
        let mut stdout_buffer = Vec::new();
        stdout.read_to_end(&mut stdout_buffer)?;
        stdout_text = String::from_utf8(stdout_buffer).expect("Failed to convert stdout to String");
    }

    child.wait().expect("Failed to execute command");

    Ok(stdout_text)
}
