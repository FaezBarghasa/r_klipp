use std::env;
use std::fs;
use std::io::Write;
use ed25519_dalek::{Signer, SigningKey, VerifyingKey};
use sha2::{Sha256, Digest};
use zstd::stream::encode_all;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_binary> <output_file.rklipp>", args[0]);
        return Ok(());
    }

    let input_path = &args[1];
    let output_path = &args[2];

    let private_key_bytes = env::var("RKLIPP_PRIVATE_KEY")?.as_bytes();
    let signing_key: SigningKey = SigningKey::from_bytes(private_key_bytes.try_into()?);

    let binary_data = fs::read(input_path)?;
    let compressed_data = encode_all(&binary_data[..], 0)?;
    let hash = Sha256::digest(&compressed_data);

    let signature = signing_key.sign(&hash);

    let mut output_file = fs::File::create(output_path)?;
    output_file.write_all(b"RKLP")?; // Magic
    output_file.write_all(&(binary_data.len() as u32).to_le_bytes())?;
    output_file.write_all(&hash)?;
    output_file.write_all(&signature.to_bytes())?;
    output_file.write_all(&compressed_data)?;

    Ok(())
}
