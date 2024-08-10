use anyhow::{anyhow, Context};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;

pub use anyhow::Result;
use toml::Table;
use which::which;

fn build_usage_string(command_name: &str) -> String {
    format!(
        "Usage:
  {command_name} bundle <package> [--release]
  {command_name} bundle -p <package1> -p <package2> ... [--release]

  {command_name} bundle-universal <package> [--release]  (macOS only)
  {command_name} bundle-universal -p <package1> -p <package2> ... [--release]  (macOS only)

  All other 'cargo build' options are supported, including '--target' and '--profile'."
    )
}

pub fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);

    let command = args.next();

    if !command.is_some_and(|c| c == "bundle" || c == "bundle-universal") {
        return nih_plug_xtask::main().context("Failed to run nih_plug xtask");
    }

    let package_manager = if which("bun").is_ok() {
        "bun"
    } else if which("yarn").is_ok() {
        "yarn"
    } else if which("pnpm").is_ok() {
        "pnpm"
    } else if which("npm").is_ok() {
        "npm"
    } else {
        return Err(anyhow!(
            "No JS package manager found. You need bun, yarn, pnpm, or npm."
        ));
    };

    let (packages, _) = split_bundle_args(args)?;

    for package in packages.iter() {
        chdir_workspace_root(package)?;

        if !Command::new("cargo")
            .arg("test")
            .status()
            .context("Failed to run 'cargo test'")?
            .success()
        {
            return Err(anyhow::anyhow!("Tests failed"));
        }

        std::env::set_current_dir("gui")
            .context("Could not change to GUI directory. Do you have a /gui directory?")?;

        if !Command::new(package_manager)
            .arg("run")
            .arg("build")
            .status()?
            .success()
        {
            return Err(anyhow!("Couldn't build GUI"));
        }
    }

    nih_plug_xtask::main().context("Failed to run nih_plug xtask")
}

pub fn chdir_workspace_root(project_name: &String) -> Result<()> {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .context(
            "'$CARGO_MANIFEST_DIR' was not set and the current working directory could not be \
             found",
        )?;

    let root = project_dir
        .ancestors()
        .map(|dir| {
            let cargo_file = dir.join("Cargo.toml");
            if !cargo_file.exists() {
                return None;
            }

            let file = File::open(cargo_file);
            if file.is_err() {
                return None;
            }

            let mut contents = String::new();
            if file.unwrap().read_to_string(&mut contents).is_err() {
                return None;
            }

            contents
                .parse::<Table>()
                .unwrap()
                .get("workspace")
                .and_then(|workspace| workspace.get("members"))
                .and_then(|members| members.as_array())
                .and_then(|members| {
                    members
                        .iter()
                        .filter_map(|m| m.as_str())
                        .find(|m| m.ends_with(project_name))
                })
                .map(|member| dir.join(member))
        })
        .find(Option::is_some)
        .flatten()
        .context("Could not find workspace root directory")?;

    std::env::set_current_dir(root).context("Could not change to workspace root directory")
}

// Taken directly from nih_plug_xtask
fn split_bundle_args(args: impl Iterator<Item = String>) -> Result<(Vec<String>, Vec<String>)> {
    let mut args = args.peekable();
    let mut packages = Vec::new();
    if args.peek().map(|s| s.as_str()) == Some("-p") {
        while args.peek().map(|s| s.as_str()) == Some("-p") {
            packages.push(args.nth(1).context("Missing package name after -p")?);
        }
    } else {
        packages.push(args.next().context("Missing package name")?);
    };
    let other_args: Vec<_> = args.collect();

    Ok((packages, other_args))
}
