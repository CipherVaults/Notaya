//! Keyfiles are local files, including USB. Never follow a network or device
//! name to find out whether it is a file. This module performs no network probe.
use crate::error::{Error, Result};
use std::{fs::File, path::Path};

pub(crate) const LOCAL_ONLY: &str =
    "Choose a keyfile on this PC or a USB drive. Network and device paths cannot be used.";
fn refused() -> Error {
    Error::invalid(LOCAL_ONLY)
}
fn unreadable() -> Error {
    Error::invalid("The keyfile could not be read.")
}

fn local_drive(mapping: &str, kind: u32) -> Result<()> {
    let lower = mapping.to_ascii_lowercase();
    const DRIVE_REMOTE: u32 = 4;
    if lower.contains("mup")
        || lower.contains("redirector")
        || lower.contains("unc\\")
        || kind == DRIVE_REMOTE
        || !matches!(kind, 2 | 3 | 5 | 6)
    {
        return Err(refused());
    }
    Ok(())
}
#[cfg(test)]
thread_local! { static OPENS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
pub(crate) fn reset_opens() {
    OPENS.set(0);
}
#[cfg(test)]
pub(crate) fn opens() -> usize {
    OPENS.get()
}

/// Pure lexical check, including Windows aliases on every platform. No stat,
/// canonicalize, open, read_link, or drive query precedes this check.
pub(crate) fn lexical(path: &str) -> Result<String> {
    let p = path.replace('/', "\\");
    let lower = p.to_ascii_lowercase();
    if p.is_empty()
        || p.chars().any(char::is_control)
        || p.starts_with("\\\\")
        || lower.starts_with("\\??\\")
        || lower.starts_with("\\device\\")
        || lower.starts_with("\\global??\\")
    {
        return Err(refused());
    }
    for (n, c) in p.chars().enumerate() {
        if matches!(c, '<' | '>' | '"' | '|' | '?' | '*')
            || (c == ':' && !(n == 1 && p.as_bytes()[0].is_ascii_alphabetic()))
        {
            return Err(refused());
        }
    }
    let rest = if p.as_bytes().get(1) == Some(&b':') {
        &p[2..]
    } else {
        &p
    };
    for part in rest.split('\\') {
        let part = part.trim_end_matches(['.', ' ']);
        let stem = part
            .split('.')
            .next()
            .unwrap_or("")
            .trim_end_matches(' ')
            .to_uppercase();
        if matches!(
            stem.as_str(),
            "CON" | "PRN" | "AUX" | "NUL" | "CLOCK$" | "CONIN$" | "CONOUT$"
        ) || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.chars().count() == 4
            && stem
                .chars()
                .last()
                .is_some_and(|c| matches!(c, '1'..='9' | '¹' | '²' | '³')))
        {
            return Err(refused());
        }
    }
    Ok(p)
}

#[cfg(windows)]
mod imp {
    use super::*;
    use std::{
        fs::OpenOptions,
        os::windows::fs::{MetadataExt, OpenOptionsExt},
        path::PathBuf,
    };
    use windows::{
        Win32::Storage::FileSystem::{
            FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
            FILE_SHARE_READ, GetDriveTypeW, QueryDosDeviceW,
        },
        core::PCWSTR,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }
    // Resolve SUBST aliases without accessing their target, then ask Windows for
    // the drive type. QueryDosDevice reads the DOS namespace, not the keyfile.
    fn local_absolute(path: &str, depth: usize) -> Result<PathBuf> {
        if depth > 16 {
            return Err(refused());
        }
        let path = lexical(path)?;
        let absolute = std::path::absolute(&path).map_err(|_| unreadable())?;
        let text = lexical(absolute.to_str().ok_or_else(refused)?)?;
        let bytes = text.as_bytes();
        if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || &bytes[1..3] != b":\\" {
            return Err(refused());
        }
        let drive = wide(&text[..2]);
        let mut mapping = vec![0u16; 32768];
        let n = unsafe { QueryDosDeviceW(PCWSTR(drive.as_ptr()), Some(&mut mapping)) };
        if n == 0 {
            return Err(unreadable());
        }
        let end = mapping.iter().position(|c| *c == 0).ok_or_else(refused)?;
        let mapping = String::from_utf16(&mapping[..end]).map_err(|_| refused())?;
        local_drive(&mapping, 3)?;
        if let Some(target) = mapping.strip_prefix("\\??\\") {
            // SUBST can alias a drive/subfolder, including a remote one.
            let bytes = target.as_bytes();
            if bytes.len() < 3 || !bytes[0].is_ascii_alphabetic() || &bytes[1..3] != b":\\" {
                return Err(refused());
            }
            return local_absolute(
                &format!("{}{}", target.trim_end_matches('\\'), &text[2..]),
                depth + 1,
            );
        }
        let root = wide(&text[..3]);
        let kind = unsafe { GetDriveTypeW(PCWSTR(root.as_ptr())) };
        local_drive(&mapping, kind)?;
        Ok(PathBuf::from(text))
    }

    fn link_target(path: &Path) -> Result<PathBuf> {
        // read_link opens the reparse point itself; it never opens its target.
        let target = std::fs::read_link(path).map_err(|_| unreadable())?;
        let raw = target.to_str().ok_or_else(refused)?;
        // Windows returns local junction destinations as \\?\C:\... . Only
        // this internally obtained drive form is unwrapped; UNC/Volume/device
        // forms still fail lexical(), before any target access.
        let raw = if raw.as_bytes().get(5) == Some(&b':')
            && raw.starts_with("\\\\?\\")
            && raw.as_bytes()[4].is_ascii_alphabetic()
        {
            &raw[4..]
        } else {
            raw
        };
        let checked = lexical(raw)?;
        let target = PathBuf::from(checked);
        Ok(if target.is_absolute() {
            target
        } else {
            path.parent().ok_or_else(refused)?.join(target)
        })
    }

