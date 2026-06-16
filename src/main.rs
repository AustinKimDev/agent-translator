fn main() {
    let args: Vec<String> = std::env::args().collect();

    match agent_translator::run_cli(&args) {
        Ok(output) => print!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}
