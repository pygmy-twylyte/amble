//! Project automation entry point.
//!
//! Implements the `xtask` command-line helpers for building, packaging,
//! and managing Amble development workflows.

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
use semver::Version;
use toml_edit::{Document, value};
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
    /// Build the Amble engine with optional `DEV_MODE` support.
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
    /// End-to-end release workflow (version bump, publish, packages).
    Release(ReleaseArgs),
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
    /// Package the engine, `amble_script` CLI, and source data.
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

#[derive(Args)]
struct ReleaseArgs {
    /// Version applied to both amble_engine and amble_script (SemVer).
    #[arg(long, value_name = "SEMVER")]
    version: String,
    /// Target triple used for Linux packages (defaults to host triple).
    #[arg(long, value_name = "TRIPLE")]
    linux_target: Option<String>,
    /// Target triple used for Windows packages.
    #[arg(long, value_name = "TRIPLE", default_value = "x86_64-pc-windows-msvc")]
    windows_target: String,
    /// Run every step except publishing to crates.io (useful for dry runs).
    #[arg(long)]
    skip_publish: bool,
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
    let mut workspace = Workspace::detect()?;

    match cli.command {
        Commands::BuildEngine(args) => build_engine(&workspace, &args),
        Commands::Package { command } => match command {
            PackageCommands::Engine(args) => package_engine(&workspace, &args),
            PackageCommands::Full(args) => package_full(&workspace, &args),
        },
        Commands::Content { command } => match command {
            ContentCommands::Refresh(args) => refresh_content(&workspace, &args),
        },
        Commands::Release(args) => release(&mut workspace, &args),
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

fn release(workspace: &mut Workspace, args: &ReleaseArgs) -> Result<()> {
    let new_version = Version::parse(&args.version).context("parsing --version argument")?;
    let linux_target = args
        .linux_target
        .clone()
        .unwrap_or_else(|| workspace.host_triple.clone());
    let windows_target = args.windows_target.clone();

    println!("==> Verifying git status");
    ensure_git_clean(workspace)?;
    ensure_on_main(workspace)?;
    ensure_git_synced_with_origin(workspace)?;

    println!("==> Running cargo check (workspace, all targets)");
    cargo_check_all_targets(workspace)?;
    println!("==> Running cargo test (workspace)");
    cargo_test_workspace(workspace)?;

    println!("==> Refreshing compiled content");
    let refresh_args = ContentRefreshArgs {
        source: PathBuf::from("amble_script/data/Amble"),
        out_dir: PathBuf::from("amble_engine/data"),
        deny_missing: true,
    };
    refresh_content(workspace, &refresh_args)?;

    ensure_git_clean(workspace).context("content refresh produced changes; commit or revert them before releasing")?;

    println!("==> Updating crate versions to v{new_version}");
    update_manifest_version(&workspace.root.join("amble_engine/Cargo.toml"), &new_version)?;
    update_manifest_version(&workspace.root.join("amble_script/Cargo.toml"), &new_version)?;
    update_lock_versions(&workspace.root.join("Cargo.lock"), &new_version)?;

    workspace.engine_version = new_version.to_string();
    workspace.script_version = new_version.to_string();

    println!("==> Rechecking workspace after version bump");
    cargo_check_all_targets(workspace)?;

    let files_to_commit = [
        workspace.root.join("amble_engine/Cargo.toml"),
        workspace.root.join("amble_script/Cargo.toml"),
        workspace.root.join("Cargo.lock"),
    ];
    git_add(workspace, &files_to_commit)?;

    let commit_message = format!("Release v{new_version}");
    println!("==> Creating commit: {commit_message}");
    git_commit(workspace, &commit_message)?;

    let tag_name = format!("v{new_version}");
    println!("==> Tagging release: {tag_name}");
    git_tag(workspace, &tag_name, &commit_message)?;

    println!("==> Pushing main branch and tag");
    git_push(workspace, "origin", "main")?;
    git_push(workspace, "origin", &tag_name)?;

    if args.skip_publish {
        println!("==> Skipping cargo publish steps (flag enabled)");
    } else {
        println!("==> Publishing amble_script");
        cargo_publish(workspace, "amble_script")?;
        println!("==> Publishing amble_engine");
        cargo_publish(workspace, "amble_engine")?;
    }

    println!("==> Building Linux distributions");
    let linux_engine = PackageEngineArgs {
        options: PackageOptions {
            target: Some(linux_target.clone()),
            profile: Profile::Release,
            dev_mode: DevMode::Disabled,
            dist_dir: None,
            format: ArchiveFormat::Zip,
            name: None,
        },
    };
    package_engine(workspace, &linux_engine)?;

    let linux_full = PackageFullArgs {
        options: PackageOptions {
            target: Some(linux_target),
            profile: Profile::Release,
            dev_mode: DevMode::Enabled,
            dist_dir: None,
            format: ArchiveFormat::Zip,
            name: None,
        },
    };
    package_full(workspace, &linux_full)?;

    println!("==> Building Windows distributions");
    let windows_engine = PackageEngineArgs {
        options: PackageOptions {
            target: Some(windows_target.clone()),
            profile: Profile::Release,
            dev_mode: DevMode::Disabled,
            dist_dir: None,
            format: ArchiveFormat::Zip,
            name: None,
        },
    };
    package_engine(workspace, &windows_engine)?;

    let windows_full = PackageFullArgs {
        options: PackageOptions {
            target: Some(windows_target),
            profile: Profile::Release,
            dev_mode: DevMode::Enabled,
            dist_dir: None,
            format: ArchiveFormat::Zip,
            name: None,
        },
    };
    package_full(workspace, &windows_full)?;

    println!("==> Release v{new_version} completed");
    Ok(())
}

fn cargo_check_all_targets(workspace: &Workspace) -> Result<()> {
    let mut command = cargo_cmd("check", workspace);
    command.arg("--workspace").arg("--all-targets");
    run_command(&mut command, "cargo check --workspace --all-targets")
}

fn cargo_test_workspace(workspace: &Workspace) -> Result<()> {
    let mut command = cargo_cmd("test", workspace);
    command.arg("--workspace");
    run_command(&mut command, "cargo test --workspace")
}

fn cargo_publish(workspace: &Workspace, package: &str) -> Result<()> {
    let mut command = cargo_cmd("publish", workspace);
    command.arg("-p").arg(package);
    let label = format!("cargo publish ({package})");
    run_command(&mut command, &label)
}

fn ensure_git_clean(workspace: &Workspace) -> Result<()> {
    let mut command = git_cmd(workspace);
    command.arg("status").arg("--porcelain");
    let output = command.output().context("git status --porcelain failed to run")?;
    if !output.status.success() {
        bail!("git status --porcelain exited with {}", output.status);
    }
    let stdout = String::from_utf8(output.stdout).context("git status output is not valid UTF-8")?;
    if !stdout.trim().is_empty() {
        bail!("working tree has local changes; please commit or stash them before running release");
    }
    Ok(())
}

fn ensure_on_main(workspace: &Workspace) -> Result<()> {
    let branch = git_output(workspace, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let branch = branch.trim();
    if branch != "main" {
        bail!("release command must run from 'main' (current branch: {branch})");
    }
    Ok(())
}

fn ensure_git_synced_with_origin(workspace: &Workspace) -> Result<()> {
    let mut fetch = git_cmd(workspace);
    fetch.arg("fetch").arg("origin");
    run_command(&mut fetch, "git fetch origin")?;

    let head = git_output(workspace, &["rev-parse", "HEAD"])?;
    let remote = git_output(workspace, &["rev-parse", "origin/main"])?;
    if head.trim() != remote.trim() {
        bail!("local main is not up to date with origin/main; please pull or push pending commits");
    }
    Ok(())
}

fn git_add(workspace: &Workspace, paths: &[PathBuf]) -> Result<()> {
    let mut command = git_cmd(workspace);
    command.arg("add");
    for path in paths {
        command.arg(path);
    }
    run_command(&mut command, "git add")
}

fn git_commit(workspace: &Workspace, message: &str) -> Result<()> {
    let mut command = git_cmd(workspace);
    command.arg("commit").arg("-m").arg(message);
    let label = format!("git commit ({message})");
    run_command(&mut command, &label)
}

fn git_tag(workspace: &Workspace, tag: &str, message: &str) -> Result<()> {
    let mut command = git_cmd(workspace);
    command.arg("tag").arg("-a").arg(tag).arg("-m").arg(message);
    let label = format!("git tag {tag}");
    run_command(&mut command, &label)
}

fn git_push(workspace: &Workspace, remote: &str, reference: &str) -> Result<()> {
    let mut command = git_cmd(workspace);
    command.arg("push").arg(remote).arg(reference);
    let label = format!("git push {remote} {reference}");
    run_command(&mut command, &label)
}

fn update_manifest_version(manifest: &Path, version: &Version) -> Result<()> {
    let contents = fs::read_to_string(manifest).with_context(|| format!("unable to read {}", manifest.display()))?;
    let mut doc: Document = contents
        .parse()
        .with_context(|| format!("parsing {} as TOML", manifest.display()))?;
    doc["package"]["version"] = value(version.to_string());
    fs::write(manifest, doc.to_string()).with_context(|| format!("writing updated {}", manifest.display()))?;
    Ok(())
}

fn update_lock_versions(lock_path: &Path, version: &Version) -> Result<()> {
    let contents = fs::read_to_string(lock_path).with_context(|| format!("unable to read {}", lock_path.display()))?;
    let mut doc: Document = contents
        .parse()
        .with_context(|| format!("parsing {} as TOML", lock_path.display()))?;
    let mut engine_updated = false;
    let mut script_updated = false;
    let packages = doc["package"]
        .as_array_of_tables_mut()
        .context("Cargo.lock missing [[package]] entries")?;
    for package in packages.iter_mut() {
        let Some(name) = package.get("name").and_then(|item| item.as_str()) else {
            continue;
        };
        match name {
            "amble_engine" => {
                package["version"] = value(version.to_string());
                engine_updated = true;
            },
            "amble_script" => {
                package["version"] = value(version.to_string());
                script_updated = true;
            },
            _ => {},
        }
    }
    if !engine_updated || !script_updated {
        bail!("failed to update Cargo.lock entries for amble_engine and amble_script");
    }
    fs::write(lock_path, doc.to_string()).with_context(|| format!("writing updated {}", lock_path.display()))?;
    Ok(())
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
        PackageKind::EngineOnly => format!("amble-engine-v{}-{}", workspace.engine_version, target_triple),
        PackageKind::FullSuite => format!(
            "amble-suite-v{}+cli{}-{}",
            workspace.engine_version, workspace.script_version, target_triple
        ),
    });

    let staging_dir = dist_root.join(&package_name);
    ensure_clean_dir(&staging_dir)?;

    // Always include the engine binary.
    fs::copy(&engine_binary_path, staging_dir.join(&engine_binary_name))
        .with_context(|| format!("failed to copy {}", engine_binary_path.display()))?;

    if matches!(kind, PackageKind::FullSuite) {
        let script_name = executable_name("amble_script", &target_triple);
        let script_path = artifact_path(
            &workspace.target_dir,
            &script_name,
            options.profile,
            options.target.as_deref(),
        );
        fs::copy(&script_path, staging_dir.join(&script_name))
            .with_context(|| format!("failed to copy {}", script_path.display()))?;
    }

    // Copy compiled TOML data.
    let data_src = workspace.root.join("amble_engine/data");
    let data_dst = staging_dir.join("data");
    copy_dir_recursive(&data_src, &data_dst)
        .with_context(|| format!("copying data directory from {}", data_src.display()))?;

    if matches!(kind, PackageKind::FullSuite) {
        let amble_src = workspace.root.join("amble_script/data/Amble");
        let amble_dst = staging_dir.join("content/Amble");
        copy_dir_recursive(&amble_src, &amble_dst)
            .with_context(|| format!("copying amble sources from {}", amble_src.display()))?;
    }

    copy_support_files(workspace, &staging_dir)?;

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

fn copy_support_files(workspace: &Workspace, staging_dir: &Path) -> Result<()> {
    let license_src = workspace.root.join("LICENSE");
    copy_optional_file(&license_src, &staging_dir.join("LICENSE"))?;

    let readme_src = workspace.root.join("docs/dist_readme.md");
    copy_optional_file(&readme_src, &staging_dir.join("README.md"))
}

fn copy_optional_file(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Ok(());
    }
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::copy(src, dst).with_context(|| format!("copying '{}' to '{}'", src.display(), dst.display()))?;
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

fn git_cmd(workspace: &Workspace) -> Command {
    let mut cmd = Command::new("git");
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

fn git_output(workspace: &Workspace, args: &[&str]) -> Result<String> {
    let mut command = git_cmd(workspace);
    for arg in args {
        command.arg(arg);
    }
    let output = command
        .output()
        .with_context(|| format!("git {} failed to run", args.join(" ")))?;
    if !output.status.success() {
        bail!("git {} exited with {}", args.join(" "), output.status);
    }
    let stdout = String::from_utf8(output.stdout).context("git output is not valid UTF-8")?;
    Ok(stdout)
}
