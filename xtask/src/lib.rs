use wasm_opt::{OptimizationOptions, Feature};
use std::process::Command;
mod build_trie;
mod trie;

pub fn build_trie_data() -> Result<(), Box<dyn std::error::Error>> {
    let trie = build_trie::build_trie();
    let bytes = postcard::to_stdvec(&trie)?;
    println!("Postcard serialized trie size: {}", bytes.len());

    let bytes = zstd::encode_all(bytes.as_slice(), 20)?;
    println!("Compressed trie size: {}", bytes.len());

    std::fs::write("data/trie.dat", bytes)?;
    Ok(())
}

pub fn build_and_optimize() -> Result<(), Box<dyn std::error::Error>> {
println!("🦀 Compiling Rust to WASM...");

    // 1. Run the actual build command
    let status = Command::new("cargo")
        .args(["build", "--release", "--target", "wasm32-unknown-unknown"])
        .status()?;

    if !status.success() {
        return Err("Cargo build failed".into());
    }

    // 2. Setup wasm-opt with your specific flags
    println!("🚀 Optimizing WASM...");
    let infile = "target/wasm32-unknown-unknown/release/rust_canto.wasm";
    let outfile = "rust_canto.wasm";

    let mut options = OptimizationOptions::new_optimize_for_size(); // -Oz

    // --strip-debug
    options.debug_info(false); 

    // --enable-bulk-memory --enable-sign-ext
    options.enable_feature(Feature::BulkMemory);
    options.enable_feature(Feature::SignExt);

    // --disable-reference-types
    options.disable_feature(Feature::ReferenceTypes);

    // Execute optimization
    options.run(infile, outfile)?;

    let original_size = std::fs::metadata(infile)?.len() / 1024;
    let optimized_size = std::fs::metadata(outfile)?.len() / 1024;

    println!("✅ Done! Size reduced: {}KB -> {}KB", original_size, optimized_size);
    Ok(())
}
