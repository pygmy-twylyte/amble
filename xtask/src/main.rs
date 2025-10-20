use std::{
    ffi::OsStr,
    fs,
    fs::File,
    io,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::MetadataCommand;
use clap::{Args, Parser, Subcommand, ValueEnum};
use walkdir::WalkDir;
use zip::{CompressionMethod, ZipWriter, write::FileOptions};

#[derive(Parser)]
#[command(author, version, about = "Project automation tasks for Amble.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the Amble engine with optional DEV_MODE support.
    BuildEngine(BuildEngineArgs),
    /// Packaging workflows for shipping binaries and data.
    Package {
        #[command(subcommand)]
        command: PackageCommands,
    },
    /// Content pipeline helpers (compile, lint, etc.).
    Content {
        #[command(subcommand)]
        command: ContentCommands,
    },
}

#[derive(Args)]
struct BuildEngineArgs {
    /// Enable developer commands at compile time.
    #[arg(long, value_enum, default_value_t = DevMode::Disabled)]
    dev_mode: DevMode,
    /// Select cargo profile (debug or release).
    #[arg(long, value_enum, default_value_t = Profile::Release)]
    profile: Profile,
    /// Build for a specific target triple.
    #[arg(long)]
    target: Option<String>,
}

#[derive(Subcommand)]
enum PackageCommands {
    /// Package the engine binary and compiled data (TOML) only.
    Engine(PackageEngineArgs),
    /// Package the engine, amble_script CLI, and source data.
    Full(PackageFullArgs),
}

#[derive(Args, Clone)]
struct PackageEngineArgs {
    #[command(flatten)]
    options: PackageOptions,
}

#[derive(Args, Clone)]
struct PackageFullArgs {
    #[command(flatten)]
    options: PackageOptions,
}

#[derive(Args, Clone)]
struct PackageOptions {
    /// Override the target triple (defaults to host compiler triple).
    #[arg(long)]
    target: Option<String>,
    /// Cargo build profile used for artifacts.
    #[arg(long, value_enum, default_value_t = Profile::Release)]
    profile: Profile,
    /// Enable developer commands in packaged builds.
    #[arg(long, value_enum, default_value_t = DevMode::Disabled)]
    dev_mode: DevMode,
    /// Where to place staged packages.
    #[arg(long, value_name = "DIR")]
    dist_dir: Option<PathBuf>,
    /// Desired archive style.
    #[arg(long, value_enum, default_value_t = ArchiveFormat::Zip, alias = "archive")]
    format: ArchiveFormat,
    /// Override generated package directory/archive name.
    #[arg(long, value_name = "NAME")]
    name: Option<String>,
}

#[derive(Subcommand)]
enum ContentCommands {
    /// Compile the .amble sources and lint the resulting TOMLs.
    Refresh(ContentRefreshArgs),
}

#[derive(Args)]
struct ContentRefreshArgs {
    /// Source directory containing .amble files.
    #[arg(long, value_name = "DIR", default_value = "amble_script/data/Amble")]
    source: PathBuf,
    /// Output directory for compiled TOML files.
    #[arg(long, value_name = "DIR", default_value = "amble_engine/data")]
    out_dir: PathBuf,
    /// Treat missing files as an error during linting.
    #[arg(long)]
    deny_missing: bool,
}

#[derive(Clone, Copy, ValueEnum)]
enum DevMode {
    Enabled,
    Disabled,
}

#[derive(Clone, Copy, ValueEnum)]
enum Profile {
    Debug,
    Release,
}

