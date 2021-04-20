use ::{
    angle_gles_generator::{build_eglplatform, build_khrplatform, gen_egl, gen_gles},
    bindgen::{Bindings, RustTarget},
    gl_generator::Fallbacks,
    pico_args::Arguments,
    serde::Deserialize,
    std::{
        convert::Infallible,
        fs::File,
        ops::Deref,
        path::{Path, PathBuf},
    },
};

#[derive(Eq, PartialEq, Deserialize)]
pub enum Fallback {
    All,
    None,
}

impl Fallback {
    pub fn convert(self) -> Fallbacks {
        if self == Fallback::All {
            Fallbacks::All
        } else {
            Fallbacks::None
        }
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub rust_version: Box<str>,
    pub angle_out_home: Box<str>,
    pub fallback: Fallback,
    pub egl_version: (u8, u8),
    pub egl_extensions: Vec<Box<str>>,
    pub gles_version: (u8, u8),
    pub gles_extensions: Vec<Box<str>>,
}

/// 大文字小文字を問わないバージョン文字列を変換する。
/// bindgenのMSRVが`1.40`なので`1.40`か`nightly`が有効。
/// 対応するバージョンがない場合`None`とする。
fn convert_rust_target(rust_version: Box<str>) -> Option<RustTarget> {
    use RustTarget::*;
    match &*rust_version.to_lowercase() {
        "1.40" => Some(Stable_1_40),
        "nightly" => Some(Nightly),
        _ => None,
    }
}

fn coerce_generate_and_write(bindings: Result<Bindings, ()>, output: &Path) {
    bindings
        .expect("generate the bindings failed")
        .write_to_file(output)
        .expect("write the bindings failed");
}

fn present_config_file_path(
    Config {
        rust_version,
        angle_out_home,
        fallback,
        egl_version,
        egl_extensions,
        gles_version,
        gles_extensions,
    }: Config,
    dest: PathBuf,
) -> Result<(), Box<dyn std::error::Error>> {
    // XXX: khrplatform.hとeglplatform.hの参照にANGLEに依存しないようにしたい
    let angle_out_home = Path::new(angle_out_home.deref());
    if !angle_out_home.exists() {
        eprintln!("`angle_out_home` not found");
        return Ok(());
    }

    let rust_target = convert_rust_target(rust_version);
    let dest = Path::new(dest.deref());

    let khrplatform_output = dest.join("khrplatform_bindings.rs");
    println!("output file {:?}", khrplatform_output);
    let binding = build_khrplatform(angle_out_home, rust_target).generate();
    coerce_generate_and_write(binding, &khrplatform_output);

    let eglplatform_output = dest.join("eglplatform_bindings.rs");
    println!("output file {:?}", eglplatform_output);
    let binding = build_eglplatform(angle_out_home, rust_target).generate();
    coerce_generate_and_write(binding, &eglplatform_output);

    let fallbacks = fallback.convert();

    let egl_output = dest.join("egl_bindings.rs");
    println!("output file {:?}", egl_output);
    let egl_extensions = egl_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    gen_egl(&egl_output, egl_version, fallbacks, egl_extensions)?;

    let gles_output = dest.join("gl_bindings.rs");
    println!("output file {:?}", gles_output);
    let gles_extensions = gles_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    gen_gles(&gles_output, gles_version, fallbacks, gles_extensions)?;

    Ok(())
}

fn generate(mut args: Arguments) -> Result<(), Box<dyn std::error::Error>> {
    let config = args.value_from_fn(["--config", "-g"], |a| {
        ron::de::from_reader(std::io::BufReader::new(File::open(a)?))
    })?;
    let dest = args.value_from_fn::<_, _, Infallible>(["--out", "-o"], |a| Ok(a.into()))?;
    present_config_file_path(config, dest)
}

const HELP: &str = "\
Usage:
    angle_gles_generator (SUBCOMMAND | FLAGS)

SUBCOMMAND:
    generate (--config | -g) PATH (--out | -o) OUT

FLAGS:
    (--help | -h)   Print this help
";

fn print_help() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", HELP);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = pico_args::Arguments::from_env();
    if args.contains(["--help", "-h"]) {
        print_help()
    } else {
        match args.subcommand() {
            Ok(Some(it)) => match &*it {
                "generate" | "g" => generate(args),
                _ => print_help(),
            },
            Ok(None) => print_help(),
            Err(e) => {
                println!("{}", e);
                Ok(())
            },
        }
    }
}
