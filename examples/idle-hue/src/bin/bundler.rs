use std::fs;
use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let project_root = std::env::current_dir()?;

    println!("Project root: {}", project_root.display());

    println!("Building for Apple Silicon (ARM64)...");
    let arm_status = Command::new("cargo")
        .args(&["bundle", "--release", "--bin", "idle-hue"])
        .current_dir(&project_root)
        .status()?;

    if !arm_status.success() {
        eprintln!("Failed to build ARM64 bundle");
        std::process::exit(1);
    }

    println!("Building for Intel (x86_64)...");
    let intel_status = Command::new("cargo")
        .args(&[
            "bundle",
            "--release",
            "--target",
            "x86_64-apple-darwin",
            "--bin",
            "idle-hue",
        ])
        .current_dir(&project_root)
        .status()?;

    if !intel_status.success() {
        eprintln!("Failed to build Intel bundle");
        std::process::exit(1);
    }

    let arm_bundle_path = project_root.join("target/release/bundle/osx/idle-hue.app");
    let intel_bundle_path =
        project_root.join("target/x86_64-apple-darwin/release/bundle/osx/idle-hue.app");

    if !arm_bundle_path.exists() {
        eprintln!("ARM bundle not found at {:?}", arm_bundle_path);
        std::process::exit(1);
    }

    if !intel_bundle_path.exists() {
        eprintln!("Intel bundle not found at {:?}", intel_bundle_path);
        std::process::exit(1);
    }

    println!("Creating ARM64 zip...");
    let arm_zip_status = Command::new("zip")
        .args(&["-r", "idle-hue-macos-arm.zip", "idle-hue.app"])
        .current_dir(arm_bundle_path.parent().unwrap())
        .status()?;

    if !arm_zip_status.success() {
        eprintln!("Failed to create ARM64 zip");
        std::process::exit(1);
    }

    println!("Creating Intel zip...");
    let intel_zip_status = Command::new("zip")
        .args(&["-r", "idle-hue-macos-intel.zip", "idle-hue.app"])
        .current_dir(intel_bundle_path.parent().unwrap())
        .status()?;

    if !intel_zip_status.success() {
        eprintln!("Failed to create Intel zip");
        std::process::exit(1);
    }

    let arm_zip_src = arm_bundle_path
        .parent()
        .unwrap()
        .join("idle-hue-macos-arm.zip");
    let intel_zip_src = intel_bundle_path
        .parent()
        .unwrap()
        .join("idle-hue-macos-intel.zip");
    let arm_zip_dest = project_root.join("idle-hue-macos-arm.zip");
    let intel_zip_dest = project_root.join("idle-hue-macos-intel.zip");

    if arm_zip_src.exists() {
        fs::rename(&arm_zip_src, &arm_zip_dest)?;
        println!("Created: {}", arm_zip_dest.display());
    }

    if intel_zip_src.exists() {
        fs::rename(&intel_zip_src, &intel_zip_dest)?;
        println!("Created: {}", intel_zip_dest.display());
    }

    println!("Bundle process completed successfully!");
    Ok(())
}
