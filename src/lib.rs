#![doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))]

//cargo-llvm-cov-ignore:file

//--------------------------------------------------------------------------------------------------
// Crates

use {
    anyhow::{Result, anyhow},
    ignore::Walk,
    std::{
        collections::{BTreeMap, BTreeSet},
        fmt::Write,
        fs::File,
        io::{BufRead, BufReader},
        path::PathBuf,
        process::Command,
    },
};

//--------------------------------------------------------------------------------------------------
// Constants

const IGNORE_FILE: &str = "//cargo-llvm-cov-ignore:file";
const IGNORE_LINE: &str = "//cargo-llvm-cov-ignore:line";

const CARGO_LLVM_COV_COMMAND: &str = "cargo llvm-cov --no-report";
const CARGO_LLVM_COV_REPORT_COMMAND: &str = "cargo llvm-cov report --show-missing-lines";

//--------------------------------------------------------------------------------------------------
// Types

type Files = BTreeSet<PathBuf>;
type Lines = BTreeSet<usize>;

type FileLines = BTreeMap<PathBuf, Lines>;

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
pub fn get_uncovered_lines() -> Result<FileLines> {
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
                            let mut n = Lines::new();

                            for line_number in line_numbers.split(", ") {
                                if let Some((a, b)) = line_number.split_once('-') {
                                    let a = a.parse::<usize>().unwrap();
                                    let b = b.parse::<usize>().unwrap();
                                    n.extend(a..=b);
                                } else {
                                    n.insert(line_number.parse::<usize>().unwrap());
                                }
                            }

                            PathBuf::from(file)
                                .strip_prefix(&current_dir)
                                .map(|path| (path.to_owned(), n))
                                .ok()
                        } else {
                            None
                        }
                    })
                    .collect())
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
pub fn get_ignored_files_and_lines() -> Result<(Files, FileLines)> {
    let mut ignored_files = Files::new();

    let ignored_lines = Walk::new(".")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_some_and(|ft| ft.is_file())
                && entry.path().extension().is_some_and(|ext| ext == "rs")
        })
        .filter_map(|entry| {
            let path = entry.path();
            let file = File::open(path).unwrap();
            let reader = BufReader::new(file);
            let mut lines = Lines::new();

            for (i, line) in reader.lines().enumerate() {
                if let Ok(line) = line {
                    if line.ends_with(IGNORE_FILE) {
                        ignored_files.insert(path.strip_prefix("./").unwrap().to_owned());
                        lines.clear();
                        break;
                    } else if line.ends_with(IGNORE_LINE) {
                        lines.insert(i + 1);
                    }
                }
            }

            if lines.is_empty() {
                None
            } else {
                Some((path.strip_prefix("./").unwrap().to_owned(), lines))
            }
        })
        .collect();

    Ok((ignored_files, ignored_lines))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Remove ignored files/lines from uncovered lines and identify unused ignored files/lines
pub fn process(
    uncovered_lines: &FileLines,
    ignored_files: &Files,
    ignored_lines: &FileLines,
) -> (FileLines, Files, FileLines) {
    let mut uncovered_lines = uncovered_lines.clone();

    // Remove ignored files

    let mut unused_ignored_files = Files::new();

    for file in ignored_files {
        if uncovered_lines.contains_key(file) {
            uncovered_lines.remove_entry(file);
        } else {
            unused_ignored_files.insert(file.clone());
        }
    }

    // Remove ignored lines

    let mut unused_ignored_lines = FileLines::new();

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
    uncovered_lines: &FileLines,
    unused_ignored_files: &Files,
    unused_ignored_lines: &FileLines,
) -> Result<(i32, String)> {
    let mut exit_code = 0;
    let mut report = String::new();

    writeln!(&mut report, "# Code Coverage")?;

    report_lines(
        &mut report,
        "Uncovered Lines",
        "uncovered line",
        uncovered_lines,
        &mut exit_code,
        1,
    )?;

    report_lines(
        &mut report,
        "Unused Ignored Lines",
        "unused ignored line",
        unused_ignored_lines,
        &mut exit_code,
        2,
    )?;

    report_files(
        &mut report,
        "Unused Ignored Files",
        "unused ignored file",
        unused_ignored_files,
        &mut exit_code,
        4,
    )?;

    Ok((exit_code, report))
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Write files and line number ranges
fn write_files_line_ranges(s: &mut String, lines: &FileLines) -> Result<()> {
    for (file, line_numbers) in lines {
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

/// Convert a list of individual line numbers into a list of ranges
fn ranges(numbers: &Lines) -> Vec<String> {
    fn push(r: &mut Vec<String>, start: usize, end: usize) {
        r.push(if start == end {
            format!("{start}")
        } else {
            format!("{start}-{end}")
        });
    }

    let mut r = vec![];

    let mut start = numbers.first().unwrap();
    let mut end = start;

    for n in numbers.iter().skip(1) {
        if *n == end + 1 {
            end = n;
        } else {
            push(&mut r, *start, *end);

            (start, end) = (n, n);
        }
    }

    push(&mut r, *start, *end);

    r
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Report lines
fn report_lines(
    s: &mut String,
    title: &str,
    desc: &str,
    lines: &FileLines,
    exit_code: &mut i32,
    lsb_val: i32,
) -> Result<()> {
    writeln!(s, "\n## {title}")?;

    let n = lines.len();

    if n == 0 {
        writeln!(s, "\nThere are zero files with {desc}s.")?;
    } else {
        if n == 1 {
            writeln!(s, "\nThere is 1 file with {desc}(s):")?;
        } else {
            writeln!(s, "\nThere are {n} files with {desc}s:")?;
        }

        write_files_line_ranges(s, lines)?;

        *exit_code |= lsb_val;
    }

    Ok(())
}

// - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - - -

/// Report files
fn report_files(
    s: &mut String,
    title: &str,
    desc: &str,
    files: &Files,
    exit_code: &mut i32,
    lsb_val: i32,
) -> Result<()> {
    writeln!(s, "\n## {title}")?;

    let n = files.len();

    if n == 0 {
        writeln!(s, "\nThere are zero {desc}s.")?;
    } else {
        if n == 1 {
            writeln!(s, "\nThere is 1 {desc}(s):")?;
        } else {
            writeln!(s, "\nThere are {n} {desc}s:")?;
        }

        for file in files {
            write!(s, "\n- `{}`", file.display())?;
        }
        writeln!(s)?;

        *exit_code |= lsb_val;
    }

    Ok(())
}
