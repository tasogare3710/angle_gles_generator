use gl_generator::Generator;

use ::{
    bindgen::{self, Builder, RustTarget},
    gl_generator::{Api, Fallbacks, Profile, Registry},
    std::{fs::File, io, path::Path},
};

fn build_with(target: Option<RustTarget>) -> bindgen::Builder {
    target.map_or_else(bindgen::builder, |target| bindgen::builder().rust_target(target))
}

pub fn build_khrplatform(angle_out_home: &Path, rust_target: Option<RustTarget>) -> Builder {
    let khrplatform = angle_out_home
        .join("../")
        .join("include")
        .join("KHR")
        .join("khrplatform.h");
    let khrplatform = khrplatform.to_str().expect("expect UTF-8");
    build_with(rust_target)
        .header(khrplatform)
        .allowlist_type("khronos_utime_nanoseconds_t")
        .allowlist_type("khronos_uint64_t")
        .allowlist_type("khronos_ssize_t")
}

pub fn build_eglplatform(angle_out_home: &Path, rust_target: Option<RustTarget>) -> Builder {
    let include_dir = angle_out_home.join("../").join("include");
    let eglplatform = include_dir.join("EGL").join("eglplatform.h");
    let eglplatform = eglplatform.to_str().expect("expect UTF-8");

    let mut include_search = String::from("-I");
    include_search.push_str(include_dir.to_str().expect("expect UTF-8"));
    build_with(rust_target)
        .clang_args(&[include_search])
        .header(eglplatform)
        .allowlist_type("EGLNativeDisplayType")
        .allowlist_type("EGLNativePixmapType")
        .allowlist_type("EGLNativeWindowType")
        .allowlist_type("EGLint")
        .allowlist_type("NativeDisplayType")
        .allowlist_type("NativePixmapType")
        .allowlist_type("NativeWindowType")
}

/// EGL `version`のバインディングを生成する。
pub fn gen_egl<'a, E, G: Generator>(output: &Path, version: (u8, u8), fallbacks: Fallbacks, extensions: E, generator: G) -> io::Result<()>
where
    E: AsRef<[&'a str]>,
{
    let mut output = File::create(output)?;
    Registry::new(Api::Egl, version, Profile::Core, fallbacks, extensions)
        .write_bindings(generator, &mut output)
}

/// OpenGLES `version`のバインディングを生成する。
pub fn gen_gles<'a, E, G: Generator>(output: &Path, version: (u8, u8), fallbacks: Fallbacks, extensions: E, generator: G) -> io::Result<()>
where
    E: AsRef<[&'a str]>,
{
    let mut output = File::create(output)?;
    Registry::new(Api::Gles2, version, Profile::Core, fallbacks, extensions)
        .write_bindings(generator, &mut output)
}
