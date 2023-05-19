use snafu::prelude::*;

#[derive(Snafu, Debug)]
#[snafu(visibility(pub(crate)))]
pub enum ProcessError {
    #[snafu(display("failed reading /proc -> {source}"))]
    ProcError { source: std::io::Error },
}
