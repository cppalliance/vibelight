//! Builds source link text and targets from crate-relative paths and the output directory.

use std::ffi::{OsStr, OsString};
use std::io;
use std::iter;
use std::path::{self, Component, Path, Prefix};

/// The two absolute directories every source link is computed from.
///
/// Both are made absolute and lexically normalized once, when the value is
/// built. Names keep the spelling they were given.
#[derive(Debug, Clone)]
pub(crate) struct LinkBase {
    crate_dir: AbsPath,
    out_dir: AbsPath,
}

impl LinkBase {
    /// Makes both directories absolute against the current directory and
    /// resolves their `.` and `..` segments.
    ///
    /// # Errors
    ///
    /// Returns the error from [`path::absolute`], as when a path is empty or
    /// the current directory cannot be read.
    pub(crate) fn new(crate_dir: &Path, out_dir: &Path) -> io::Result<LinkBase> {
        // `fs::canonicalize` would add the `\\?\` prefix on Windows and follow symlinks.
        Ok(LinkBase {
            crate_dir: split(&path::absolute(crate_dir)?),
            out_dir: split(&path::absolute(out_dir)?),
        })
    }

    /// Returns the link text for the crate-relative path `rel`, such as
    /// `src/prompt.rs:136`, or `src/prompt.rs` when there is no line.
    ///
    /// The path is written with forward slashes and its `.` and `..`
    /// segments resolved. The text is never percent-encoded.
    #[expect(
        clippy::unused_self,
        reason = "a method, so text and target are both read from the one base"
    )]
    pub(crate) fn text(&self, rel: &str, line: Option<u32>) -> String {
        let path = rel_names(rel).join("/");
        match line {
            Some(line) => format!("{path}:{line}"),
            None => path,
        }
    }

    /// Returns the link target for the crate-relative path `rel`, such as
    /// `../src/prompt.rs#L136`, or `../src/prompt.rs` when there is no line.
    ///
    /// The path is relative to the output directory and percent-encoded.
    /// When no relative path exists, as between two Windows drives or
    /// shares, it is an absolute `file:` URL instead.
    pub(crate) fn target(&self, rel: &str, line: Option<u32>) -> String {
        let mut file = self.crate_dir.clone();
        for name in rel_names(rel) {
            if name == ".." {
                file.names.pop();
            } else {
                file.names.push(OsString::from(name));
            }
        }
        let path = relative(&self.out_dir, &file).unwrap_or_else(|| file_url(&file));
        match line {
            Some(line) => format!("{path}#L{line}"),
            None => path,
        }
    }

    /// Returns the crate directory from the output directory, such as
    /// `../mycrate`, or `.` for the output directory itself, with forward
    /// slashes and no encoding. When no relative path exists, it is the
    /// absolute path, such as `C:/Users/x/crate`.
    pub(crate) fn root(&self) -> String {
        let join = |steps: &[&[u8]]| String::from_utf8_lossy(&steps.join(&b'/')).into_owned();
        if let Some(steps) = steps(&self.out_dir, &self.crate_dir) {
            return if steps.is_empty() {
                ".".to_owned()
            } else {
                join(&steps)
            };
        }
        let names: Vec<&[u8]> = (self.crate_dir.names.iter())
            .map(|n| n.as_encoded_bytes())
            .collect();
        let path = join(&names);
        match &self.crate_dir.root {
            Root::Unix => format!("/{path}"),
            Root::Disk(letter) => format!("{}:/{path}", char::from(*letter)),
            Root::Unc(server, share) => {
                format!(
                    "//{}/{}/{path}",
                    server.to_string_lossy(),
                    share.to_string_lossy()
                )
            }
        }
    }
}

/// An absolute path split into its root and its names, with no `.` or `..`
/// segments left.
#[derive(Debug, Clone, PartialEq, Eq)]
struct AbsPath {
    root: Root,
    names: Vec<OsString>,
}

