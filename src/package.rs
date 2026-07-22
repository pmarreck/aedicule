//! Deterministic `.aed` packaging and capability-bounded application file lookup.
//!
//! The same virtual-root adapter backs bare WAT, source-directory, archive,
//! and browser delivery so packaging cannot change application-visible bytes.

use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, OpenOptions},
    io::Write,
    path::{Component, Path, PathBuf},
};

use async_zip::{
    Compression, ZipEntryBuilder, base::read::mem::ZipFileReader, base::write::ZipFileWriter,
};
use futures_lite::future::block_on;
use futures_lite::io::Cursor;
use unicode_normalization::UnicodeNormalization;

use crate::{ApplicationAssets, DEFAULT_PLUGIN_FILE, FALLBACK_WAT, PluginSource};

const MAX_PACKAGE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 16 * 1024 * 1024;
const MAX_ENTRIES: usize = 1_024;
pub const AED_MIME_TYPE: &str = "application/vnd.aedicule.app+zip";
const MIME_TYPE_ENTRY: &str = "mimetype";

struct PackageEntry {
    name: String,
    bytes: Vec<u8>,
}

/// Creates a byte-reproducible stored-ZIP `.aed` without mutating its source tree.
pub fn package_application(source: &Path, output: &Path) -> Result<(), String> {
    if !source.is_dir() {
        return Err(format!(
            "application source is not a directory: {}",
            source.display()
        ));
    }
    if !source.join(DEFAULT_PLUGIN_FILE).is_file() {
        return Err(format!(
            "application source lacks {DEFAULT_PLUGIN_FILE}: {}",
            source.display()
        ));
    }
    if output.exists() {
        return Err(format!(
            "package output already exists: {}",
            output.display()
        ));
    }

    let mut entries = Vec::new();
    collect_entries(source, source, &mut entries)?;
    entries.sort_by(|left, right| left.name.as_bytes().cmp(right.name.as_bytes()));
    if entries.len() > MAX_ENTRIES {
        return Err(format!("application has more than {MAX_ENTRIES} files"));
    }

    let archive = block_on(async {
        let mut writer = ZipFileWriter::new(Cursor::new(Vec::new())).force_no_zip64();
        writer
            .write_entry_whole(
                ZipEntryBuilder::new(MIME_TYPE_ENTRY.into(), Compression::Stored)
                    .unix_permissions(0o644),
                AED_MIME_TYPE.as_bytes(),
            )
            .await
            .map_err(|error| format!("write package mimetype: {error}"))?;
        for entry in entries {
            let descriptor = ZipEntryBuilder::new(entry.name.into(), Compression::Stored)
                .unix_permissions(0o644);
            writer
                .write_entry_whole(descriptor, &entry.bytes)
                .await
                .map_err(|error| format!("write package entry: {error}"))?;
        }
        writer
            .close()
            .await
            .map(Cursor::into_inner)
            .map_err(|error| format!("finish package: {error}"))
    })?;
    if archive.len() as u64 > MAX_PACKAGE_BYTES {
        return Err(format!("package exceeds {MAX_PACKAGE_BYTES} bytes"));
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(|error| format!("create {}: {error}", output.display()))?;
    file.write_all(&archive)
        .map_err(|error| format!("write {}: {error}", output.display()))
}

/// Safely expands a validated `.aed`, refusing to merge with existing content.
pub fn depackage_application(source: &Path, output: &Path) -> Result<(), String> {
    if output.exists() {
        return Err(format!(
            "depackage output already exists: {}",
            output.display()
        ));
    }
    let entries = read_archive_entries(source)?;
    if !entries
        .iter()
        .any(|entry| entry.name == DEFAULT_PLUGIN_FILE)
    {
        return Err(format!(
            "package lacks {DEFAULT_PLUGIN_FILE}: {}",
            source.display()
        ));
    }

    fs::create_dir(output).map_err(|error| format!("create {}: {error}", output.display()))?;
    for entry in entries {
        let destination = output.join(name_to_path(&entry.name)?);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)
                .map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&destination)
            .map_err(|error| format!("create {}: {error}", destination.display()))?;
        file.write_all(&entry.bytes)
            .map_err(|error| format!("write {}: {error}", destination.display()))?;
    }
    Ok(())
}

