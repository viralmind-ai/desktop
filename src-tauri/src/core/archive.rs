use std::fs;
use std::io;
use std::path::Path;

/// Extracts a file from a zip archive
pub fn extract_from_zip(
    archive_path: &Path,
    output_path: &Path,
    file_pattern: &str,
) -> Result<bool, String> {
    log::info!(
        "[Archive] Opening zip archive at {}",
        archive_path.display()
    );
    let file = fs::File::open(archive_path).map_err(|e| {
        log::info!("[Archive] Error: Failed to open zip: {}", e);
        format!("Failed to open zip: {}", e)
    })?;

    let mut archive = match zip::ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            log::info!("[Archive] Error: Failed to read zip: {}", e);
            return Err("RETRY_NEEDED".to_string());
        }
    };

    // Process the archive contents
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| {
            log::info!("[Archive] Error: Failed to read zip entry: {}", e);
            format!("Failed to read zip entry: {}", e)
        })?;

        let name = file.name();
        if name.contains(file_pattern) {
            log::info!("[Archive] Found matching file in zip: {}", name);
            let mut outfile = create_output_file(output_path)?;
            copy_file_data(&mut file, &mut outfile)?;
            return Ok(true);
        }
    }

    log::info!("[Archive] No file matching '{}' found in zip", file_pattern);
    Ok(false) // File not found
}

/// Extracts a file from a tar.xz archive
pub fn extract_from_tar_xz(
    archive_path: &Path,
    output_path: &Path,
    file_pattern: &str,
    exclude_pattern: Option<&str>,
) -> Result<bool, String> {
    log::info!(
        "[Archive] Opening tar.xz archive at {}",
        archive_path.display()
    );
    let file = fs::File::open(archive_path).map_err(|e| {
        log::info!("[Archive] Error: Failed to open tar.xz: {}", e);
        format!("Failed to open tar.xz: {}", e)
    })?;

    let tar = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(tar);

    let entries = match archive.entries() {
        Ok(entries) => entries,
        Err(e) => {
            log::info!("[Archive] Error: Failed to read tar entries: {}", e);
            return Err("RETRY_NEEDED".to_string());
        }
    };

    for entry_result in entries {
        let mut entry = entry_result.map_err(|e| {
            log::info!("[Archive] Error: Failed to read tar entry: {}", e);
            format!("Failed to read tar entry: {}", e)
        })?;

        let path = entry.path().map_err(|e| {
            log::info!("[Archive] Error: Failed to get entry path: {}", e);
            format!("Failed to get entry path: {}", e)
        })?;

        let path_str = path.to_string_lossy();

        // Check if the path matches our pattern and doesn't match the exclude pattern
        let is_match = path_str.contains(file_pattern)
            && (exclude_pattern.is_none() || !path_str.contains(exclude_pattern.unwrap()));

        if is_match {
            log::info!("[Archive] Found matching file in tar.xz: {}", path_str);
            let mut outfile = create_output_file(output_path)?;
            copy_file_data(&mut entry, &mut outfile)?;
            return Ok(true);
        }
    }

    log::info!(
        "[Archive] No file matching '{}' found in tar.xz",
        file_pattern
    );
    Ok(false) // File not found
}

/// Creates the output file for the binary
fn create_output_file(output_path: &Path) -> Result<fs::File, String> {
    fs::File::create(output_path).map_err(|e| {
        log::info!("[Archive] Error: Failed to create output file: {}", e);
        format!("Failed to create output file: {}", e)
    })
}

/// Copies data from source to destination
fn copy_file_data<R: io::Read, W: io::Write>(
    source: &mut R,
    destination: &mut W,
) -> Result<(), String> {
    io::copy(source, destination).map_err(|e| {
        log::info!("[Archive] Error: Failed to extract file: {}", e);
        format!("Failed to extract file: {}", e)
    })?;
    Ok(())
}

/// Makes a file executable on Unix systems
#[cfg(unix)]
pub fn make_file_executable(file_path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    log::info!(
        "[Archive] Setting executable permissions for {}",
        file_path.display()
    );
    fs::set_permissions(file_path, fs::Permissions::from_mode(0o755)).map_err(|e| {
        log::info!("[Archive] Error: Failed to make file executable: {}", e);
        format!("Failed to make file executable: {}", e)
    })
}

/// Deletes an archive file
pub fn cleanup_archive(archive_path: &Path) -> Result<(), String> {
    log::info!("[Archive] Cleaning up archive file");
    fs::remove_file(archive_path).map_err(|e| {
        log::info!("[Archive] Warning: Failed to cleanup archive: {}", e);
        format!("Failed to cleanup archive: {}", e)
    })
}
