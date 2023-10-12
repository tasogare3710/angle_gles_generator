use ::{
    angle_gles_generator::{build_eglplatform, build_khrplatform, gen_egl, gen_gles},
    bindgen::RustTarget,
    gl_generator::Fallbacks,
    pico_args::Arguments,
    serde::Deserialize,
    std::{
        convert::Infallible,
        fs::File,
        ops::Deref,
        path::{Path, PathBuf},
        str::FromStr,
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
pub enum Generator {
    DebugStructGenerator,
    GlobalGenerator,
    StaticGenerator,
    StaticStructGenerator,
    StructGenerator,
}

macro_rules! pass_generator {
    ($generator:ident, $generator_fn:ident, $output:ident, $version:ident, $fallbacks:ident, $extensions:ident) => {{
        match $generator {
            Generator::DebugStructGenerator => $generator_fn(
                &$output,
                $version,
                $fallbacks,
                $extensions,
                gl_generator::DebugStructGenerator,
            ),
            Generator::GlobalGenerator => $generator_fn(
                &$output,
                $version,
                $fallbacks,
                $extensions,
                gl_generator::GlobalGenerator,
            ),
            Generator::StaticGenerator => $generator_fn(
                &$output,
                $version,
                $fallbacks,
                $extensions,
                gl_generator::StaticGenerator,
            ),
            Generator::StaticStructGenerator => $generator_fn(
                &$output,
                $version,
                $fallbacks,
                $extensions,
                gl_generator::StaticStructGenerator,
            ),
            Generator::StructGenerator => $generator_fn(
                &$output,
                $version,
                $fallbacks,
                $extensions,
                gl_generator::StructGenerator,
            ),
        }
    }};
}

#[derive(Deserialize)]
pub struct Config {
    pub generator: Generator,
    pub rust_version: Box<str>,
    pub angle_out_home: Box<str>,
    pub fallback: Fallback,
    pub egl_version: (u8, u8),
    pub egl_extensions: Vec<Box<str>>,
    pub gles_version: (u8, u8),
    pub gles_extensions: Vec<Box<str>>,
}

fn present_config_file_path(
    Config {
        generator,
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

    let rust_target = RustTarget::from_str(&*rust_version)?;
    let dest = Path::new(dest.deref());

    let khrplatform_output = dest.join("khrplatform_bindings.rs");
    println!("output file {:?}", khrplatform_output);
    build_khrplatform(angle_out_home, rust_target)
        .generate()?
        .write_to_file(&khrplatform_output)
        .expect("failed to write the bindings");

    let eglplatform_output = dest.join("eglplatform_bindings.rs");
    println!("output file {:?}", eglplatform_output);
    build_eglplatform(angle_out_home, rust_target)
        .generate()?
        .write_to_file(&eglplatform_output)
        .expect("failed to write the bindings");

    let fallbacks = fallback.convert();

    let output = dest.join("egl_bindings.rs");
    println!("output file {:?}", output);
    let extensions = egl_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    pass_generator!(generator, gen_egl, output, egl_version, fallbacks, extensions)
        .expect("failed to write the bindings");

    let output = dest.join("gl_bindings.rs");
    println!("output file {:?}", output);
    let extensions = gles_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    pass_generator!(generator, gen_gles, output, gles_version, fallbacks, extensions)
        .expect("failed to write the bindings");

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
