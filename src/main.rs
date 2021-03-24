use ::{
    bindgen::{self, RustTarget},
    gl_generator::{Api, Fallbacks, Profile, Registry},
    serde::Deserialize,
    std::{env, fs::File, ops::Deref, path::Path},
};

fn binding(target: Option<RustTarget>) -> bindgen::Builder {
    target.map_or_else(bindgen::builder, |target| bindgen::builder().rust_target(target))
}

fn coerce_generate_and_write(bindings: Result<bindgen::Bindings, ()>, output: &Path) {
    bindings
        .expect("generate the bindings failed")
        .write_to_file(output)
        .expect("write the bindings failed");
}

fn gen_khrplatform(output: &Path, angle_out_home: &Path, rust_target: Option<RustTarget>) {
    println!("output file {:?}", output);
    let khrplatform = angle_out_home
        .join("../")
        .join("include")
        .join("KHR")
        .join("khrplatform.h");
    coerce_generate_and_write(
        binding(rust_target)
            .header(khrplatform.to_str().expect("expect UTF-8"))
            .whitelist_type("khronos_utime_nanoseconds_t")
            .whitelist_type("khronos_uint64_t")
            .whitelist_type("khronos_ssize_t")
            .generate(),
        output,
    );
}

fn gen_eglplatform(output: &Path, angle_out_home: &Path, rust_target: Option<RustTarget>) {
    println!("output file {:?}", output);
    let inclide_dir = angle_out_home.join("../").join("include");
    let eglplatform = inclide_dir.join("EGL").join("eglplatform.h");
    let mut inclide_search = String::from("-I");
    inclide_search.push_str(inclide_dir.to_str().expect("expect UTF-8"));
    println!("inclide_search: {}", inclide_search);
    coerce_generate_and_write(
        binding(rust_target)
            .clang_args(&[inclide_search])
            .header(eglplatform.to_str().expect("expect UTF-8"))
            .whitelist_type("EGLNativeDisplayType")
            .whitelist_type("EGLNativePixmapType")
            .whitelist_type("EGLNativeWindowType")
            .whitelist_type("EGLint")
            .whitelist_type("NativeDisplayType")
            .whitelist_type("NativePixmapType")
            .whitelist_type("NativeWindowType")
            .generate(),
        output,
    );
}

/// EGL1.4のバインディングを生成する。
fn gen_egl<'a, EXT>(output: &Path, fallbacks: Fallbacks, extensions: EXT)
where
    EXT: AsRef<[&'a str]>,
{
    println!("output file {:?}", output);
    let mut output = File::create(output).unwrap();
    Registry::new(Api::Egl, (1, 4), Profile::Core, fallbacks, extensions)
        .write_bindings(gl_generator::GlobalGenerator, &mut output)
        .unwrap();
}

/// OpenGLES `version`のバインディングを生成する。
fn gen_gles<'a, EXT>(output: &Path, version: (u8, u8), fallbacks: Fallbacks, extensions: EXT)
where
    EXT: AsRef<[&'a str]>,
{
    println!("output file {:?}", output);
    let mut output = File::create(output).unwrap();
    Registry::new(Api::Gles2, version, Profile::Core, fallbacks, extensions)
        .write_bindings(gl_generator::GlobalGenerator, &mut output)
        .unwrap();
}

#[derive(Eq, PartialEq, Deserialize)]
enum Fallback {
    All,
    None,
}

impl Fallback {
    fn convert(self) -> Fallbacks {
        if self == Fallback::All {
            Fallbacks::All
        } else {
            Fallbacks::None
        }
    }
}

#[derive(Deserialize)]
struct Config {
    rust_version: Box<str>,
    angle_out_home: Box<str>,
    fallback: Fallback,
    egl_extensions: Vec<Box<str>>,
    gles_version: Box<str>,
    gles_extensions: Vec<Box<str>>,
    dest: Box<str>,
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

/// バージョン文字列を変換する。
/// 対応するバージョンがない場合`3.1`とする
fn convert_gles_version(gles_version: Box<str>) -> (u8, u8) {
    match &*gles_version {
        "2.0" => (2, 0),
        "3.0" => (3, 0),
        "3.1" => (3, 1),
        _ => (3, 1),
    }
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
    gen_khrplatform(&dest.join("khrplatform_bindings.rs"), angle_out_home, rust_target);
    gen_eglplatform(&dest.join("eglplatform_bindings.rs"), angle_out_home, rust_target);

    let fallbacks = fallback.convert();

    let egl_extensions = egl_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    gen_egl(&dest.join("egl_bindings.rs"), fallbacks, egl_extensions);

    let gles_extensions = gles_extensions.iter().map(Box::deref).collect::<Vec<_>>();
    let gles_version = convert_gles_version(gles_version);
    gen_gles(&dest.join("gl_bindings.rs"), gles_version, fallbacks, gles_extensions);
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match env::args().nth(1).map(File::open) {
        Some(Ok(config)) => present_config_file_path(config),
        Some(Err(err)) => Err(err.into()),
        None => Ok(println!("usage: angle_gles_generator <config_file_path.ron>")),
    }
}
