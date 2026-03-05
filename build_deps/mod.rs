pub mod build_trie;
pub mod trie;

pub fn build_trie_data() -> Result<(), Box<dyn std::error::Error>> {
    let trie = build_trie::build_trie();
    let bytes = postcard::to_stdvec(&trie)?;
    let compressed = zstd::encode_all(bytes.as_slice(), 20)?;

    let out_dir = std::env::var("OUT_DIR")?;
    let dest_path = std::path::Path::new(&out_dir).join("trie.dat");

    std::fs::write(dest_path, compressed)?;
    Ok(())
}