impl Profile {
    fn cargo_flag(self) -> Option<&'static str> {
        match self {
            Profile::Debug => None,
            Profile::Release => Some("--release"),
        }
    }

    fn dir_name(self) -> &'static str {
        match self {
            Profile::Debug => "debug",
            Profile::Release => "release",
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum ArchiveFormat {
    Zip,
    Directory,
}

struct Workspace {
    root: PathBuf,
    target_dir: PathBuf,
    engine_version: String,
    script_version: String,
    host_triple: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let workspace = Workspace::detect()?;

    match cli.command {
        Commands::BuildEngine(args) => build_engine(&workspace, &args),
        Commands::Package { command } => match command {
            PackageCommands::Engine(args) => package_engine(&workspace, &args),
            PackageCommands::Full(args) => package_full(&workspace, &args),
        },
        Commands::Content { command } => match command {
            ContentCommands::Refresh(args) => refresh_content(&workspace, &args),
        },
    }
}

fn build_engine(workspace: &Workspace, args: &BuildEngineArgs) -> Result<()> {
    let mut command = cargo_cmd("build", workspace);
    command.arg("-p").arg("amble_engine");
    if let Some(flag) = args.profile.cargo_flag() {
        command.arg(flag);
    }
    if let Some(target) = &args.target {
        command.arg("--target").arg(target);
    }
    if matches!(args.dev_mode, DevMode::Enabled) {
        command.arg("--features").arg("dev-mode");
    }

    run_command(&mut command, "cargo build (amble_engine)")
}

fn package_engine(workspace: &Workspace, args: &PackageEngineArgs) -> Result<()> {
    build_engine(
        workspace,
        &BuildEngineArgs {
            dev_mode: args.options.dev_mode,
            profile: args.options.profile,
            target: args.options.target.clone(),
        },
    )?;
    package_impl(workspace, &args.options, PackageKind::EngineOnly)
}

fn package_full(workspace: &Workspace, args: &PackageFullArgs) -> Result<()> {
    build_engine(
        workspace,
        &BuildEngineArgs {
            dev_mode: args.options.dev_mode,
            profile: args.options.profile,
            target: args.options.target.clone(),
        },
    )?;
    build_script(workspace, args.options.profile, args.options.target.as_deref())?;
    package_impl(workspace, &args.options, PackageKind::FullSuite)
}

fn refresh_content(workspace: &Workspace, args: &ContentRefreshArgs) -> Result<()> {
    let source_dir = workspace.root.join(&args.source);
    let out_dir = workspace.root.join(&args.out_dir);

    if !source_dir.exists() {
        bail!("source directory '{}' does not exist", source_dir.display());
    }

    fs::create_dir_all(&out_dir).with_context(|| format!("unable to create output directory {}", out_dir.display()))?;

    let mut compile_cmd = cargo_cmd("run", workspace);
    compile_cmd
        .arg("-p")
        .arg("amble_script")
        .arg("--bin")
        .arg("amble_script");

    compile_cmd.arg("--");
    compile_cmd.arg("compile-dir");
    compile_cmd.arg(&source_dir);
    compile_cmd.arg("--out-dir");
    compile_cmd.arg(&out_dir);

    run_command(&mut compile_cmd, "amble_script compile-dir")?;

    let mut lint_cmd = cargo_cmd("run", workspace);
    lint_cmd.arg("-p").arg("amble_script").arg("--bin").arg("amble_script");
    lint_cmd.arg("--");
    lint_cmd.arg("lint");
    lint_cmd.arg(&source_dir);
    lint_cmd.arg("--data-dir");
    lint_cmd.arg(&out_dir);
    if args.deny_missing {
        lint_cmd.arg("--deny-missing");
    }

    run_command(&mut lint_cmd, "amble_script lint")
}

fn build_script(workspace: &Workspace, profile: Profile, target: Option<&str>) -> Result<()> {
    let mut command = cargo_cmd("build", workspace);
    command.arg("-p").arg("amble_script");
    if let Some(flag) = profile.cargo_flag() {
        command.arg(flag);
    }
    if let Some(target) = target {
        command.arg("--target").arg(target);
    }

    run_command(&mut command, "cargo build (amble_script)")
}

#[derive(Clone, Copy)]
enum PackageKind {
    EngineOnly,
    FullSuite,
}

fn package_impl(workspace: &Workspace, options: &PackageOptions, kind: PackageKind) -> Result<()> {
    let target_triple = options.target.clone().unwrap_or_else(|| workspace.host_triple.clone());
    let engine_binary_name = executable_name("amble_engine", &target_triple);
    let engine_binary_path = artifact_path(
        &workspace.target_dir,
        &engine_binary_name,
        options.profile,
        options.target.as_deref(),
    );

    if !engine_binary_path.exists() {
        bail!(
            "expected engine binary at '{}' but it was not found",
            engine_binary_path.display()
        );
    }

    if matches!(kind, PackageKind::FullSuite) {
        let script_name = executable_name("amble_script", &target_triple);
        let script_path = artifact_path(
            &workspace.target_dir,
            &script_name,
            options.profile,
            options.target.as_deref(),
        );
        if !script_path.exists() {
            bail!(
                "expected amble_script binary at '{}' but it was not found",
                script_path.display()
            );
        }
    }

    let dist_root = options
        .dist_dir
        .clone()
        .unwrap_or_else(|| workspace.target_dir.join("dist"));
    fs::create_dir_all(&dist_root).with_context(|| format!("unable to ensure dist dir {}", dist_root.display()))?;

    let package_name = options.name.clone().unwrap_or_else(|| match kind {
        PackageKind::EngineOnly => format!("amble-engine-{}-{}", workspace.engine_version, target_triple),
        PackageKind::FullSuite => format!(
            "amble-suite-{}-{}-{}",
            workspace.engine_version, workspace.script_version, target_triple
        ),
    });

    let staging_dir = dist_root.join(&package_name);
    ensure_clean_dir(&staging_dir)?;

    // Always include the engine binary.
    let bin_dir = staging_dir.join("bin");
    fs::create_dir_all(&bin_dir).with_context(|| format!("unable to create {}", bin_dir.display()))?;
    fs::copy(&engine_binary_path, bin_dir.join(&engine_binary_name))
        .with_context(|| format!("failed to copy {}", engine_binary_path.display()))?;

    if matches!(kind, PackageKind::FullSuite) {
        let script_name = executable_name("amble_script", &target_triple);
        let script_path = artifact_path(
            &workspace.target_dir,
            &script_name,
            options.profile,
            options.target.as_deref(),
        );
        fs::copy(&script_path, bin_dir.join(&script_name))
            .with_context(|| format!("failed to copy {}", script_path.display()))?;
    }

    // Copy compiled TOML data.
    let data_src = workspace.root.join("amble_engine/data");
    let data_dst = staging_dir.join("amble_engine/data");
    copy_dir_recursive(&data_src, &data_dst)
        .with_context(|| format!("copying data directory from {}", data_src.display()))?;

    if matches!(kind, PackageKind::FullSuite) {
        let amble_src = workspace.root.join("amble_script/data/Amble");
        let amble_dst = staging_dir.join("amble_script/data/Amble");
        copy_dir_recursive(&amble_src, &amble_dst)
            .with_context(|| format!("copying amble sources from {}", amble_src.display()))?;
    }

    match options.format {
        ArchiveFormat::Directory => {
            println!("Package staged at {}", staging_dir.display());
        },
        ArchiveFormat::Zip => {
            let archive_path = dist_root.join(format!("{package_name}.zip"));
            create_zip_from_dir(&staging_dir, &archive_path)
                .with_context(|| format!("creating archive {}", archive_path.display()))?;
            println!("Archive written to {}", archive_path.display());
        },
    }

    Ok(())
}

fn ensure_clean_dir(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_dir_all(path).with_context(|| format!("removing existing directory {}", path.display()))?;
    }
    fs::create_dir_all(path).with_context(|| format!("creating directory {}", path.display()))
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst).with_context(|| format!("clearing {}", dst.display()))?;
    }
    for entry in WalkDir::new(src) {
        let entry = entry.with_context(|| format!("walking {}", src.display()))?;
        let path = entry.path();
        let relative = match path.strip_prefix(src) {
            Ok(rel) if rel.as_os_str().is_empty() => {
                fs::create_dir_all(dst).with_context(|| format!("creating {}", dst.display()))?;
                continue;
            },
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let target_path = dst.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).with_context(|| format!("creating {}", target_path.display()))?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
            }
            fs::copy(path, &target_path)
                .with_context(|| format!("copying '{}' to '{}'", path.display(), target_path.display()))?;
        }
    }
    Ok(())
}

