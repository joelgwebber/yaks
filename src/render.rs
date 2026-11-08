//! `yaks render` (yaks-8fd1): rasterize an SVG to a PNG with the fonts
//! embedded in the binary (toque's `png` feature), so an agent whose file tools
//! only read in-project paths can look at a docshot or an attached SVG without
//! a browser. It is the same rasterizer the TUI's image viewer uses, so the
//! PNG is what a human sees in `yaks tui`, on any machine.

use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};

/// Largest SVG rendered (as in the TUI's image viewer).
const SVG_MAX_BYTES: u64 = 4 << 20;

/// Where the SVG named `arg` is: a file path as given (relative to the
/// current directory) wins; else, with a farm, a farm path (`/doks/x.svg`,
/// `sources/<id>/shot.svg`) or an attachment (`<id>/<file>`).
pub fn resolve(arg: &str, cwd: &Path, farm_root: Option<&Path>) -> Result<PathBuf> {
    let direct = cwd.join(arg);
    if direct.is_file() {
        return Ok(direct);
    }
    if let Some(root) = farm_root {
        let rel = arg.trim_start_matches('/');
        let rel = rel.strip_prefix(".yaks/").unwrap_or(rel);
        let in_farm = root.join(rel);
        if in_farm.is_file() {
            return Ok(in_farm);
        }
        if let Some((id, name)) = rel.split_once('/')
            && let Some(found) = crate::farm::locate_attachment(root, id, name)
        {
            return Ok(found.path);
        }
    }
    bail!("{arg}: no such file, farm path or attachment")
}

/// The PNG file for `svg` when `-o` is not given: `<stem>.png` in `cwd`.
pub fn default_output(svg: &Path, cwd: &Path) -> PathBuf {
    let stem = svg
        .file_stem()
        .map_or("render".into(), |s| s.to_os_string());
    cwd.join(stem).with_extension("png")
}

/// The PNG bytes of the SVG at `path`, at `scale` times its size.
pub fn png_of(path: &Path, scale: f32) -> Result<Vec<u8>> {
    let len = std::fs::metadata(path)
        .with_context(|| format!("reading {}", path.display()))?
        .len();
    if len > SVG_MAX_BYTES {
        bail!(
            "{}: {len} bytes; the limit is {SVG_MAX_BYTES}",
            path.display()
        );
    }
    if !(scale.is_finite() && scale > 0.0) {
        bail!("--scale must be a positive number, not {scale}");
    }
    let svg = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    toque::svg_to_png_scaled(&svg, scale)
        .with_context(|| format!("{}: not an SVG this renderer can draw", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="40" height="20"><rect width="40" height="20" fill="#123456"/><text x="2" y="14" font-family="monospace">ab</text></svg>"##;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("yaks-render-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn a_path_a_farm_path_and_an_attachment_resolve() {
        let d = tmp("resolve");
        let cwd = d.join("code");
        let farm = d.join("farm");
        std::fs::create_dir_all(cwd.join("docs")).unwrap();
        std::fs::create_dir_all(farm.join("doks")).unwrap();
        std::fs::create_dir_all(farm.join("yaks/yaks/yaks-1a2b")).unwrap();
        std::fs::write(cwd.join("docs/a.svg"), SVG).unwrap();
        std::fs::write(farm.join("doks/b.svg"), SVG).unwrap();
        std::fs::write(
            farm.join("yaks/yaks/yaks-1a2b.md"),
            "---\nid: yaks-1a2b\n---\n",
        )
        .unwrap();
        std::fs::write(farm.join("yaks/yaks/yaks-1a2b/c.svg"), SVG).unwrap();
        let f = Some(farm.as_path());
        assert_eq!(
            resolve("docs/a.svg", &cwd, f).unwrap(),
            cwd.join("docs/a.svg")
        );
        assert_eq!(
            resolve("/doks/b.svg", &cwd, f).unwrap(),
            farm.join("doks/b.svg")
        );
        assert_eq!(
            resolve("yaks-1a2b/c.svg", &cwd, f).unwrap(),
            farm.join("yaks/yaks/yaks-1a2b/c.svg")
        );
        assert!(resolve("nope.svg", &cwd, f).is_err());
        assert!(
            resolve("/doks/b.svg", &cwd, None).is_err(),
            "farm paths need a farm"
        );
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn it_writes_a_png_at_the_scale_asked() {
        let d = tmp("png");
        let p = d.join("x.svg");
        std::fs::write(&p, SVG).unwrap();
        let png = png_of(&p, 2.0).unwrap();
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        // IHDR width/height, big-endian, at bytes 16..24.
        let w = u32::from_be_bytes(png[16..20].try_into().unwrap());
        let h = u32::from_be_bytes(png[20..24].try_into().unwrap());
        assert_eq!((w, h), (80, 40));
        assert!(png_of(&p, 0.0).is_err());
        std::fs::write(&p, "not svg").unwrap();
        assert!(png_of(&p, 1.0).is_err());
        assert_eq!(
            default_output(&p, Path::new("/w")),
            PathBuf::from("/w/x.png")
        );
        let _ = std::fs::remove_dir_all(&d);
    }
}
