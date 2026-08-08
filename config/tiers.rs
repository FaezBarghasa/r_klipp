
#[cfg(feature = "tier1")]
pub const LOOKAHEAD_BUFFER_SIZE: usize = 64;
#[cfg(feature = "tier2")]
pub const LOOKAHEAD_BUFFER_SIZE: usize = 1024;
#[cfg(feature = "tier3")]
pub const LOOKAHEAD_BUFFER_SIZE: usize = 4096;

#[cfg(not(any(feature = "tier1", feature = "tier2", feature = "tier3")))]
pub const LOOKAHEAD_BUFFER_SIZE: usize = 32; // Default for untiered builds

pub fn setup_build_profile() {
    #[cfg(feature = "tier1")]
    println!("cargo:rustc-cfg=target_feature=\"\"");

    #[cfg(any(feature = "tier2", feature = "tier3"))]
    println!("cargo:rustc-cfg=target_feature=\"+v7e-m\"");

    #[cfg(feature = "tier1")]
    println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=cortex-m3");

    #[cfg(feature = "tier2")]
    println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=cortex-m4");

    #[cfg(feature = "tier3")]
    println!("cargo:rustc-env=RUSTFLAGS=-C target-cpu=cortex-m7");
}