fn create_zip_from_dir(src: &Path, dest: &Path) -> Result<()> {
    let file = File::create(dest)?;
    let mut zip = ZipWriter::new(file);
    let dir_options = FileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o755);

    for entry in WalkDir::new(src) {
        let entry = entry?;
        let path = entry.path();
        let rel = match path.strip_prefix(src) {
            Ok(rel) if rel.as_os_str().is_empty() => continue,
            Ok(rel) => rel,
            Err(_) => continue,
        };
        let mut name = rel.to_string_lossy().replace('\\', "/");
        if entry.file_type().is_dir() {
            if !name.ends_with('/') {
                name.push('/');
            }
            zip.add_directory(name, dir_options)?;
            continue;
        }

        let perms = if is_executable_candidate(rel) { 0o755 } else { 0o644 };
        let options = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(perms);
        zip.start_file(name, options)?;
        let mut input = File::open(path)?;
        io::copy(&mut input, &mut zip)?;
    }

    zip.finish()?;
    Ok(())
}

fn is_executable_candidate(path: &Path) -> bool {
    let file_name = match path.file_name().and_then(OsStr::to_str) {
        Some(name) => name,
        None => return false,
    };
    file_name.ends_with(".exe") || matches!(file_name, "amble_engine" | "amble_script")
}