/// Reads one validated virtual path identically from WAT, directory, or `.aed` sources.
pub fn read_application_file(source: &PluginSource, name: &str) -> Result<Vec<u8>, String> {
    validate_name(name)?;
    match source {
        PluginSource::Embedded if name == DEFAULT_PLUGIN_FILE => {
            Ok(FALLBACK_WAT.as_bytes().to_vec())
        }
        PluginSource::Embedded => Err(format!("embedded application lacks {name}")),
        PluginSource::File(path) => {
            let candidate = if name == DEFAULT_PLUGIN_FILE {
                path.clone()
            } else {
                path.parent()
                    .unwrap_or_else(|| Path::new("."))
                    .join(name_to_path(name)?)
            };
            read_bounded_file(&candidate)
        }
        PluginSource::Directory(root) => read_bounded_file(&root.join(name_to_path(name)?)),
        PluginSource::Archive(path) => read_archive_entries(path)?
            .into_iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.bytes)
            .ok_or_else(|| format!("package {} lacks {name}", path.display())),
    }
}

/// Preloads only the bounded `assets/` capability subtree from any supported
/// application container, producing identical virtual names before WAT runs.
pub fn read_application_assets(source: &PluginSource) -> Result<ApplicationAssets, String> {
    match source {
        PluginSource::Embedded => Ok(BTreeMap::new()),
        PluginSource::File(path) => {
            read_directory_assets(path.parent().unwrap_or_else(|| Path::new(".")))
        }
        PluginSource::Directory(root) => read_directory_assets(root),
        PluginSource::Archive(path) => Ok(read_archive_entries(path)?
            .into_iter()
            .filter(|entry| is_valid_asset_name(&entry.name))
            .map(|entry| (entry.name, entry.bytes))
            .collect()),
    }
}

fn read_directory_assets(root: &Path) -> Result<ApplicationAssets, String> {
    let asset_root = root.join("assets");
    let metadata = match fs::symlink_metadata(&asset_root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(format!("inspect {}: {error}", asset_root.display())),
    };
    if metadata.file_type().is_symlink() {
        return Err(format!(
            "symbolic links are not asset entries: {}",
            asset_root.display()
        ));
    }
    if !metadata.is_dir() {
        return Err(format!(
            "application assets path is not a directory: {}",
            asset_root.display()
        ));
    }

    let mut assets = BTreeMap::new();
    let mut total_bytes = 0_u64;
    collect_asset_entries(root, &asset_root, &mut assets, &mut total_bytes)?;
    Ok(assets)
}

fn collect_asset_entries(
    root: &Path,
    directory: &Path,
    assets: &mut ApplicationAssets,
    total_bytes: &mut u64,
) -> Result<(), String> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| format!("read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read {}: {error}", directory.display()))?;
    children.sort_by_key(|entry| entry.file_name());
    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symbolic links are not asset entries: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_asset_entries(root, &path, assets, total_bytes)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!("unsupported asset entry type: {}", path.display()));
        }
        if metadata.len() > MAX_ENTRY_BYTES {
            return Err(format!(
                "asset entry exceeds {MAX_ENTRY_BYTES} bytes: {}",
                path.display()
            ));
        }
        *total_bytes = total_bytes
            .checked_add(metadata.len())
            .ok_or_else(|| "asset entry sizes overflow".to_owned())?;
        if *total_bytes > MAX_PACKAGE_BYTES {
            return Err(format!(
                "application assets exceed {MAX_PACKAGE_BYTES} bytes"
            ));
        }
        if assets.len() == MAX_ENTRIES {
            return Err(format!("application has more than {MAX_ENTRIES} assets"));
        }
        let name = path_to_name(
            path.strip_prefix(root)
                .expect("asset entry remains below application root"),
        )?;
        debug_assert!(is_valid_asset_name(&name));
        assets.insert(
            name,
            fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?,
        );
    }
    Ok(())
}

