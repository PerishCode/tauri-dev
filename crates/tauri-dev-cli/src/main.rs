fn main() {
    if let Err(error) = tauri_dev_cli::run(std::env::args().collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
