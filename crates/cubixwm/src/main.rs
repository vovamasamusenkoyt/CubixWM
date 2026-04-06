use cubixwm::Application;
use cubixwm::utils::Error;
use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    match try_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("cubixwm: {error}");
            ExitCode::FAILURE
        }
    }
}

fn try_main() -> Result<(), Error> {
    let command = env::args().nth(1).unwrap_or_else(|| "run".to_string());
    let mut app = Application::new();

    match command.as_str() {
        "run" => app.run(),
        "tty" => app.run_tty(),
        "demo" => {
            app.demo();
            Ok(())
        }
        "help" | "--help" | "-h" => {
            print_help();
            Ok(())
        }
        other => Err(format!("unknown command `{other}`; try `run`, `demo` or `help`").into()),
    }
}

fn print_help() {
    println!("cubixwm");
    println!();
    println!("Commands:");
    println!("  run   Start the application loop");
    println!("  tty   Start the tty/drm backend (build with --features tty-backend)");
    println!("  demo  Exercise the WM core with mock windows");
    println!("  help  Show this help");
}