/// Where an absolute path starts, with the Windows extended-length forms
/// folded into the plain ones.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Root {
    /// The one root of a Unix file system, `/`.
    Unix,
    /// A Windows drive letter as written, as in `C:\`.
    Disk(u8),
    /// A Windows share, as in `\\server\share\`.
    Unc(OsString, OsString),
}

impl Root {
    /// Returns the root a Windows path prefix names.
    fn from_prefix(prefix: Prefix<'_>) -> Root {
        match prefix {
            Prefix::Disk(letter) | Prefix::VerbatimDisk(letter) => Root::Disk(letter),
            Prefix::UNC(server, share) | Prefix::VerbatimUNC(server, share) => {
                Root::Unc(server.to_owned(), share.to_owned())
            }
            // `\\.\name` and `\\?\name` name one device; it reads as share `name` on server `.`.
            Prefix::DeviceNS(name) | Prefix::Verbatim(name) => {
                Root::Unc(OsString::from("."), name.to_owned())
            }
        }
    }

    /// Reports whether two roots are the same place. Windows ignores case.
    fn matches(&self, other: &Root) -> bool {
        match (self, other) {
            (Root::Unix, Root::Unix) => true,
            (Root::Disk(letter), Root::Disk(other_letter)) => {
                letter.eq_ignore_ascii_case(other_letter)
            }
            (Root::Unc(server, share), Root::Unc(other_server, other_share)) => {
                eq_ignore_case(server, other_server) && eq_ignore_case(share, other_share)
            }
            _ => false,
        }
    }

    /// Reports whether two names under this root are the same name. Windows
    /// ignores case and Unix does not.
    fn same_name(&self, left: &OsStr, right: &OsStr) -> bool {
        match self {
            Root::Unix => left == right,
            Root::Disk(_) | Root::Unc(..) => eq_ignore_case(left, right),
        }
    }
}

/// Compares two names ignoring case, as Windows drives and shares do.
fn eq_ignore_case(left: &OsStr, right: &OsStr) -> bool {
    match (left.to_str(), right.to_str()) {
        (Some(left), Some(right)) => left
            .chars()
            .flat_map(char::to_lowercase)
            .eq(right.chars().flat_map(char::to_lowercase)),
        _ => left
            .as_encoded_bytes()
            .eq_ignore_ascii_case(right.as_encoded_bytes()),
    }
}

/// Splits an absolute path into its root and names, resolving `.` and `..`
/// lexically. A `..` at the root stays at the root.
fn split(path: &Path) -> AbsPath {
    let mut root = Root::Unix;
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => root = Root::from_prefix(prefix.kind()),
            Component::RootDir | Component::CurDir => {}
            Component::ParentDir => {
                names.pop();
            }
            Component::Normal(name) => names.push(name.to_owned()),
        }
    }
    AbsPath { root, names }
}

/// Splits a crate-relative path into names, resolving `.` and `..`
/// lexically. Each `..` that climbs out of the crate stays at the front.
fn rel_names(rel: &str) -> Vec<&str> {
    let mut names: Vec<&str> = Vec::new();
    for name in rel.split(path::is_separator) {
        match name {
            "" | "." => {}
            ".." if names.last().is_some_and(|last| *last != "..") => {
                names.pop();
            }
            _ => names.push(name),
        }
    }
    names
}

/// Returns the names that lead from the directory `dir` to `file`, or
/// `None` when the two do not share a root.
fn steps<'f>(dir: &AbsPath, file: &'f AbsPath) -> Option<Vec<&'f [u8]>> {
    if !dir.root.matches(&file.root) {
        return None;
    }
    let common = dir
        .names
        .iter()
        .zip(&file.names)
        .take_while(|(from, to)| dir.root.same_name(from, to))
        .count();
    let ups = iter::repeat_n(b"..".as_slice(), dir.names.len() - common);
    let downs = file.names[common..]
        .iter()
        .map(|name| name.as_encoded_bytes());
    Some(ups.chain(downs).collect())
}

