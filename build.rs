use std::env;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::str;

struct PackagePaths {
    dsp_source: PathBuf,
    dsp_include: PathBuf,
    dsp_private_include: PathBuf,
    core_include: PathBuf,
}

const CMSIS_CORE: &'static str = "lib/ARM.CMSIS.5.9.0/CMSIS/Core";
const CMSIS_DSP: &'static str = "lib/ARM.CMSIS-DSP.1.15.0";

fn build(cmsis: &PackagePaths) {
    let common_tables = cmsis.dsp_source.join("CommonTables/CommonTables.c");
    let support_functions = cmsis.dsp_source.join("SupportFunctions/SupportFunctions.c");
    let basic_math_functions = cmsis
        .dsp_source
        .join("BasicMathFunctions/BasicMathFunctions.c");
    let fast_math_functions = cmsis
        .dsp_source
        .join("FastMathFunctions/FastMathFunctions.c");
    let filter_functions = cmsis
        .dsp_source
        .join("FilteringFunctions/FilteringFunctions.c");

    cc::Build::new()
        .include(cmsis.dsp_include.as_path())
        .include(cmsis.dsp_private_include.as_path())
        .include(cmsis.core_include.as_path())
        .file(common_tables)
        .file(support_functions)
        .file(basic_math_functions)
        .file(fast_math_functions)
        .file(filter_functions)
        .opt_level_str("fast")
        .compile("cmsis_dsp");
}

fn bindgen(cmsis: &PackagePaths) {
    // Tell cargo to invalidate the built crate whenever the wrapper changes
    let sysroot = {
        let output = Command::new("arm-none-eabi-gcc")
            .args(["-print-sysroot"])
            .output()
            .expect("failed to get sysroot");
        str::from_utf8(&output.stdout).unwrap().trim().to_owned()
    };

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    let bindings = bindgen::Builder::default()
        .use_core()
        .clang_arg(format!(
            "--include-directory={}",
            cmsis.dsp_include.to_str().unwrap()
        ))
        .clang_arg(format!(
            "--include-directory={}",
            cmsis.core_include.to_str().unwrap()
        ))
        .clang_arg(format!("--sysroot={}", sysroot))
        // The input header we would like to generate
        // bindings for.
        .header(
            cmsis
                .dsp_include
                .join("./dsp/filtering_functions.h")
                .to_str()
                .unwrap(),
        )
        .header(
            cmsis
                .dsp_include
                .join("./dsp/support_functions.h")
                .to_str()
                .unwrap(),
        )
        .allowlist_function("arm_fir_.*")
        .allowlist_function("arm_float_.*")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}

fn main() {
    let cmsis_core: &Path = Path::new(CMSIS_CORE);
    let cmsis_dsp: &Path = Path::new(CMSIS_DSP);

    let cmsis = PackagePaths {
        dsp_source: cmsis_dsp.join("Source"),
        dsp_include: cmsis_dsp.join("Include"),
        dsp_private_include: cmsis_dsp.join("PrivateInclude"),
        core_include: cmsis_core.join("Include"),
    };

    build(&cmsis);
    bindgen(&cmsis);
}
