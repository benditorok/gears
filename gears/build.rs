use anyhow::*;
use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;

fn main() -> anyhow::Result<()> {
    // Environment setup
    let exec_dir = env::current_dir()?;
    let res_dir_val = exec_dir.join("../res");
    println!(format!("cargo:rustc-env=RES_DIR={}", res_dir_val.display()));

    // This tells cargo to rerun this script if something in res/ changes.
    println!("cargo:rerun-if-changed=../res/*");

    // Prepare what to copy and how
    let mut copy_options = CopyOptions::new();
    copy_options.overwrite = true;
    let paths_to_copy = vec!["../res/"];

    // Copy them next to the compiled executable and set an environment variable for it
    let res_dir = env::current_dir()?.join("../res");

    // Copy the items to the directory where the executable will be built
    let res_dir = env::var("RES_DIR")?;
    copy_items(&paths_to_copy, res_dir, &copy_options)?;

    Ok(())
}
