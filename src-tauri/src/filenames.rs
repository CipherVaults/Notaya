//! The names of the vault files. Since V1.4.0 a vault is `vault.nota` and its backups are `*.nota`.
//! Up to V1.3.0 they were `vault.clf` and `*.clf`: those files are still read (and listed as
//! backups), but Notaya only writes new names. Keeping every name in one place makes the next
//! change a one-line edit.

/// The extension of every vault file Notaya writes (without the dot).
pub const EXT: &str = "nota";
/// The extension of the vault files of V1.3.0 and earlier (without the dot).
pub const OLD_EXT: &str = "clf";
/// The vault in the data folder.
pub const VAULT_FILE: &str = "vault.nota";
/// The vault in the data folder until V1.3.0.
pub const OLD_VAULT_FILE: &str = "vault.clf";
/// Where the V1.3.0 vault file is kept after Notaya made its `vault.nota` from it. Notaya never deletes it on its own; the only thing that removes it is the person's own choice to delete earlier unprotected copies when turning security on.
pub const BEFORE_NOTAYA_FILE: &str = "vault-before-notaya.clf";

/// The file name without a vault extension (`.nota` or `.clf`, any case), or `None`.
pub fn strip_vault_ext(name: &str) -> Option<&str> {
    let (stem, ext) = name.rsplit_once('.')?;
    (ext.eq_ignore_ascii_case(EXT) || ext.eq_ignore_ascii_case(OLD_EXT)).then_some(stem)
}

/// True for a file name that ends in `.nota` or `.clf`.
pub fn has_vault_ext(name: &str) -> bool {
    strip_vault_ext(name).is_some()
}

/// True when the path ends in `.nota` or `.clf` (any case).
pub fn path_has_vault_ext(path: &std::path::Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case(EXT) || e.eq_ignore_ascii_case(OLD_EXT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_extensions_are_vault_files() {
        assert_eq!(strip_vault_ext("vault-12.nota"), Some("vault-12"));
        assert_eq!(strip_vault_ext("vault-12.clf"), Some("vault-12"));
        assert_eq!(strip_vault_ext("vault-12.NOTA"), Some("vault-12"));
        assert_eq!(strip_vault_ext("vault-12.tmp"), None);
        assert_eq!(strip_vault_ext("vault-12.nota.bak"), None);
        assert_eq!(strip_vault_ext("nota"), None);
        assert!(has_vault_ext("a.clf") && !has_vault_ext("a.txt"));
        assert!(path_has_vault_ext(std::path::Path::new("E:\\x\\y.Nota")) && !path_has_vault_ext(std::path::Path::new("y.kdbx")));
    }
}
