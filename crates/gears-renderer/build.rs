use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    // Get output directory where executable will be built
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    // Go up 3 levels to get to target/debug or target/release
    let target_dir = out_dir.ancestors().nth(3).unwrap();

    // Watch for resource changes
    println!("cargo:rerun-if-changed=res/*");

    // Setup copy options
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;

    // Copy resources relative to executable
    let res_source = env::current_dir()?.join("../../res");
    let res_target = target_dir.join("res");

    // Create target directory if it doesn't exist
    std::fs::create_dir_all(&res_target)?;

    // Copy resource directory
    copy_items(&[res_source], target_dir, &copy_options)?;

    // Set environment variable for resource path
    println!("cargo:rustc-env=RES_DIR=res");

    Ok(())
}
