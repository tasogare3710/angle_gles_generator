use ::{
    angle_gles_generator::{build_eglplatform, build_khrplatform, gen_egl, gen_gles},
    bindgen::{Bindings, RustTarget},
    gl_generator::Fallbacks,
    serde::Deserialize,
    std::{env, fs::File, ops::Deref, path::Path},
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
    pub egl_extensions: Vec<Box<str>>,
    pub gles_version: (u8, u8),
    pub gles_extensions: Vec<Box<str>>,
    pub dest: Box<str>,
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

fn present_config_file_path(config: File) -> Result<(), Box<dyn std::error::Error>> {
    let Config {
        rust_version,
        angle_out_home,
        fallback,
        egl_extensions,
        gles_version,
        gles_extensions,
        dest,
    } = ron::de::from_reader(std::io::BufReader::new(config))?;

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
    gen_egl(&egl_output, fallbacks, egl_extensions)?;

    let gles_output = dest.join("gl_bindings.rs");
    println!("output file {:?}", gles_output);
    let gles_extensions = gles_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    gen_gles(&gles_output, gles_version, fallbacks, gles_extensions)?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::args().nth(1).map(File::open) {
        Some(Ok(config)) => present_config_file_path(config),
        Some(Err(err)) => Err(err.into()),
        None => Ok(println!("usage: angle_gles_generator <config_file_path.ron>")),
    }
}
