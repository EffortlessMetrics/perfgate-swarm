use anyhow::{Context, bail};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "noop" => Ok(()),
        "cpu-fixed" => cpu_fixed(),
        "io-fixed" => io_fixed(),
        "json-read" => json_read(args.get(2).map(|s| s.as_str())),
        "ci-compare-small" => ci_compare(
            ".ci/fixtures/compare/small-baseline.json",
            ".ci/fixtures/compare/small-current.json",
        ),
        "ci-compare-large" => ci_compare(
            ".ci/fixtures/compare/large-baseline.json",
            ".ci/fixtures/compare/large-current.json",
        ),
        "ci-check-single" => ci_check("test-bench"),
        "ci-check-no-baseline" => ci_check("test-no-baseline"),
        "ci-render-md" => ci_render("md", "comment.md"),
        "ci-render-report" => ci_render("report", "report.json"),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: perfgate-selfbench <command>");
    eprintln!(
        "Commands: noop, cpu-fixed, io-fixed, json-read, ci-compare-small, ci-compare-large, \
         ci-check-single, ci-check-no-baseline, ci-render-md, ci-render-report"
    );
}

fn cpu_fixed() -> anyhow::Result<()> {
    let start = Instant::now();
    let mut sum = 0u64;
    // Perform a fixed amount of CPU work
    for i in 0..10_000_000 {
        sum = sum.wrapping_add(i);
    }
    println!("CPU work complete: {}", sum);
    eprintln!("Duration: {:?}", start.elapsed());
    Ok(())
}

fn io_fixed() -> anyhow::Result<()> {
    let start = Instant::now();
    let tmp_dir = std::env::temp_dir();
    let path = tmp_dir.join("perfgate-selfbench-workload.bin");

    // Write 1MB of data
    let data = vec![0u8; 1024 * 1024];
    fs::write(&path, &data)?;

    // Read it back
    let read = fs::read(&path)?;
    assert_eq!(read.len(), data.len());

    // Clean up
    let _ = fs::remove_file(&path);

    eprintln!("IO work complete. Duration: {:?}", start.elapsed());
    Ok(())
}

fn json_read(path: Option<&str>) -> anyhow::Result<()> {
    let start = Instant::now();
    let content = if let Some(p) = path {
        fs::read_to_string(p)?
    } else {
        // Default small JSON
        r#"{"foo": "bar", "count": 123, "active": true}"#.to_string()
    };

    let _val: serde_json::Value = serde_json::from_str(&content)?;
    eprintln!("JSON work complete. Duration: {:?}", start.elapsed());
    Ok(())
}

fn ci_compare(baseline: &str, current: &str) -> anyhow::Result<()> {
    let perfgate = perfgate_bin()?;
    let out_dir = TempDir::create("perfgate-selfbench-compare")?;
    let out_file = out_dir.path().join("out.json");
    run_perfgate(
        &perfgate,
        [
            OsStr::new("compare"),
            OsStr::new("--baseline"),
            OsStr::new(baseline),
            OsStr::new("--current"),
            OsStr::new(current),
            OsStr::new("--out"),
            out_file.as_os_str(),
        ],
        true,
    )
}

fn ci_check(bench: &str) -> anyhow::Result<()> {
    let perfgate = perfgate_bin()?;
    let out_dir = TempDir::create("perfgate-selfbench-check")?;
    run_perfgate(
        &perfgate,
        [
            OsStr::new("check"),
            OsStr::new("--config"),
            OsStr::new(".ci/fixtures/check/perfgate.toml"),
            OsStr::new("--bench"),
            OsStr::new(bench),
            OsStr::new("--out-dir"),
            out_dir.path().as_os_str(),
        ],
        true,
    )
}

fn ci_render(command: &str, output: &str) -> anyhow::Result<()> {
    let perfgate = perfgate_bin()?;
    let out_dir = TempDir::create("perfgate-selfbench-render")?;
    let out_file = out_dir.path().join(output);
    run_perfgate(
        &perfgate,
        [
            OsStr::new(command),
            OsStr::new("--compare"),
            OsStr::new(".ci/fixtures/compare/compare-receipt.json"),
            OsStr::new("--out"),
            out_file.as_os_str(),
        ],
        false,
    )
}

fn run_perfgate<'a, I>(perfgate: &Path, args: I, allow_policy_exit: bool) -> anyhow::Result<()>
where
    I: IntoIterator<Item = &'a OsStr>,
{
    let status = Command::new(perfgate)
        .args(args)
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("running {}", perfgate.display()))?;

    match status.code() {
        Some(0) => Ok(()),
        Some(2 | 3) if allow_policy_exit => Ok(()),
        Some(code) => bail!("{} exited with status {}", perfgate.display(), code),
        None => bail!("{} was terminated by signal", perfgate.display()),
    }
}

fn perfgate_bin() -> anyhow::Result<PathBuf> {
    candidate_perfgate_bins()
        .into_iter()
        .find(|path| path.is_file())
        .context("perfgate binary not found; build it with `cargo build --release -p perfgate-cli --bin perfgate`")
}

fn candidate_perfgate_bins() -> Vec<PathBuf> {
    let binary_name = if cfg!(windows) {
        "perfgate.exe"
    } else {
        "perfgate"
    };
    let mut candidates = vec![PathBuf::from("target").join("release").join(binary_name)];

    if let Ok(current_exe) = env::current_exe()
        && let Some(parent) = current_exe.parent()
    {
        candidates.push(parent.join(binary_name));
    }

    candidates
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn create(prefix: &str) -> anyhow::Result<Self> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("system clock is before unix epoch")?
            .as_nanos();
        let path = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
        fs::create_dir(&path).with_context(|| format!("creating {}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