/// Discovers independently runnable WAST suites directly below `tests/`.
/// Nested WAST files remain available as composition fragments but are not
/// mistaken for standalone suites.
#[cfg(feature = "native-runtime")]
pub(crate) fn application_test_entries(source: &PluginSource) -> Result<Vec<String>, String> {
    let mut names = match source {
        PluginSource::Embedded => Vec::new(),
        PluginSource::File(path) => {
            directory_test_entries(path.parent().unwrap_or_else(|| Path::new(".")))?
        }
        PluginSource::Directory(root) => directory_test_entries(root)?,
        PluginSource::Archive(path) => read_archive_entries(path)?
            .into_iter()
            .filter_map(|entry| is_test_entry(&entry.name).then_some(entry.name))
            .collect(),
    };
    names.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(names)
}

#[cfg(feature = "native-runtime")]
fn directory_test_entries(root: &Path) -> Result<Vec<String>, String> {
    let directory = root.join("tests");
    let children = match fs::read_dir(&directory) {
        Ok(children) => children,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(format!("read {}: {error}", directory.display())),
    };
    let mut names = Vec::new();
    for child in children {
        let child = child.map_err(|error| format!("read {}: {error}", directory.display()))?;
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("inspect {}: {error}", path.display()))?;
        let name = path_to_name(
            path.strip_prefix(root)
                .expect("test entry remains below application root"),
        )?;
        if is_test_entry(&name) {
            if metadata.file_type().is_symlink() {
                return Err(format!(
                    "symbolic links are not test entries: {}",
                    path.display()
                ));
            }
            if metadata.is_file() {
                names.push(name);
            }
        }
    }
    Ok(names)
}

#[cfg(feature = "native-runtime")]
fn is_test_entry(name: &str) -> bool {
    let mut components = name.split('/');
    matches!(
        (components.next(), components.next(), components.next()),
        (Some("tests"), Some(file), None) if file.ends_with(".wast") && file.len() > ".wast".len()
    )
}

fn collect_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<PackageEntry>,
) -> Result<(), String> {
    let mut children = fs::read_dir(directory)
        .map_err(|error| format!("read {}: {error}", directory.display()))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| format!("read {}: {error}", directory.display()))?;
    children.sort_by_key(|entry| entry.file_name());

    for child in children {
        let path = child.path();
        let metadata = fs::symlink_metadata(&path)
            .map_err(|error| format!("inspect {}: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "symbolic links are not package entries: {}",
                path.display()
            ));
        }
        if metadata.is_dir() {
            collect_entries(root, &path, entries)?;
            continue;
        }
        if !metadata.is_file() {
            return Err(format!(
                "unsupported package entry type: {}",
                path.display()
            ));
        }
        if metadata.len() > MAX_ENTRY_BYTES {
            return Err(format!(
                "package entry exceeds {MAX_ENTRY_BYTES} bytes: {}",
                path.display()
            ));
        }
        if entries.len() == MAX_ENTRIES {
            return Err(format!("application has more than {MAX_ENTRIES} files"));
        }
        let name = path_to_name(
            path.strip_prefix(root)
                .expect("collected path remains below root"),
        )?;
        if name == MIME_TYPE_ENTRY {
            return Err(format!(
                "application source uses reserved package path: {MIME_TYPE_ENTRY}"
            ));
        }
        entries.push(PackageEntry {
            name,
            bytes: fs::read(&path).map_err(|error| format!("read {}: {error}", path.display()))?,
        });
    }
    Ok(())
}