/// Returns the percent-encoded path of `file` relative to the directory
/// `dir`, or `None` when the two do not share a root.
fn relative(dir: &AbsPath, file: &AbsPath) -> Option<String> {
    let steps = steps(dir, file)?;
    let path = encode(&steps.join(&b'/'));
    Some(match steps.first() {
        None => ".".to_owned(),
        // A first segment holding a colon would read as a URL scheme.
        Some(first) if first.contains(&b':') => format!("./{path}"),
        Some(_) => path,
    })
}

/// Returns `file` as an absolute, percent-encoded `file:` URL.
fn file_url(file: &AbsPath) -> String {
    let names: Vec<&[u8]> = file
        .names
        .iter()
        .map(|name| name.as_encoded_bytes())
        .collect();
    let path = encode(&names.join(&b'/'));
    match &file.root {
        Root::Unix => format!("file:///{path}"),
        Root::Disk(letter) => format!("file:///{}:/{path}", char::from(*letter)),
        Root::Unc(server, share) => {
            let server = encode(server.as_encoded_bytes());
            let share = encode(share.as_encoded_bytes());
            format!("file://{server}/{share}/{path}")
        }
    }
}

/// Percent-encodes every byte except the ASCII letters and digits and
/// `-._~/!$&'*+,;=:@`, which can stand as is in a URL path, an HTML `href`,
/// and a JSON string. Hex digits are uppercase.
fn encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut out = String::with_capacity(bytes.len());
    for &byte in bytes {
        if byte.is_ascii_alphanumeric() || b"-._~/!$&'*+,;=:@".contains(&byte) {
            out.push(char::from(byte));
        } else {
            out.push('%');
            out.push(char::from(HEX[usize::from(byte >> 4)]));
            out.push(char::from(HEX[usize::from(byte & 0x0F)]));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds an absolute path from a root and `/`-separated names.
    fn abs(root: Root, path: &str) -> AbsPath {
        let names = path.split('/').filter(|name| !name.is_empty());
        AbsPath {
            root,
            names: names.map(OsString::from).collect(),
        }
    }

    /// Builds a base whose two directories sit under the Unix root.
    fn unix(crate_dir: &str, out_dir: &str) -> LinkBase {
        LinkBase {
            crate_dir: abs(Root::Unix, crate_dir),
            out_dir: abs(Root::Unix, out_dir),
        }
    }

    /// Returns the root of a Windows share.
    fn unc(server: &str, share: &str) -> Root {
        Root::Unc(OsString::from(server), OsString::from(share))
    }

    /// The crate-relative file most tests link to.
    const REL: &str = "src/prompt.rs";

    #[test]
    fn out_dir_inside_beside_equal_to_and_above_the_crate() {
        for (out_dir, expected, root) in [
            ("w/mycrate/out", "../src/prompt.rs#L136", ".."),
            ("w/docs", "../mycrate/src/prompt.rs#L136", "../mycrate"),
            ("w/mycrate", "src/prompt.rs#L136", "."),
            ("w", "mycrate/src/prompt.rs#L136", "mycrate"),
        ] {
            let base = unix("w/mycrate", out_dir);
            assert_eq!(base.target(REL, Some(136)), expected, "out dir {out_dir}");
            assert_eq!(base.root(), root, "out dir {out_dir}");
        }
    }

    #[test]
    fn line_and_whole_file_forms() {
        let base = unix("w/mycrate", "w/mycrate/out");
        assert_eq!(base.text(REL, Some(136)), "src/prompt.rs:136");
        assert_eq!(base.text(REL, None), "src/prompt.rs");
        assert_eq!(base.target(REL, Some(136)), "../src/prompt.rs#L136");
        assert_eq!(base.target(REL, None), "../src/prompt.rs");
    }

    #[test]
    fn text_uses_forward_slashes() {
        let base = unix("w/mycrate", "w");
        assert_eq!(base.text("src/a/b.rs", Some(7)), "src/a/b.rs:7");
        // A backslash separates names only on Windows; on Unix it is part of a name.
        let expected = if cfg!(windows) {
            "src/a/b.rs:7"
        } else {
            r"src\a\b.rs:7"
        };
        assert_eq!(base.text(r"src\a\b.rs", Some(7)), expected);
    }

    #[test]
    fn rel_dot_segments_are_resolved() {
        let base = unix("w/mycrate", "w/mycrate/out");
        assert_eq!(base.text("./src/../src//x.rs", Some(1)), "src/x.rs:1");
        assert_eq!(base.target("./src/../src//x.rs", Some(1)), "../src/x.rs#L1");
        assert_eq!(base.text("src/../../shared/x.rs", None), "../shared/x.rs");
        assert_eq!(
            base.target("src/../../shared/x.rs", None),
            "../../shared/x.rs"
        );
    }

    #[test]
    fn split_resolves_dot_segments() {
        let path = Path::new("/w/./mycrate/src/../out/");
        assert_eq!(split(path), abs(Root::Unix, "w/mycrate/out"));
        assert_eq!(split(Path::new("/../w")), abs(Root::Unix, "w"));
    }

    #[test]
    fn new_makes_relative_inputs_absolute_and_normalized() {
        let crate_dir = Path::new("w/./mycrate/src/..");
        let base = LinkBase::new(crate_dir, Path::new("w/mycrate/../docs/.")).unwrap();
        assert_eq!(base.target(REL, Some(1)), "../mycrate/src/prompt.rs#L1");
    }

    #[test]
    fn different_drives_give_a_file_url() {
        let base = LinkBase {
            crate_dir: abs(Root::Disk(b'C'), "Users/x/crate"),
            out_dir: abs(Root::Disk(b'D'), "out"),
        };
        let url = "file:///C:/Users/x/crate/src/prompt.rs";
        assert_eq!(base.target(REL, Some(136)), format!("{url}#L136"));
        assert_eq!(base.target(REL, None), url);
        assert_eq!(base.text(REL, Some(136)), "src/prompt.rs:136");
        assert_eq!(base.root(), "C:/Users/x/crate");
    }

    #[test]
    fn different_shares_give_a_file_url() {
        let url = "file://server/share/crate/src/prompt.rs#L1";
        for out_dir in [
            abs(unc("server", "other"), "out"),
            abs(Root::Disk(b'C'), "out"),
        ] {
            let base = LinkBase {
                crate_dir: abs(unc("server", "share"), "crate"),
                out_dir,
            };
            assert_eq!(base.target(REL, Some(1)), url);
            assert_eq!(base.root(), "//server/share/crate");
        }
    }

    #[test]
    fn windows_roots_ignore_case_and_keep_the_spelling() {
        let base = LinkBase {
            crate_dir: abs(Root::Disk(b'c'), "Users/\u{c4}rger/MyCrate"),
            out_dir: abs(Root::Disk(b'C'), "users/\u{e4}rger/out"),
        };
        assert_eq!(base.target(REL, Some(1)), "../MyCrate/src/prompt.rs#L1");
        let base = LinkBase {
            crate_dir: abs(unc("Server", "Share"), "MyCrate"),
            out_dir: abs(unc("server", "share"), "out"),
        };
        assert_eq!(base.target(REL, Some(1)), "../MyCrate/src/prompt.rs#L1");
    }

    #[test]
    fn unix_root_compares_names_exactly() {
        let base = unix("W/mycrate", "w/out");
        assert_eq!(
            base.target(REL, Some(1)),
            "../../W/mycrate/src/prompt.rs#L1"
        );
    }

    #[test]
    fn target_is_percent_encoded_and_text_is_not() {
        let base = unix("w/my crate (old)", "w/out");
        assert_eq!(base.root(), "../my crate (old)");
        let target = "../my%20crate%20%28old%29/src/a%23b%25c.rs#L2";
        assert_eq!(base.target("src/a#b%c.rs", Some(2)), target);
        assert_eq!(base.text("src/a#b%c.rs", Some(2)), "src/a#b%c.rs:2");
        let base = unix("w/caf\u{e9}", "w");
        assert_eq!(base.target("\u{fc}.rs", None), "caf%C3%A9/%C3%BC.rs");
    }

    #[test]
    fn encode_keeps_only_the_safe_bytes() {
        assert_eq!(encode(b"AZaz09-._~/!$&'*+,;=:@"), "AZaz09-._~/!$&'*+,;=:@");
        let escaped = "%20%22%23%25%3C%3E%3F%5B%5C%5D%5E%60%7B%7C%7D";
        assert_eq!(encode(b" \"#%<>?[\\]^`{|}"), escaped);
        assert_eq!(encode(&[0x00, 0x7F, 0xFF]), "%00%7F%FF");
    }

    #[test]
    fn a_colon_in_the_first_segment_gets_a_dot_slash() {
        assert_eq!(unix("w/a:b", "w").target("x.rs", None), "./a:b/x.rs");
        assert_eq!(unix("w/a:b", "w/out").target("x.rs", None), "../a:b/x.rs");
    }

    #[test]
    fn a_path_to_the_out_dir_itself_is_a_dot() {
        assert_eq!(unix("w", "w").target(".", None), ".");
    }

    #[cfg(windows)]
    #[test]
    fn extended_length_prefix_never_appears() {
        let target = |crate_dir: &str, out_dir: &str| {
            let base = LinkBase::new(Path::new(crate_dir), Path::new(out_dir)).unwrap();
            base.target(REL, Some(1))
        };
        let relative = "../mycrate/src/prompt.rs#L1";
        assert_eq!(target(r"\\?\C:\w\mycrate", r"C:\w\out"), relative);
        assert_eq!(target(r"C:\w\mycrate", r"\\?\c:\W\out"), relative);
        let unc_crate = r"\\?\UNC\server\share\mycrate";
        assert_eq!(target(unc_crate, r"\\server\share\out"), relative);
        let url = "file:///C:/w/mycrate/src/prompt.rs#L1";
        assert_eq!(target(r"\\?\C:\w\mycrate", r"D:\out"), url);
        let url = "file://server/share/mycrate/src/prompt.rs#L1";
        assert_eq!(target(unc_crate, r"C:\out"), url);
    }

    #[cfg(windows)]
    #[test]
    fn verbatim_dot_segments_are_resolved() {
        let path = Path::new(r"\\?\C:\w\.\mycrate\src\..");
        assert_eq!(split(path), abs(Root::Disk(b'C'), "w/mycrate"));
        let path = Path::new(r"\\?\UNC\server\share\a\..\b");
        assert_eq!(split(path), abs(unc("server", "share"), "b"));
    }

    #[cfg(windows)]
    #[test]
    fn windows_paths_ignore_case() {
        let crate_dir = Path::new(r"c:\Users\X\MyCrate");
        let base = LinkBase::new(crate_dir, Path::new(r"C:\users\x\out")).unwrap();
        assert_eq!(base.target(REL, Some(1)), "../MyCrate/src/prompt.rs#L1");
    }

    #[cfg(windows)]
    #[test]
    fn device_paths_read_as_shares_on_server_dot() {
        let file = split(Path::new(r"\\?\Volume{1}\mycrate\x.rs"));
        assert_eq!(file, abs(unc(".", "Volume{1}"), "mycrate/x.rs"));
        assert_eq!(split(Path::new(r"\\.\Volume{1}\mycrate")).root, file.root);
        assert_eq!(file_url(&file), "file://./Volume%7B1%7D/mycrate/x.rs");
    }
}
