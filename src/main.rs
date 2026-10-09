//cargo-llvm-cov-ignore:file

//--------------------------------------------------------------------------------------------------
// Crates

use {
    anyhow::Result, cargo_llvm_cov_ignore::*, clap::ValueEnum, clap_cargo::style::CLAP_STYLING,
    std::io::IsTerminal,
};

//--------------------------------------------------------------------------------------------------
// Traits

use clap::Parser;

//--------------------------------------------------------------------------------------------------
// Enums

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
        /// Color output
        #[arg(
            long,
            value_enum,
            value_name = "auto|always|never",
            default_value = "auto",
            hide_default_value = true,
            hide_possible_values = true
        )]
        color: Color,

        #[arg(short = '0', hide = true)]
        exit_0: bool,
    },
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Color {
    #[default]
    Auto,
    Always,
    Never,
}

//--------------------------------------------------------------------------------------------------
// Functions

fn main() -> Result<()> {
    let Args::LlvmCovIgnore { exit_0, color } = Args::parse();

    let color = match color {
        Color::Auto => std::io::stdout().is_terminal(),
        Color::Always => true,
        Color::Never => false,
    };

    run_cargo_llvm_cov()?;

    let uncovered_lines = get_uncovered_lines()?;

    let (ignored_files, ignored_lines) = get_ignored_files_and_lines()?;

    let (uncovered_lines, unused_ignored_files, unused_ignored_lines) =
        process(&uncovered_lines, &ignored_files, &ignored_lines);

    let (exit_code, report) = generate_report(
        &uncovered_lines,
        &unused_ignored_files,
        &unused_ignored_lines,
        color,
    )?;

    print!("{report}");

    if !exit_0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
