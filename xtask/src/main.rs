use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Get the arguments passed to cargo xtask
    // e.g., "cargo xtask dist" -> args will contain "dist"
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str());

    match command {
        // Run this to generate trie.dat manually
        Some("data") => {
            println!("🛠️ Generating dictionary data...");
            xtask::build_trie_data()?;
        }

        // Run this to do the full WASM build + wasm-opt
        Some("dist") => {
            println!("📦 Starting distribution build...");
            // build_trie_data() is called by build.rs automatically,
            // but we call it here too just to be safe.
            xtask::build_trie_data()?;
            xtask::build_and_optimize()?;
        }

        // Default: Show help
        _ => {
            eprintln!("Usage: cargo xtask [data|dist]");
            eprintln!("  data: Just generate data/trie.dat");
            eprintln!("  dist: Build WASM and optimize with wasm-opt");
            std::process::exit(1);
        }
    }

    Ok(())
}
