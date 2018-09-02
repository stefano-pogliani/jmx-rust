// J4RS comes with multiple JAR files as dependencies but only injects one of them.
// To have all JARs available in the classpath until https://github.com/astonbitecode/j4rs/pull/3
// is merged (or if it is rjected) copy the fat JAR from this repo to the expected target location.
use std::env;
use std::fs::File;
use std::fs::copy;
use std::path::PathBuf;

static JAR_SOURCE: &'static str = "./j4rs-0.1.4-jar-with-dependencies.jar";
static JAR_TARGET: &'static str = "./j4rs-0.1.4.jar";


fn main() {
    copy_fat_jar();
}

fn copy_fat_jar() {
    if File::open(JAR_SOURCE).is_ok() {
        let cargo_target = env::var("OUT_DIR").expect("Cargo target directory not set");
        let mut cargo_target = PathBuf::from(cargo_target);
        cargo_target.pop();
        cargo_target.pop();
        cargo_target.pop();
        cargo_target.push("jassets");
        cargo_target.push(JAR_TARGET);
        let cargo_target = cargo_target.to_str().unwrap().to_owned();
        copy(JAR_SOURCE, cargo_target).expect("Failed to copy archive");
    }
}
