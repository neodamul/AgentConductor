use std::{
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use zip::{CompressionMethod, DateTime, ZipWriter, write::SimpleFileOptions};

pub fn archive_package(package: &Path) -> Result<PathBuf, String> {
    let package = package
        .canonicalize()
        .map_err(|error| format!("resolve package directory: {error}"))?;
    if !package.is_dir() {
        return Err("release package must be a directory".into());
    }
    let archive = package.with_extension("zip");
    let pending = package.with_extension("zip.pending");
    if archive.exists() || pending.exists() {
        return Err("refusing to overwrite release archive".into());
    }

    let result = write_archive(&package, &pending, &archive);
    if result.is_err() {
        let _ = fs::remove_file(&pending);
    }
    result.map(|()| archive)
}

fn write_archive(package: &Path, pending: &Path, archive: &Path) -> Result<(), String> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(pending)
        .map_err(|error| format!("create pending archive: {error}"))?;
    let mut writer = ZipWriter::new(file);
    let mut files = Vec::new();
    collect_files(package, &mut files)?;
    files.sort();
    let root = package
        .parent()
        .ok_or_else(|| "package directory has no parent".to_owned())?;

    for path in files {
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(format!(
                "package contains a symbolic link or special file: {}",
                path.display()
            ));
        }
        let name = path
            .strip_prefix(root)
            .map_err(|error| error.to_string())?
            .to_str()
            .ok_or_else(|| "package path is not valid UTF-8".to_owned())?
            .replace('\\', "/");
        let modified = metadata.modified().unwrap_or(UNIX_EPOCH);
        let mut options = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .last_modified_time(zip_time(modified));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            options = options.external_attributes((metadata.permissions().mode() as u32) << 16);
        }
        writer
            .start_file(name, options)
            .map_err(|error| format!("add package file to archive: {error}"))?;
        io::copy(
            &mut File::open(&path).map_err(|error| error.to_string())?,
            &mut writer,
        )
        .map_err(|error| format!("write package archive entry: {error}"))?;
    }

    writer
        .finish()
        .map_err(|error| format!("finish release archive: {error}"))?
        .sync_all()
        .map_err(|error| format!("sync release archive: {error}"))?;
    fs::rename(pending, archive).map_err(|error| format!("publish release archive: {error}"))
}

fn collect_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let kind = fs::symlink_metadata(&path)
            .map_err(|error| error.to_string())?
            .file_type();
        if kind.is_symlink() {
            return Err(format!(
                "package contains a symbolic link: {}",
                path.display()
            ));
        }
        if kind.is_dir() {
            collect_files(&path, files)?;
        } else if kind.is_file() {
            files.push(path);
        } else {
            return Err(format!(
                "package contains a special file: {}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn zip_time(time: SystemTime) -> DateTime {
    let value = time::OffsetDateTime::from(time);
    let value = if value.year() < 1980 {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(1980, time::Month::January, 1).unwrap(),
            time::Time::MIDNIGHT,
        )
    } else if value.year() > 2107 {
        time::PrimitiveDateTime::new(
            time::Date::from_calendar_date(2107, time::Month::December, 31).unwrap(),
            time::Time::from_hms(23, 59, 58).unwrap(),
        )
    } else {
        time::PrimitiveDateTime::new(value.date(), value.time())
    };
    DateTime::try_from(value).unwrap_or_default()
}
