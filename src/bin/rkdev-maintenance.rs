use std::process::ExitCode;

fn main() -> ExitCode {
    match rkdev::maintenance::run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rkdev-maintenance: {error}");
            ExitCode::FAILURE
        }
    }
}