fn read_archive_entries(path: &Path) -> Result<Vec<PackageEntry>, String> {
    let bytes = read_bounded_file_with_limit(path, MAX_PACKAGE_BYTES)?;
    block_on(async {
        let reader = ZipFileReader::new(bytes)
            .await
            .map_err(|error| format!("open package {}: {error}", path.display()))?;
        let stored = reader.file().entries();
        if stored.len() > MAX_ENTRIES + 1 {
            return Err(format!("package has more than {MAX_ENTRIES} entries"));
        }
        let Some(first) = stored.first() else {
            return Err(format!(
                "package {} lacks {MIME_TYPE_ENTRY}",
                path.display()
            ));
        };
        let first_name = first
            .filename()
            .as_str()
            .map_err(|error| format!("package {MIME_TYPE_ENTRY} name: {error}"))?;
        if first_name != MIME_TYPE_ENTRY || first.compression() != Compression::Stored {
            return Err(format!(
                "package {} must begin with stored {MIME_TYPE_ENTRY}",
                path.display()
            ));
        }
        let mut names = HashSet::new();
        let mut entries = Vec::with_capacity(stored.len());
        let mut total = 0_u64;
        for (index, descriptor) in stored.iter().enumerate() {
            let name = descriptor
                .filename()
                .as_str()
                .map_err(|error| format!("package entry {index} name: {error}"))?;
            validate_name(name)?;
            if index > 0 && name == MIME_TYPE_ENTRY {
                return Err(format!("duplicate package entry: {MIME_TYPE_ENTRY}"));
            }
            if name.ends_with('/') {
                continue;
            }
            if descriptor.compression() != Compression::Stored {
                return Err(format!(
                    "package entry uses unsupported compression: {name}"
                ));
            }
            if descriptor.uncompressed_size() > MAX_ENTRY_BYTES {
                return Err(format!(
                    "package entry exceeds {MAX_ENTRY_BYTES} bytes: {name}"
                ));
            }
            total = total
                .checked_add(descriptor.uncompressed_size())
                .ok_or_else(|| "package entry sizes overflow".to_owned())?;
            if total > MAX_PACKAGE_BYTES {
                return Err(format!("package contents exceed {MAX_PACKAGE_BYTES} bytes"));
            }
            if !names.insert(name.to_owned()) {
                return Err(format!("duplicate package entry: {name}"));
            }
            let mut entry_reader = reader
                .reader_with_entry(index)
                .await
                .map_err(|error| format!("open package entry {name}: {error}"))?;
            let mut entry_bytes = Vec::with_capacity(descriptor.uncompressed_size() as usize);
            entry_reader
                .read_to_end_checked(&mut entry_bytes)
                .await
                .map_err(|error| format!("read package entry {name}: {error}"))?;
            if index == 0 {
                if entry_bytes != AED_MIME_TYPE.as_bytes() {
                    return Err(format!(
                        "package {} has invalid {MIME_TYPE_ENTRY}",
                        path.display()
                    ));
                }
                continue;
            }
            entries.push(PackageEntry {
                name: name.to_owned(),
                bytes: entry_bytes,
            });
        }
        Ok(entries)
    })
}

fn read_bounded_file(path: &Path) -> Result<Vec<u8>, String> {
    read_bounded_file_with_limit(path, MAX_ENTRY_BYTES)
}

fn read_bounded_file_with_limit(path: &Path, limit: u64) -> Result<Vec<u8>, String> {
    let metadata =
        fs::metadata(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    if metadata.len() > limit {
        return Err(format!("file exceeds {limit} bytes: {}", path.display()));
    }
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn path_to_name(path: &Path) -> Result<String, String> {
    let mut components = Vec::new();
    for component in path.components() {
        let Component::Normal(component) = component else {
            return Err(format!("invalid package path: {}", path.display()));
        };
        components.push(
            component
                .to_str()
                .ok_or_else(|| format!("package path is not UTF-8: {}", path.display()))?,
        );
    }
    let name = components.join("/");
    validate_name(&name)?;
    Ok(name)
}

fn name_to_path(name: &str) -> Result<PathBuf, String> {
    validate_name(name)?;
    Ok(name.split('/').collect())
}

fn validate_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || name.contains(':')
        || name.contains('\0')
        || name
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(format!("invalid package path: {name:?}"));
    }
    if name.nfc().ne(name.chars()) {
        return Err(format!("package path is not NFC Unicode: {name:?}"));
    }
    Ok(())
}

pub(crate) fn is_valid_asset_name(name: &str) -> bool {
    name.strip_prefix("assets/")
        .is_some_and(|relative| !relative.is_empty())
        && validate_name(name).is_ok()
}
