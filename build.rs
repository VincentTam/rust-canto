#[path = "build_deps/mod.rs"]
mod codegen; // Avoid 'gen' keyword in 2024 edition

fn main() {
    // Re-run if data files change
    println!("cargo:rerun-if-changed=data/");

    if let Err(e) = codegen::build_trie_data() {
        eprintln!("Build script failed: {}", e);
        std::process::exit(1);
    }
}
