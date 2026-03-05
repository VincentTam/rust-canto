// src/bin/dist.rs
#[path = "../../build_deps/mod.rs"]
mod generate;

use std::process::Command;
use wasm_opt::{Feature, OptimizationOptions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("📦 Starting distribution build...");

    generate::build_trie_data()?;

    println!("🦀 Compiling Rust to WASM...");
    let status = Command::new("cargo")
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()?;
    if !status.success() {
        return Err("Cargo build failed".into());
    }

    println!("🚀 Optimizing WASM...");
    let infile = "target/wasm32-unknown-unknown/release/rust_canto.wasm";
    let outfile = "rust_canto.wasm";

    let mut options = OptimizationOptions::new_optimize_for_size();
    options.debug_info(false);
    options.enable_feature(Feature::BulkMemory);
    options.enable_feature(Feature::SignExt);
    options.disable_feature(Feature::ReferenceTypes);
    options.run(infile, outfile)?;

    let original = std::fs::metadata(infile)?.len() / 1024;
    let optimized = std::fs::metadata(outfile)?.len() / 1024;
    println!("✅ Done! Size reduced: {}KB -> {}KB", original, optimized);
    Ok(())
}