fn cargo_cmd(subcommand: &str, workspace: &Workspace) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg(subcommand);
    cmd.current_dir(&workspace.root);
    cmd
}

fn run_command(command: &mut Command, label: &str) -> Result<()> {
    let status = command.status().with_context(|| format!("{label} failed to start"))?;
    if !status.success() {
        bail!("{label} exited with {}", status);
    }
    Ok(())
}

fn artifact_path(target_dir: &Path, binary: &str, profile: Profile, target: Option<&str>) -> PathBuf {
    let mut path = target_dir.to_path_buf();
    if let Some(triple) = target {
        path.push(triple);
    }
    path.push(profile.dir_name());
    path.push(binary);
    path
}

fn executable_name(base: &str, target_triple: &str) -> String {
    if target_triple.contains("windows") {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

impl Workspace {
    fn detect() -> Result<Self> {
        let metadata = MetadataCommand::new()
            .no_deps()
            .exec()
            .context("gathering cargo metadata for workspace")?;

        let root = metadata.workspace_root.into_std_path_buf();
        let target_dir = metadata.target_directory.into_std_path_buf();

        let mut engine_version = None;
        let mut script_version = None;
        for package in metadata.packages {
            match package.name.as_str() {
                "amble_engine" => engine_version = Some(package.version.to_string()),
                "amble_script" => script_version = Some(package.version.to_string()),
                _ => {},
            }
        }

        Ok(Self {
            root,
            target_dir,
            engine_version: engine_version.context("unable to find amble_engine package metadata")?,
            script_version: script_version.context("unable to find amble_script package metadata")?,
            host_triple: detect_host_triple()?,
        })
    }
}

fn detect_host_triple() -> Result<String> {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .context("running `rustc -vV`")?;
    if !output.status.success() {
        bail!("`rustc -vV` exited with {}", output.status);
    }
    let stdout = String::from_utf8(output.stdout).context("parsing rustc output as UTF-8")?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: ").map(str::to_string))
        .ok_or_else(|| anyhow!("failed to parse host triple from rustc -vV output"))
}
