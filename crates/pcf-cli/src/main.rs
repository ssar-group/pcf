fn main() {
    if let Err(error) = pcf_cli::run() {
        eprintln!("{error}");
        std::process::exit(1);
    } else {
        std::process::exit(0);
    }
}
