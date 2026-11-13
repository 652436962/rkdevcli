use std::process::ExitCode;

fn main() -> ExitCode {
    match rkdev::cli::run(std::env::args_os().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("rkdevcli: {error}");
            ExitCode::FAILURE
        }
    }
}
