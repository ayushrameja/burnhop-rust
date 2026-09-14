//! Client-only, bounded cosmetic catalog and transactional local persistence.
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const LABELS: [&str; 8] = [
    "Outfit",
    "Skin",
    "Hair",
    "Hair color",
    "Headgear",
    "Headgear color",
    "Top color",
    "Trouser color",
];
pub const OUTFITS: &[&str] = &["approved", "base", "field", "scout"];
pub const SKINS: &[&str] = &["porcelain", "light", "tan", "warm", "brown", "deep"];
pub const HAIR: &[&str] = &["none", "buzz", "crop", "swept"];
pub const HAIR_COLORS: &[&str] = &["black", "brown", "chestnut", "blond", "grey", "white"];
pub const HEADGEAR: &[&str] = &["none", "helmet", "cap", "beret"];
pub const CLOTHES: &[&str] = &[
    "olive", "sand", "slate", "rust", "navy", "forest", "charcoal", "cream",
];
pub const CATALOG: [&[&str]; 8] = [
    OUTFITS,
    SKINS,
    HAIR,
    HAIR_COLORS,
    HEADGEAR,
    CLOTHES,
    CLOTHES,
    CLOTHES,
];
const KEYS: [&str; 8] = [
    "outfit",
    "skin",
    "hair",
    "hair_color",
    "headgear",
    "headgear_color",
    "top_color",
    "trouser_color",
];
pub const SKIN_RGB: [[u32; 3]; 6] = [
    [0xeed1ad, 0xffdfbd, 0xbb8e73],
    [0xe3bc90, 0xf7d4a3, 0xb68961],
    [0xcc9a6c, 0xe9b782, 0x9a6c4c],
    [0xb98055, 0xdba374, 0x865a3d],
    [0x946542, 0xb78459, 0x674631],
    [0x6b4936, 0x93684a, 0x493629],
];
pub const HAIR_RGB: [[u32; 2]; 6] = [
    [0x30342d, 0x535649],
    [0x5b4431, 0x806046],
    [0x874d32, 0xaf754c],
    [0xb39555, 0xdcc580],
    [0x838b7f, 0xb1b8a5],
    [0xcbd0b9, 0xecebd3],
];
pub const CLOTH_RGB: [[u32; 3]; 8] = [
    [0x849465, 0xa7b180, 0x586648],
    [0xc99b65, 0xe0be88, 0x8b6b48],
    [0x688994, 0x8ba9ae, 0x405f6c],
    [0xb96e50, 0xd9966d, 0x784d3e],
    [0x4e6b84, 0x7995aa, 0x35475b],
    [0x4d7563, 0x7f9a7b, 0x355345],
    [0x414e4b, 0x74817a, 0x2b3634],
    [0xcfccb1, 0xe4dfc5, 0x96947c],
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Appearance([usize; 8]);
impl Default for Appearance {
    fn default() -> Self {
        Self([0, 1, 2, 1, 2, 0, 0, 5])
    }
}
impl Appearance {
    pub fn index(self, field: usize) -> usize {
        self.0[field]
    }
    pub fn label(self, field: usize) -> &'static str {
        CATALOG[field][self.0[field]]
    }
    pub fn cycle(&mut self, field: usize, backwards: bool) {
        let count = CATALOG[field].len();
        self.0[field] = (self.0[field] + if backwards { count - 1 } else { 1 }) % count;
        if field == 0 {
            let identity = [self.0[1], self.0[2], self.0[3]];
            *self = Self::preset(self.0[0]);
            // Active web outfits preserve personal skin/hair selections. The
            // explicitly named approved preset restores the exact native original.
            if self.0[0] != 0 {
                self.0[1..4].copy_from_slice(&identity);
            }
        }
    }
    pub fn preset(index: usize) -> Self {
        match index {
            1 => Self([1, 1, 2, 1, 0, 0, 0, 5]),
            2 => Self([2, 1, 2, 1, 1, 0, 0, 0]),
            3 => Self([3, 2, 3, 2, 2, 1, 1, 1]),
            _ => Self::default(),
        }
    }
    pub fn baseline(self) -> bool {
        self == Self::default()
    }
    fn encode(self) -> String {
        let mut text = "burnhop-appearance=1\n".to_string();
        for (i, key) in KEYS.iter().enumerate() {
            text.push_str(&format!("{key}={}\n", self.label(i)));
        }
        text
    }
    fn decode(text: &str) -> Result<Self, String> {
        let mut lines = text.lines();
        if lines.next() != Some("burnhop-appearance=1") {
            return Err("Unsupported appearance version/header".into());
        }
        let mut result = Self::default();
        let mut seen = [false; 8];
        for line in lines {
            let (key, value) = line.split_once('=').ok_or("Malformed appearance entry")?;
            let field = KEYS
                .iter()
                .position(|k| *k == key)
                .ok_or("Unknown appearance field")?;
            if seen[field] {
                return Err("Duplicate appearance field".into());
            }
            result.0[field] = CATALOG[field]
                .iter()
                .position(|v| *v == value)
                .ok_or_else(|| format!("Invalid {key} option/color"))?;
            seen[field] = true;
        }
        if !seen.into_iter().all(|v| v) {
            return Err("Incomplete appearance".into());
        }
        Ok(result)
    }
}
pub fn config_path() -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let base = std::env::var_os("APPDATA").map(PathBuf::from);
    #[cfg(target_os = "macos")]
    let base =
        std::env::var_os("HOME").map(|p| PathBuf::from(p).join("Library/Application Support"));
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".config")));
    base.filter(|p| p.is_absolute())
        .map(|p| p.join("Burnhop/appearance.conf"))
        .ok_or("Application data directory unavailable".into())
}
pub fn load(path: &Path) -> Result<Appearance, String> {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Appearance::default()),
        Err(e) => return Err(e.to_string()),
    };
    let mut bytes = Vec::new();
    file.take(4097)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > 4096 {
        return Err("Appearance file exceeds 4096 bytes".into());
    }
    Appearance::decode(std::str::from_utf8(&bytes).map_err(|_| "Appearance is not UTF-8")?)
}
// Same-directory atomic replacement. Windows std::fs::rename replaces files using
// MoveFileExW(REPLACE_EXISTING); a failed replacement leaves the prior file intact.
pub fn save(path: &Path, value: Appearance) -> Result<(), String> {
    let parent = path.parent().ok_or("Missing config directory")?;
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let mut pending = None;
    for serial in 0..32 {
        let temp = parent.join(format!(".appearance-{}-{serial}.tmp", std::process::id()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        match options.open(&temp) {
            Ok(file) => {
                pending = Some((temp, file));
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e.to_string()),
        }
    }
    let (temp, mut file) = pending.ok_or("Temporary save slots busy; retry")?;
    let result = (|| -> std::io::Result<()> {
        file.write_all(value.encode().as_bytes())?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result.map_err(|e| e.to_string())
}
pub struct Editor {
    pub saved: Appearance,
    pub draft: Appearance,
    pub field: usize,
    pub pose: usize,
    pub left: bool,
    pub message: String,
    pub path: Result<PathBuf, String>,
}
impl Default for Editor {
    fn default() -> Self {
        Self {
            saved: Appearance::default(),
            draft: Appearance::default(),
            field: 0,
            pose: 0,
            left: false,
            message: String::new(),
            path: Err("No save location configured".into()),
        }
    }
}
impl Editor {
    pub fn startup() -> Self {
        let path = config_path();
        let loaded = path.as_ref().map_err(Clone::clone).and_then(|p| load(p));
        let mut s = Self {
            path,
            ..Self::default()
        };
        match loaded {
            Ok(a) => s.saved = a,
            Err(e) => s.message = format!("Could not load appearance: {e}. Using approved pilot."),
        }
        s.draft = s.saved;
        s
    }
    pub fn begin(&mut self) {
        self.draft = self.saved;
        self.field = 0;
    }
    pub fn cancel(&mut self) {
        self.draft = self.saved;
    }
    pub fn defaults(&mut self) {
        self.draft = Appearance::default();
        self.message = "Defaults previewed. Apply to save; Cancel to discard.".into();
    }
    pub fn apply(&mut self) -> bool {
        match self
            .path
            .as_ref()
            .map_err(Clone::clone)
            .and_then(|p| save(p, self.draft))
        {
            Ok(()) => {
                self.saved = self.draft;
                self.message = "Saved. Offline practice is ready.".into();
                true
            }
            Err(e) => {
                self.message = format!("Save failed: {e}. Previous appearance kept; retry Apply.");
                false
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn temp() -> PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static N: AtomicUsize = AtomicUsize::new(0);
        std::env::temp_dir().join(format!(
            "burnhop-appearance-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ))
    }
    #[test]
    fn catalog_roundtrips_and_rejects_invalid_colors() {
        for (field, options) in CATALOG.iter().enumerate() {
            let mut a = Appearance::default();
            for _ in *options {
                a.cycle(field, false);
                assert_eq!(Appearance::decode(&a.encode()).unwrap(), a);
            }
        }
        let text = Appearance::default().encode();
        for bad in [
            text.replace("=1\n", "=2\n"),
            text.replace("skin=light", "skin=unknown"),
            text.replace("top_color=olive", "top_color=#ffffff"),
            format!("{text}skin=light\n"),
            text.replace("hair=crop\n", ""),
        ] {
            assert!(Appearance::decode(&bad).is_err());
        }
    }
    #[test]
    fn outfit_changes_preserve_selected_skin_and_hair_except_explicit_approved_reset() {
        let mut a = Appearance::preset(1);
        for field in 1..4 {
            a.cycle(field, true);
        }
        let identity = [a.index(1), a.index(2), a.index(3)];
        for expected in [2, 3] {
            a.cycle(0, false);
            assert_eq!(a.index(0), expected);
            assert_eq!([a.index(1), a.index(2), a.index(3)], identity);
        }
        a.cycle(0, false);
        assert_eq!(a, Appearance::default());
    }
    #[test]
    fn persistence_transactions_and_bad_files() {
        let dir = temp();
        let path = dir.join("appearance.conf");
        assert_eq!(load(&path).unwrap(), Appearance::default());
        let mut e = Editor {
            path: Ok(path.clone()),
            ..Default::default()
        };
        e.begin();
        e.draft = Appearance::preset(3);
        e.cancel();
        assert_eq!(e.draft, e.saved);
        assert!(!path.exists());
        e.draft = Appearance::preset(2);
        assert!(e.apply());
        e.defaults();
        assert_eq!(load(&path).unwrap(), Appearance::preset(2));
        e.cancel();
        assert_eq!(e.draft, Appearance::preset(2));
        e.defaults();
        assert!(e.apply());
        assert_eq!(load(&path).unwrap(), Appearance::default());
        e.path = Ok(path.join("impossible"));
        e.draft = Appearance::preset(3);
        assert!(!e.apply());
        assert_eq!(load(&path).unwrap(), Appearance::default());
        assert_eq!(e.saved, Appearance::default());
        // Exhaust the bounded exclusive temporary-file slots. A failed save
        // cannot truncate the current configuration or reuse someone else's temp.
        for serial in 0..32 {
            fs::write(
                dir.join(format!(".appearance-{}-{serial}.tmp", std::process::id())),
                b"occupied",
            )
            .unwrap();
        }
        assert!(save(&path, Appearance::preset(3)).is_err());
        assert_eq!(load(&path).unwrap(), Appearance::default());
        for bytes in [
            vec![255],
            vec![b'x'; 4097],
            b"garbage".to_vec(),
            b"burnhop-appearance=9\n".to_vec(),
        ] {
            fs::write(&path, bytes).unwrap();
            assert!(load(&path).is_err());
        }
        fs::remove_dir_all(dir).unwrap();
    }
}
