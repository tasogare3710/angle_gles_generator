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
        .whitelist_type("khronos_utime_nanoseconds_t")
        .whitelist_type("khronos_uint64_t")
        .whitelist_type("khronos_ssize_t")
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
        .whitelist_type("EGLNativeDisplayType")
        .whitelist_type("EGLNativePixmapType")
        .whitelist_type("EGLNativeWindowType")
        .whitelist_type("EGLint")
        .whitelist_type("NativeDisplayType")
        .whitelist_type("NativePixmapType")
        .whitelist_type("NativeWindowType")
}

/// EGL1.4のバインディングを生成する。
pub fn gen_egl<'a, E>(output: &Path, fallbacks: Fallbacks, extensions: E) -> io::Result<()>
where
    E: AsRef<[&'a str]>,
{
    let mut output = File::create(output)?;
    Registry::new(Api::Egl, (1, 4), Profile::Core, fallbacks, extensions)
        .write_bindings(gl_generator::GlobalGenerator, &mut output)
}

/// OpenGLES `version`のバインディングを生成する。
pub fn gen_gles<'a, E>(output: &Path, version: (u8, u8), fallbacks: Fallbacks, extensions: E) -> io::Result<()>
where
    E: AsRef<[&'a str]>,
{
    let mut output = File::create(output)?;
    Registry::new(Api::Gles2, version, Profile::Core, fallbacks, extensions)
        .write_bindings(gl_generator::GlobalGenerator, &mut output)
}
