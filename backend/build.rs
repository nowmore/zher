use std::process::Command;

fn main() {
    let frontend_dir = "../frontend";

    println!("cargo:rerun-if-changed={}/src", frontend_dir);
    println!("cargo:rerun-if-changed={}/package.json", frontend_dir);
    println!("cargo:rerun-if-changed={}/vite.config.js", frontend_dir);

    let dist_path = format!("{}/dist", frontend_dir);
    let dist_exists = std::path::Path::new(&dist_path).exists();

    if !dist_exists {
        println!("cargo:warning=Frontend dist not found, building...");
        build_frontend(frontend_dir);
    } else {
        let index_html = format!("{}/index.html", dist_path);
        if !std::path::Path::new(&index_html).exists() {
            println!("cargo:warning=Frontend dist incomplete, rebuilding...");
            build_frontend(frontend_dir);
        }
    }

    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("icons/icon.ico");
        res.compile().unwrap();
    }
}

fn build_frontend(frontend_dir: &str) {
    let npm_cmd = if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    };

    let install_status = Command::new(npm_cmd)
        .args(["install"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm install");

    if !install_status.success() {
        panic!("npm install failed");
    }

    let build_status = Command::new(npm_cmd)
        .args(["run", "build"])
        .current_dir(frontend_dir)
        .status()
        .expect("Failed to run npm build");

    if !build_status.success() {
        panic!("npm build failed");
    }

    println!("cargo:warning=Frontend built successfully");
}
