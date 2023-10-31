//use clap::*;
use chatgpt::prelude::*;
use std::{env, io, fs, process};
use std::path::Path;


#[tokio::main]
async fn main() -> Result<()> {
    let exe_file = env::args().take(1).next().expect("Executable path not passed to args");
    
    let exe_file_path = Path::new(exe_file.as_str());
    let cwd_path = exe_file_path.parent().expect("Executable path is empty");
    let prompt_file_path = cwd_path.join("gita.prompt.txt");
    let prompt = fs::read_to_string(prompt_file_path).expect("Prompt file does not exist");
    
    let key = env::var("OPEN_AI_KEY").expect("OPEN_AI_KEY is not set");

    let client = ChatGPT::new_with_config(
        key,
        ModelConfigurationBuilder::default()
            .engine(ChatGPTEngine::Gpt35Turbo)
            .build()
            .unwrap())?;

    let args: Vec<String> = env::args().skip(1).collect();
    let message = format!("{prompt}{}", args.join(" "));

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
        println!("{command}");

        let parts: Vec<String> = winsplit::split(command);
        let command_exe = parts.first().expect("Command is empty");
        let mut process = process::Command::new(command_exe);

        process.args(parts.into_iter().skip(1)).stdout(process::Stdio::inherit()).stderr(process::Stdio::inherit());
        process.spawn()?.wait().expect("Failed to execute command");
    }

    Ok(())
}
