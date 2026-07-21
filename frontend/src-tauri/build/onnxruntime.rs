use flate2::read::GzDecoder;
use std::io::Cursor;
use std::path::{Path, PathBuf};

const ONNXRUNTIME_VERSION: &str = "1.24.2";

struct Distribution {
    archive_name: String,
    library_name: String,
    bundled_name: &'static str,
    is_zip: bool,
}

/// Download the official Microsoft ONNX Runtime shared library used by
/// Parakeet and Silero. `ort` is configured for dynamic loading so its build
/// never needs to contact the blocked Pyke CDN.
pub fn ensure_onnxruntime_library() {
    let target = std::env::var("TARGET")
        .or_else(|_| std::env::var("HOST"))
        .expect("Neither TARGET nor HOST environment variable is set");
    let distribution = distribution_for_target(&target)
        .unwrap_or_else(|error| panic!("ONNX Runtime setup failed: {error}"));

    let manifest_dir = PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR")
            .expect("CARGO_MANIFEST_DIR environment variable is not set"),
    );
    let output_dir = manifest_dir.join("binaries").join("onnxruntime");
    let output_path = output_dir.join(distribution.bundled_name);

    println!("cargo:rerun-if-env-changed=ONNXRUNTIME_FORCE_DOWNLOAD");
    println!("cargo:warning=Checking ONNX Runtime for target: {target}");

    let force_download = std::env::var_os("ONNXRUNTIME_FORCE_DOWNLOAD").is_some();
    if !force_download && is_valid_cached_library(&output_path) {
        println!(
            "cargo:warning=ONNX Runtime {} is already cached",
            ONNXRUNTIME_VERSION
        );
        return;
    }

    std::fs::create_dir_all(&output_dir)
        .expect("Failed to create the ONNX Runtime binaries directory");

    let url = format!(
        "https://github.com/microsoft/onnxruntime/releases/download/v{}/{}",
        ONNXRUNTIME_VERSION, distribution.archive_name
    );
    println!("cargo:warning=Downloading ONNX Runtime from {url}");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .expect("Failed to create the ONNX Runtime download client");
    let response = client
        .get(&url)
        .send()
        .unwrap_or_else(|error| panic!("Failed to download ONNX Runtime: {error}"));
    if !response.status().is_success() {
        panic!(
            "Failed to download ONNX Runtime from {}: HTTP {}",
            url,
            response.status()
        );
    }
    let archive = response
        .bytes()
        .expect("Failed to read the ONNX Runtime archive");

    if distribution.is_zip {
        extract_from_zip(
            &archive,
            &distribution.library_name,
            &output_path,
        )
        .unwrap_or_else(|error| panic!("Failed to extract ONNX Runtime: {error}"));
    } else {
        extract_from_tgz(
            &archive,
            &distribution.library_name,
            &output_path,
        )
        .unwrap_or_else(|error| panic!("Failed to extract ONNX Runtime: {error}"));
    }

    if !is_valid_cached_library(&output_path) {
        panic!(
            "Extracted ONNX Runtime library is missing or unexpectedly small: {}",
            output_path.display()
        );
    }
    println!(
        "cargo:warning=ONNX Runtime {} ready at {}",
        ONNXRUNTIME_VERSION,
        output_path.display()
    );
}

fn distribution_for_target(target: &str) -> Result<Distribution, String> {
    let (platform, library_name, bundled_name, is_zip) = if target == "aarch64-apple-darwin" {
        (
            "osx-arm64",
            format!("libonnxruntime.{}.dylib", ONNXRUNTIME_VERSION),
            "libonnxruntime.dylib",
            false,
        )
    } else if target == "x86_64-pc-windows-msvc" {
        ("win-x64", "onnxruntime.dll".to_string(), "onnxruntime.dll", true)
    } else if target == "aarch64-pc-windows-msvc" {
        ("win-arm64", "onnxruntime.dll".to_string(), "onnxruntime.dll", true)
    } else if target == "x86_64-unknown-linux-gnu" {
        (
            "linux-x64",
            format!("libonnxruntime.so.{}", ONNXRUNTIME_VERSION),
            "libonnxruntime.so",
            false,
        )
    } else if target == "aarch64-unknown-linux-gnu" {
        (
            "linux-aarch64",
            format!("libonnxruntime.so.{}", ONNXRUNTIME_VERSION),
            "libonnxruntime.so",
            false,
        )
    } else {
        return Err(format!("unsupported build target '{target}'"));
    };

    let archive_extension = if is_zip { "zip" } else { "tgz" };
    Ok(Distribution {
        archive_name: format!(
            "onnxruntime-{}-{}.{}",
            platform, ONNXRUNTIME_VERSION, archive_extension
        ),
        library_name,
        bundled_name,
        is_zip,
    })
}

fn is_valid_cached_library(path: &Path) -> bool {
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.len() > 1_000_000)
        .unwrap_or(false)
}

fn extract_from_zip(
    archive: &[u8],
    library_name: &str,
    output_path: &Path,
) -> Result<(), String> {
    let reader = Cursor::new(archive);
    let mut zip = zip::ZipArchive::new(reader).map_err(|error| error.to_string())?;
    for index in 0..zip.len() {
        let mut entry = zip.by_index(index).map_err(|error| error.to_string())?;
        if Path::new(entry.name()).file_name().and_then(|name| name.to_str())
            == Some(library_name)
        {
            let mut output = std::fs::File::create(output_path)
                .map_err(|error| error.to_string())?;
            std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
            return Ok(());
        }
    }
    Err(format!("'{library_name}' was not found in the ZIP archive"))
}

fn extract_from_tgz(
    archive: &[u8],
    library_name: &str,
    output_path: &Path,
) -> Result<(), String> {
    let decoder = GzDecoder::new(Cursor::new(archive));
    let mut tar = tar::Archive::new(decoder);
    let entries = tar.entries().map_err(|error| error.to_string())?;
    for entry in entries {
        let mut entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path().map_err(|error| error.to_string())?;
        if path.file_name().and_then(|name| name.to_str()) == Some(library_name) {
            let mut output = std::fs::File::create(output_path)
                .map_err(|error| error.to_string())?;
            std::io::copy(&mut entry, &mut output).map_err(|error| error.to_string())?;
            return Ok(());
        }
    }
    Err(format!("'{library_name}' was not found in the TGZ archive"))
}
