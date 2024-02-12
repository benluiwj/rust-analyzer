//! Runs `rustc --print target-spec-json` to get the target_data_layout.
use std::process::Command;

use rustc_hash::FxHashMap;

use crate::{utf8_stdout, ManifestPath, Sysroot};

/// Determines how `rustc --print target-spec-json` is discovered and invoked.
pub enum RustcDataLayoutConfig<'a> {
    /// Use `rustc --print target-spec-json`, either from with the binary from the sysroot or by discovering via
    /// [`toolchain::rustc`].
    Rustc(Option<&'a Sysroot>),
    /// Use `cargo --print target-spec-json`, either from with the binary from the sysroot or by discovering via
    /// [`toolchain::cargo`].
    Cargo(Option<&'a Sysroot>, &'a ManifestPath),
}

pub fn get(
    config: RustcDataLayoutConfig<'_>,
    target: Option<&str>,
    extra_env: &FxHashMap<String, String>,
) -> anyhow::Result<String> {
    let output = match config {
        RustcDataLayoutConfig::Cargo(sysroot, cargo_toml) => {
            let cargo = Sysroot::discover_tool(sysroot, toolchain::Tool::Cargo)?;
            let mut cmd = Command::new(cargo);
            cmd.envs(extra_env);
            cmd.current_dir(cargo_toml.parent())
                .args(["-Z", "unstable-options", "--print", "target-spec-json"])
                .env("RUSTC_BOOTSTRAP", "1");
            if let Some(target) = target {
                cmd.args(["--target", target]);
            }
            utf8_stdout(cmd)
        }
        RustcDataLayoutConfig::Rustc(sysroot) => {
            let rustc = Sysroot::discover_tool(sysroot, toolchain::Tool::Rustc)?;
            let mut cmd = Command::new(rustc);
            cmd.envs(extra_env)
                .args(["-Z", "unstable-options", "--print", "target-spec-json"])
                .env("RUSTC_BOOTSTRAP", "1");
            if let Some(target) = target {
                cmd.args(["--target", target]);
            }
            utf8_stdout(cmd)
        }
    }?;
    (|| Some(output.split_once(r#""data-layout": ""#)?.1.split_once('"')?.0.to_owned()))()
        .ok_or_else(|| anyhow::format_err!("could not fetch target-spec-json from command output"))
}