    fn walk(path: &str, depth: usize, pins: &mut Vec<File>) -> Result<File> {
        if depth > 32 {
            return Err(refused());
        }
        let absolute = local_absolute(path, 0)?;
        let mut prefix = PathBuf::new();
        let parts: Vec<_> = absolute.components().collect();
        for (index, part) in parts.iter().enumerate() {
            prefix.push(part.as_os_str());
            if index == 0 {
                continue;
            } // C: is not a root until the next component.
            // Do not traverse reparse points. Denying write/delete sharing pins
            // every inspected ancestor against rename and reparse retargeting.
            #[cfg(test)]
            OPENS.set(OPENS.get() + 1);
            let file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ.0)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT.0 | FILE_FLAG_BACKUP_SEMANTICS.0)
                .open(&prefix)
                .map_err(|_| unreadable())?;
            let meta = file.metadata().map_err(|_| unreadable())?;
            if meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT.0 != 0 {
                pins.push(file);
                let mut target = link_target(&prefix)?;
                for remaining in parts.iter().skip(index + 1) {
                    target.push(remaining.as_os_str());
                }
                return walk(target.to_str().ok_or_else(refused)?, depth + 1, pins);
            }
            if index + 1 == parts.len() {
                if !meta.is_file() {
                    return Err(unreadable());
                }
                return Ok(file);
            }
            pins.push(file);
        }
        Err(unreadable())
    }
    pub(super) fn open(path: &Path) -> Result<File> {
        let text = lexical(path.to_str().ok_or_else(refused)?)?;
        walk(&text, 0, &mut Vec::new())
    }
}
#[cfg(not(windows))]
mod imp {
    use super::*;
    pub(super) fn open(path: &Path) -> Result<File> {
        lexical(path.to_str().ok_or_else(refused)?)?;
        File::open(path).map_err(|_| unreadable())
    }
}
pub(crate) fn open(path: &Path) -> Result<File> {
    imp::open(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_network_device_and_alias_forms_are_refused_before_io() {
        for path in [
            r"\\server\share\key",
            r"\\?\UNC\server\share\key",
            r"\\.\NUL",
            r"\\?\C:\key",
            "//server/share/key",
            r"\/server\share\key",
            r"\??\C:\key",
            "CON",
            "nul.txt",
            "COM1",
            "LPT9.log",
            "NUL.  ",
            "con .txt",
            r"C:\folder\COM1.txt",
            "COM¹",
            "CONIN$",
            "key:stream",
        ] {
            assert_eq!(lexical(path).unwrap_err().to_string(), LOCAL_ONLY, "{path}");
            assert_eq!(
                open(Path::new(path)).unwrap_err().to_string(),
                LOCAL_ONLY,
                "{path}"
            );
        }
        for path in [
            r"C:\folder\key",
            "D:/folder/key",
            "key",
            r"..\key",
            r"C:key",
            r"\folder\key",
            "ordinary. ",
            "COM10.key",
        ] {
            assert!(lexical(path).is_ok(), "{path}");
        }
    }
    #[test]
    fn drive_resolver_refuses_remote_and_unknown_but_keeps_fixed_and_usb() {
        for mapping in [
            r"\Device\Mup\share",
            r"\Device\LanmanRedirector\share",
            r"\??\UNC\server\share",
        ] {
            assert_eq!(local_drive(mapping, 3).unwrap_err().to_string(), LOCAL_ONLY);
        }
        for kind in [0, 1, 4] {
            assert!(local_drive(r"\Device\HarddiskVolume1", kind).is_err());
        }
        for kind in [2, 3, 5, 6] {
            assert!(local_drive(r"\Device\HarddiskVolume1", kind).is_ok());
        }
    }
    #[cfg(windows)]
    #[test]
    fn local_file_and_junction_work_and_junction_cycle_is_bounded() {
        let root =
            std::path::PathBuf::from(std::env::var_os("NOTAYA_HOME").expect("throwaway home"))
                .join(format!("sf2-keyfile-{}", crate::model::new_id()));
        let target = root.join("ordinary");
        std::fs::create_dir_all(&target).unwrap();
        let key = target.join("key.bin");
        let bytes = crate::crypto::random_bytes::<64>();
        std::fs::write(&key, bytes).unwrap();
        assert_eq!(
            *crate::crypto::keyfile_digest(&key).unwrap(),
            crate::crypto::sha256(&bytes)
        );
        let junction = root.join("junction");
        let made = std::process::Command::new("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&target)
            .output()
            .unwrap();
        assert!(made.status.success(), "could not create throwaway junction");
        assert_eq!(
            *crate::crypto::keyfile_digest(&junction.join("key.bin")).unwrap(),
            crate::crypto::sha256(&bytes)
        );
        let alias = target.join("key.bin. ");
        assert_eq!(
            *crate::crypto::keyfile_digest(&alias).unwrap(),
            crate::crypto::sha256(&bytes)
        );
        let cycle = root.join("cycle");
        let made = std::process::Command::new("cmd.exe")
            .args(["/D", "/C", "mklink", "/J"])
            .arg(&cycle)
            .arg(&root)
            .output()
            .unwrap();
        assert!(made.status.success());
        // Each reparse expansion is bounded, including an intentionally deep cycle.
        let mut path = cycle;
        for _ in 0..40 {
            path = path.join("cycle");
        }
        path = path.join("key.bin");
        assert!(crate::crypto::keyfile_digest(&path).is_err());
    }
}
