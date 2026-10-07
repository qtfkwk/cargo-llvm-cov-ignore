#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

//cargo-llvm-cov-ignore:file

//--------------------------------------------------------------------------------------------------
// Crates

use {
    anyhow::{Result, anyhow},
    std::{
        collections::{BTreeMap, BTreeSet},
        fmt::Write,
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
        process::Command,
    },
    walkdir::WalkDir,
};

//--------------------------------------------------------------------------------------------------
// Constants

const IGNORE_FILE: &str = "//cargo-llvm-cov-ignore:file";
const IGNORE_LINE: &str = "//cargo-llvm-cov-ignore:line";
const CARGO_LLVM_COV_COMMAND: &str = "cargo llvm-cov --no-report";
const CARGO_LLVM_COV_REPORT_COMMAND: &str = "cargo llvm-cov report --show-missing-lines";

//--------------------------------------------------------------------------------------------------
// Types

type IgnoredFilesAndLines = (BTreeSet<PathBuf>, BTreeMap<PathBuf, BTreeSet<usize>>);

type Results = (
    BTreeMap<PathBuf, BTreeSet<usize>>,
    BTreeSet<PathBuf>,
    BTreeMap<PathBuf, BTreeSet<usize>>,
);

//--------------------------------------------------------------------------------------------------
// Functions

