use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context;
use ci_tool::commit_parse;
use ci_tool::gitutils;
use ci_tool::public_api;
use ci_tool::public_api::PublicApi;
use clap::Parser;
use glob::glob;

/// ZROJ sandbox
#[derive(Parser)]
#[command(
    name = "zroj-sandbox", 
    author,
    disable_version_flag = true,
    about,
    long_about = None,
)]
struct Cli {
    /// currnet commit sha/id
    dev: String,

    /// old commit/id to be compared with
    #[arg(short, long, default_value = "master")]
    base: String,

    /// the dir that contains workspace manifest file
    #[arg(short = 'w', long, default_value = ".")]
    workspace_dir: String,
}

fn gen_workspace_public_apis(
    proj_dir: impl AsRef<std::path::Path>,
) -> HashMap<PathBuf, anyhow::Result<PublicApi>> {
    let mut map = HashMap::new();

    for entry in glob(
        proj_dir
            .as_ref()
            .join("crates/*/Cargo.toml")
            .to_str()
            .unwrap(),
    )
    .expect("Failed to read glob pattern")
    {
        match entry {
            Ok(path) => {
                let v = gen_public_api(&path);
                map.insert(path, v);
            }
            Err(e) => eprintln!("{:?}", e),
        }
    }
    map
}

fn gen_public_api(manifest_path: impl AsRef<std::path::Path>) -> anyhow::Result<PublicApi> {
    // Build rustdoc JSON
    eprintln!("gen public api {}", manifest_path.as_ref().display());
    let rustdoc_json = rustdoc_json::Builder::default()
        .toolchain(public_api::MINIMUM_NIGHTLY_RUST_VERSION)
        // .manifest_path(proj_dir.as_ref().join("crates/*/Cargo.toml"))
        .manifest_path(manifest_path.as_ref())
        .build()
        .context("build rustdoc json")?;

    eprintln!("json path: {}", rustdoc_json.display());

    let apis = public_api::Builder::from_rustdoc_json(rustdoc_json)
        .omit_auto_derived_impls(true)
        .omit_auto_trait_impls(true)
        .omit_blanket_impls(true)
        .sorted(true)
        .build()
        .context("parse public api")?;

    Ok(apis)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let proj_dir = std::fs::canonicalize(cli.workspace_dir)?;

    if gitutils::has_staged(&proj_dir)? || gitutils::has_unstaged(&proj_dir)? {
        return Err(anyhow!("you have unstaged or uncommitted changes"));
    }

    println!("# Commits\n");
    // let cur_commit = match gitutils::current_branch(&proj_dir)? {
    //     Some(r) => r,
    //     None => gitutils::current_commit(&proj_dir)?,
    // };

    // only consider commits from LCA to dev
    let commits = gitutils::commit_list(&proj_dir, &cli.dev, &cli.base)?;
    let commits = commit_parse::render_changelog(&commits);
    println!("{commits}");

    // Install a compatible nightly toolchain if it is missing
    rustup_toolchain::install(public_api::MINIMUM_NIGHTLY_RUST_VERSION).unwrap();

    // no matter what error happen, checkout back to the current commit
    gitutils::git_checkout(&proj_dir, &cli.dev, true, false)?;

    let cur_apis = gen_workspace_public_apis(&proj_dir)
        .into_iter()
        .map(|(k, v)| v.map(|v| (k, v)))
        .collect::<anyhow::Result<HashMap<PathBuf, PublicApi>>>()
        .context("build new api")?;

    gitutils::git_checkout(&proj_dir, &cli.base, true, false)?;

    let mut old_apis = gen_workspace_public_apis(&proj_dir);

    println!("# API Changes\n");
    for (crate_name, cur_api) in cur_apis {
        let diff = if let Some(Ok(old_api)) = old_apis.remove(&crate_name) {
            public_api::diff::PublicApiDiff::between(old_api, cur_api)
        } else {
            public_api::diff::PublicApiDiff {
                removed: Vec::new(),
                changed: Vec::new(),
                added: cur_api.items,
            }
        };
        let changes = diff.render_changelog();
        if changes.trim().len() > 0 {
            println!(
                "## {}\n{changes}",
                crate_name
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
            );
        }
    }

    Ok(())
}
