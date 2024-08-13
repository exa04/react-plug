use anyhow::{anyhow, Context};
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

pub use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use toml::Table;
use which::which;

/// Builds the GUI and its bindings before calling nih_plug_xtask
pub fn main() -> Result<()> {
    let mut xtask_args = std::env::args().skip(1);

    // e.g. "bundle"
    let command = &xtask_args.next().context("No command provided")?;

    // e.g. ["-p", "plugin1", "-p", "plugin2", "--release"]
    let args = xtask_args.collect::<Vec<_>>();

    // If the command does not require building the GUI, just directly run nih_plug xtask
    if command.as_str() != "bundle"
        && command.as_str() != "bundle-universal"
        && command.as_str() != "dev"
    {
        return nih_plug_xtask::main();
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
        chdir_project_root(package)?;

        fs::create_dir_all(Path::new("gui/dist"))?;

        println!("Generating bindings...");

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

        if !Path::new("node_modules").exists() {
            println!("Installing GUI dependencies...");
            if !Command::new(package_manager)
                .arg("install")
                .status()
                .with_context(|| format!("Failed to run `{} install`", { package_manager }))?
                .success()
            {
                return Err(anyhow!("Couldn't install GUI dependencies"));
            }
        }

        println!("Building GUI...");

        if !Command::new(package_manager)
            .arg("run")
            .arg("build")
            .status()
            .with_context(|| {
                format!("Failed to run `build` script using `{}`", {
                    package_manager
                })
            })?
            .success()
        {
            return Err(anyhow!("Couldn't build GUI"));
        }

        if !Command::new("cargo")
            .arg("clean")
            .arg("-p")
            .arg(package)
            .status()
            .context("Failed to clean")?
            .success()
        {
            return Err(anyhow!("Couldn't clean"));
        }
    }

    println!("Bundling...");

    if command.as_str() == "dev" {
        // Bundle with the "dev" cfg flag
        let mut args = vec!["bundle".to_string()];
        args.extend(std::env::args().skip(2));
        args.extend(vec![
            "--config".to_string(),
            r#"build.rustflags=["--cfg", "rp_dev"]"#.to_string(),
        ]);

        nih_plug_xtask::main_with_args("cargo xtask", args).context("nih_plug xtask failed")?;

        packages
            .into_par_iter()
            .map(|package| {
                let mut cwd = get_project_root(&package)
                    .context(format!("Could not change to project root of {}", &package))?;

                cwd.push("gui");

                println!("Starting dev server of {}...", &package);

                if !Command::new(package_manager)
                    .arg("run")
                    .arg("dev")
                    .current_dir(cwd)
                    .status()
                    .with_context(|| {
                        format!(
                            "Failed to run `dev` script for {} using `{}`",
                            &package, &package_manager
                        )
                    })?
                    .success()
                {
                    Ok(())
                } else {
                    Err(anyhow!(
                        "Failed to run `dev` script for {} using `{}`",
                        &package,
                        &package_manager
                    ))
                }
            })
            .collect::<Result<()>>()
    } else {
        nih_plug_xtask::main().context("nih_plug xtask failed")
    }
}

fn get_project_root(project_name: &String) -> Result<PathBuf> {
    let project_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .context(
            "'$CARGO_MANIFEST_DIR' was not set and the current working directory could not be \
             found",
        )?;

    project_dir
        .ancestors()
        .chain(std::iter::once(project_dir.as_path()))
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

            if dir.ends_with(project_name) {
                return Some(dir.into());
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
        .context("Could not find project root directory")
}

/// See [nih_plug_xtask::split_bundle_args].
fn split_bundle_args(args: impl IntoIterator<Item = String>) -> Result<(Vec<String>, Vec<String>)> {
    let mut args = args.into_iter().peekable();
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

/// To a similar effect as [`nih_plug_xtask::chdir_workspace_root`].
///
/// This function will change the current working directory to the root of the workspace that contains
/// the project with the given name. It tries to find the exact project directory by parsing the
/// `Cargo.toml` files of the ancestors of the current working directory.
///
/// This is done because the `gui` subdirectory needs to be changed into and built from.
fn chdir_project_root(project_name: &String) -> Result<()> {
    let root = get_project_root(project_name)?;
    std::env::set_current_dir(root).context("Could not change to project root")
}