/// Run `cargo llvm-cov --no-report`
pub fn run_cargo_llvm_cov() -> Result<()> {
    match Command::new("cargo")
        .args(["llvm-cov", "--no-report"])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                return Err(anyhow!(
                    "Command `{CARGO_LLVM_COV_COMMAND}` failed: {}",
                    output.status,
                ));
            }
        }
        Err(e) => {
            return Err(anyhow!("Command `{CARGO_LLVM_COV_COMMAND}` failed: {e}"));
        }
    }

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Get uncovered lines
pub fn get_uncovered_lines() -> Result<BTreeMap<PathBuf, BTreeSet<usize>>> {
    let current_dir = std::env::current_dir()?;

    match Command::new("cargo")
        .args(["llvm-cov", "report", "--show-missing-lines"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                Ok(stdout
                    .lines()
                    .skip_while(|line| !line.starts_with("Uncovered Lines:"))
                    .filter_map(|line| {
                        if let Some((file, line_numbers)) = line.split_once(": ") {
                            let mut n = BTreeSet::new();

                            for line_number in line_numbers.split(", ") {
                                if let Some((a, b)) = line_number.split_once('-') {
                                    let a = a.parse::<usize>().unwrap();
                                    let b = b.parse::<usize>().unwrap();
                                    n.extend(a..=b);
                                } else {
                                    n.insert(line_number.parse::<usize>().unwrap());
                                }
                            }

                            if let Ok(path) = PathBuf::from(file).strip_prefix(&current_dir) {
                                Some((path.to_owned(), n))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeMap<_, _>>())
            } else {
                Err(anyhow!(
                    "Command `{CARGO_LLVM_COV_REPORT_COMMAND}` failed: {}",
                    output.status,
                ))
            }
        }
        Err(e) => Err(anyhow!(
            "Command `{CARGO_LLVM_COV_REPORT_COMMAND}` failed: {e}",
        )),
    }
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Get ignored files and lines
pub fn get_ignored_files_and_lines() -> Result<IgnoredFilesAndLines> {
    let mut ignored_files = BTreeSet::new();

    let ignored_lines = WalkDir::new(".")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_owned())
        .filter(|path| {
            if let Some(ext) = path.extension() {
                ext == "rs"
            } else {
                false
            }
        })
        .filter_map(|path| {
            let file = File::open(&path).unwrap();
            let reader = BufReader::new(file);
            let ignored = reader
                .lines()
                .enumerate()
                .filter_map(|(i, line)| {
                    if let Ok(line) = line {
                        if line == IGNORE_FILE {
                            ignored_files.insert(path.strip_prefix("./").unwrap().to_owned());

                            None
                        } else {
                            line.ends_with(IGNORE_LINE).then_some(i + 1)
                        }
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>();

            (!ignored.is_empty()).then_some((path.strip_prefix("./").unwrap().to_owned(), ignored))
        })
        .collect::<BTreeMap<_, _>>();

    Ok((ignored_files, ignored_lines))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Process uncovered lines to remove ignored files/lines and identify unused ignored files/lines
pub fn process(
    uncovered_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
    ignored_files: &BTreeSet<PathBuf>,
    ignored_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
) -> Results {
    let mut uncovered_lines = uncovered_lines.clone();

    // Remove ignored files

    let mut unused_ignored_files = BTreeSet::new();

    for file in ignored_files {
        if uncovered_lines.contains_key(file) {
            uncovered_lines.remove_entry(file);
        } else {
            unused_ignored_files.insert(file.clone());
        }
    }

    // Remove ignored lines

    let mut unused_ignored_lines = BTreeMap::new();

    for (file, ignored) in ignored_lines {
        if let Some(uncovered) = uncovered_lines.get_mut(file) {
            let unused = ignored - uncovered;

            uncovered.retain(|x| !ignored.contains(x));

            if !unused.is_empty() {
                unused_ignored_lines.insert(file.clone(), unused);
            }
        }
    }

    // Remove files with zero uncovered lines

    let remove = uncovered_lines
        .iter()
        .filter(|(_file, lines)| lines.is_empty())
        .map(|(file, _lines)| file.clone())
        .collect::<Vec<_>>();

    for file in remove {
        uncovered_lines.remove(&file);
    }

    (uncovered_lines, unused_ignored_files, unused_ignored_lines)
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Generate report
pub fn generate_report(
    uncovered_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
    unused_ignored_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
    unused_ignored_files: &BTreeSet<PathBuf>,
) -> Result<(i32, String)> {
    let mut exit_code = 0;
    let mut report = String::new();

    writeln!(&mut report, "# Code Coverage")?;

    writeln!(&mut report, "\n## Uncovered Lines")?;
    let n = uncovered_lines.len();
    if n == 0 {
        writeln!(&mut report, "\nThere are zero files with uncovered lines.")?;
    } else {
        if n == 1 {
            writeln!(&mut report, "\nThere is 1 file with uncovered line(s):")?;
        } else {
            writeln!(&mut report, "\nThere are {n} files with uncovered lines:")?;
        }

        write_files_line_ranges(&mut report, uncovered_lines)?;

        exit_code |= 1;
    }

    writeln!(&mut report, "\n## Unused Ignored Lines")?;
    let n = unused_ignored_lines.len();
    if n == 0 {
        writeln!(
            &mut report,
            "\nThere are zero files with unused ignored lines."
        )?;
    } else {
        if n == 1 {
            writeln!(
                &mut report,
                "\nThere is 1 file with unused ignored line(s):"
            )?;
        } else {
            writeln!(
                &mut report,
                "\nThere are {n} files with unused ignored lines:"
            )?;
        }

        write_files_line_ranges(&mut report, unused_ignored_lines)?;

        exit_code |= 2;
    }

    writeln!(&mut report, "\n## Unused Ignored Files")?;
    let n = unused_ignored_files.len();
    if n == 0 {
        writeln!(&mut report, "\nThere are zero unused ignored files.")?;
    } else {
        if n == 1 {
            writeln!(&mut report, "\nThere is 1 unused ignored file(s):")?;
        } else {
            writeln!(&mut report, "\nThere are {n} unused ignored files:")?;
        }

        for file in unused_ignored_files {
            write!(&mut report, "\n- `{}`", file.display())?;
        }
        writeln!(&mut report)?;

        exit_code |= 4;
    }

    Ok((exit_code, report))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Write files and line number ranges
fn write_files_line_ranges(
    s: &mut String,
    files_lines: &BTreeMap<PathBuf, BTreeSet<usize>>,
) -> Result<()> {
    for (file, line_numbers) in files_lines {
        let total = line_numbers.len();
        if total > 0 {
            let r = ranges(line_numbers);
            let num_ranges = r.len();

            writeln!(
                s,
                "\n- `{}`: {total} {}, {num_ranges} {}:\n",
                file.display(),
                if total == 1 { "line" } else { "lines" },
                if num_ranges == 1 { "range" } else { "ranges" },
            )?;

            for (i, range) in r.iter().enumerate() {
                writeln!(s, "  {}. {range}", i + 1)?;
            }
        }
    }

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Convert a list of individual numbers into a list of ranges
fn ranges(numbers: &BTreeSet<usize>) -> Vec<String> {
    let mut r = vec![];
    let mut start = numbers.first().unwrap();
    let mut prev = start;
    for n in numbers.iter().skip(1) {
        if *n == prev + 1 {
            prev = n;
        } else {
            r.push(if start == prev {
                format!("{start}")
            } else {
                format!("{start}-{prev}")
            });
            (start, prev) = (n, n);
        }
    }
    r.push(if start == prev {
        format!("{start}")
    } else {
        format!("{start}-{prev}")
    });
    r
}
