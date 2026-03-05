mod build_trie;
mod trie;

fn main() {
    build_trie_data().expect("Failed to build trie data");
}

fn build_trie_data() -> Result<(), Box<dyn std::error::Error>> {
    let trie = build_trie::build_trie();
    let bytes = postcard::to_stdvec(&trie)?;
    println!("Postcard serialized trie size: {}", bytes.len());

    let bytes = zstd::encode_all(bytes.as_slice(), 20)?;
    println!("Compressed trie size: {}", bytes.len());

    std::fs::write("data/trie.dat", bytes)?;
    Ok(())
}
