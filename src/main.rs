fn main() {
    match treer::cmd::root::run() {
        Ok(()) => {}
        Err(err) => {
            eprintln!("Error: {err}");
            std::process::exit(1);
        }
    }
}
