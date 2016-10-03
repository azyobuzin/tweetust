extern crate serde_codegen;

use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    serde_expand(&out_dir);
    model_list(&out_dir);
}

fn serde_expand(out_dir: &OsString) {
    struct Env<'a> { src_dir: PathBuf, out_dir: &'a Path }

    fn visit_dir(env: &Env, current_dir: &Path) {
        if let Ok(entries) = fs::read_dir(current_dir) {
            for path in entries.filter_map(|x| x.ok()).map(|x| x.path()) {
                if path.is_dir() { visit_dir(env, &path) }
                else { visit_file(env, &path) }
            }
        }
    }

    fn visit_file(env: &Env, file_path: &Path) {
        if !file_path.to_str().unwrap().ends_with(".in.rs") { return; }

        let dst_path = {
            let slice = file_path.strip_prefix(&env.src_dir).unwrap().to_str().unwrap();
            let mut buf = String::with_capacity(slice.len() - 3);
            buf.push_str(&slice[..slice.len() - 6]);
            buf.push_str(".rs");
            env.out_dir.join(buf)
        };

        if let Some(parent) = dst_path.parent() { fs::create_dir_all(parent).unwrap() }
        serde_codegen::expand(file_path, dst_path).unwrap();
    }

    let env = Env { src_dir: fs::canonicalize("./src/").unwrap(), out_dir: Path::new(out_dir) };
    visit_dir(&env, &env.src_dir);
}

fn model_list(out_dir: &OsString) {
    let models_dir = Path::new(out_dir).join("models");
    let mut dst_file = fs::File::create(models_dir.join("_models_list.rs")).unwrap();

    for path in fs::read_dir(models_dir).unwrap()
        .filter_map(|x| x.ok().map(|y| y.path()))
        .filter(|x| match x.extension() { Some(x) => x == "rs", None => false })
    {
        if let Some(file_name) = path.file_name().and_then(|x| x.to_str()) {
            if file_name.starts_with('_') { continue; }
            writeln!(&mut dst_file, r#"include!(concat!(env!("OUT_DIR"), "/models/{}"));"#, file_name).unwrap();
        }
    }
}
