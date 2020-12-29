use ::{
    //  bindgen::RustTarget,
    gl_generator::{Api, Fallbacks, Profile, Registry},
    std::{env, fs::File, path::Path},
};

fn gen_khrplatform(file: &Path, angle_out_home: &Path) {
    println!("output file {:?}", file);
    let khrplatform = angle_out_home
        .join("../")
        .join("include")
        .join("KHR")
        .join("khrplatform.h");
    let bindings = bindgen::builder()
        .header(khrplatform.to_str().unwrap())
        .whitelist_type("khronos_utime_nanoseconds_t")
        .whitelist_type("khronos_uint64_t")
        .whitelist_type("khronos_ssize_t")
        .generate()
        .unwrap();
    bindings.write_to_file(file).unwrap();
}

fn gen_eglplatform(file: &Path, angle_out_home: &Path) {
    println!("output file {:?}", file);
    let inclide_dir = angle_out_home.join("../").join("include");
    let eglplatform = inclide_dir.join("EGL").join("eglplatform.h");
    let mut inclide_search = String::from("-I");
    inclide_search.push_str(inclide_dir.to_str().unwrap());
    println!("inclide_search: {}", inclide_search);
    let bindings = bindgen::builder().clang_args(&[inclide_search])
        .header(eglplatform.to_str().unwrap())
        // .rust_target(RustTarget::Nightly)
        // .detect_include_paths(true)
        // .rustfmt_bindings(false)
        .whitelist_type("EGLNativeDisplayType")
        .whitelist_type("EGLNativePixmapType")
        .whitelist_type("EGLNativeWindowType")
        .whitelist_type("EGLint")
        .whitelist_type("NativeDisplayType")
        .whitelist_type("NativePixmapType")
        .whitelist_type("NativeWindowType")
        .generate().unwrap();
    bindings.write_to_file(file).unwrap();
}

/// EGL1.5を生成
fn gen_egl(file: &Path) {
    println!("output file {:?}", file);
    let mut file = File::create(file).unwrap();
    Registry::new(Api::Egl, (1, 5), Profile::Core, Fallbacks::All, [])
        .write_bindings(gl_generator::GlobalGenerator, &mut file)
        .unwrap();
}

fn gen_gles31(file: &Path) {
    println!("output file {:?}", file);
    let mut file = File::create(file).unwrap();
    Registry::new(Api::Gles2, (3, 1), Profile::Core, Fallbacks::All, ["GL_KHR_debug"])
        .write_bindings(gl_generator::GlobalGenerator, &mut file)
        .unwrap();
}

fn main() {
    if let Some(arg) = env::args().nth(1) {
        let dest = Path::new(&arg);

        // XXX: khrplatform.hとeglplatform.hの参照にANGLEに依存しないようにしたい
        let angle_out_home = env::var("ANGLE_OUT_PATH").expect("`ANGLE_OUT_PATH` not found");
        let angle_out_home = Path::new(angle_out_home.as_str());

        gen_khrplatform(&dest.join("khrplatform_bindings.rs"), angle_out_home);
        gen_eglplatform(&dest.join("eglplatform_bindings.rs"), angle_out_home);
        gen_egl(&dest.join("egl_bindings.rs"));
        gen_gles31(&dest.join("gl_bindings.rs"));
    } else {
        eprintln!("usage: angle_gles_generator <output_dir>");
    }
}
