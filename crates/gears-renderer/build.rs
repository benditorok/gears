use fs_extra::copy_items;
use fs_extra::dir::CopyOptions;
use std::env;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

/// Custom error type for build script.
#[derive(Debug)]
enum BuildError {
    /// Environment variable error.
    EnvVar(env::VarError),

    /// IO error.
    Io(std::io::Error),

    /// File system operation error.
    FsExtra(fs_extra::error::Error),
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::EnvVar(e) => write!(f, "Environment variable error: {}", e),
            BuildError::Io(e) => write!(f, "IO error: {}", e),
            BuildError::FsExtra(e) => write!(f, "File system operation error: {}", e),
        }
    }
}

impl Error for BuildError {}

impl From<env::VarError> for BuildError {
    fn from(err: env::VarError) -> Self {
        BuildError::EnvVar(err)
    }
}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::Io(err)
    }
}

impl From<fs_extra::error::Error> for BuildError {
    fn from(err: fs_extra::error::Error) -> Self {
        BuildError::FsExtra(err)
    }
}

fn main() -> Result<(), BuildError> {
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
