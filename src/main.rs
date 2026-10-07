//cargo-llvm-cov-ignore:file

//--------------------------------------------------------------------------------------------------
// Crates

use {anyhow::Result, cargo_llvm_cov_ignore::*, clap_cargo::style::CLAP_STYLING};

//--------------------------------------------------------------------------------------------------
// Traits

use clap::Parser;

//--------------------------------------------------------------------------------------------------
// Enum

#[derive(Parser)]
#[command(
    name = "cargo",
    bin_name = "cargo",
    max_term_width = 80,
    styles = CLAP_STYLING,
)]
enum Args {
    #[command(about, version)]
    LlvmCovIgnore {
        #[arg(short = '0', hide = true)]
        exit_0: bool,
    },
}

//--------------------------------------------------------------------------------------------------
// Functions

fn main() -> Result<()> {
    let Args::LlvmCovIgnore { exit_0 } = Args::parse();

    run_cargo_llvm_cov()?;

    let uncovered_lines = get_uncovered_lines()?;

    let (ignored_files, ignored_lines) = get_ignored_files_and_lines()?;

    let (uncovered_lines, unused_ignored_lines, unused_ignored_files) =
        process(&uncovered_lines, &ignored_files, &ignored_lines);

    let (exit_code, report) = generate_report(
        &uncovered_lines,
        &unused_ignored_lines,
        &unused_ignored_files,
    )?;

    print!("{report}");

    if !exit_0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
