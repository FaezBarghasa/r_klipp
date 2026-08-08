use std::env;
use std::fs;
use ed25519_dalek::{Signer, SigningKey};
use sha2::{Sha256, Digest};

fn main() {
    let args: Vec<String> = env::args().collect();
    let binary_path = &args[1];
    let output_path = &args[2];
    let private_key_hex = env::var("RKLIPP_PRIVATE_KEY").expect("RKLIPP_PRIVATE_KEY not set");

    let binary = fs::read(binary_path).unwrap();
    let compressed = zstd::encode_all(&binary[..], 0).unwrap();

    let mut hasher = Sha256::new();
    hasher.update(&compressed);
    let hash = hasher.finalize();

    let private_key_bytes = hex::decode(private_key_hex).unwrap();
    let signing_key = SigningKey::from_bytes(&private_key_bytes.try_into().unwrap());
    let signature = signing_key.sign(&hash);

    // In a real implementation, we'd package this into a proper .rklipp file format
    fs::write(output_path, signature.to_bytes()).unwrap();
}
