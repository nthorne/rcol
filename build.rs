use std::{env, fs::File, io::Write, path::Path};
use directories::ProjectDirs;

/// Used internally to represent errors when attempting to retrieve
/// the configuration path that confy will use.
enum PathError {
    ProjectDirNotFound,
    CouldNotUnwrap
}

/// Returns the name of the configuration file that confy will use, or
/// a PathError if the file name cannot be resolved.
fn get_config_path() -> Result<String, PathError> {
    let app_name = option_env!("CARGO_PKG_NAME").unwrap();

    ProjectDirs::from("rs", "", app_name)
        .ok_or(PathError::ProjectDirNotFound)?
        .config_dir().to_str()
        .ok_or(PathError::CouldNotUnwrap)
        .map(|s| s.to_string())
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let build_result_file = Path::new(&out_dir).join("build_result.rs");
    let mut fil = File::create(&build_result_file).unwrap();

    let config_path_str = get_config_path()
        .unwrap_or("<UNKNOWN>".to_string());

    // We generate a static about string here, so that we can interpolate the
    // configuration file path at build time, and in turn use that to build
    // up our about documentation. Once confy bumps its version, making the
    // path getter function public, this thing can be dropped.
    writeln!(fil, "pub const ABOUT_STRING: &'static str = \
                 \"Colorize lines from a file, or stdin, by grouping lines according to a given delimiter and column. Configuration data is stored in {}\";", config_path_str).unwrap();
}
