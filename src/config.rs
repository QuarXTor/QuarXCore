use std::env;
use std::fs;
use std::path::PathBuf;

/// Глобальный конфиг QuarXTor / QuarXDrive / quarxctl.
///
/// Источники:
///   1) дефолты (Default)
///   2) ini-файл (~/.config/quarxtor/quarxctl.ini)
///   3) env-переменные (перекрывают ini)
#[derive(Debug, Clone)]
pub struct QuarxConfig {
    /// Размер L0-чунка (байт).
    pub l0_chunk: usize,

    /// Минимальный размер файла (в байтах) для импорта.
    /// Если 0 — порога нет.
    pub import_min_file_size: u64,

    /// Политика импорта host FS.
    pub import_skip_hidden: bool,
    pub import_skip_symlink: bool,
    pub import_skip_zero: bool,
    pub import_skip_devices: bool,
    pub import_skip_special: bool,

    /// Лимит RAM для будущего RAM-tier / кэша, в байтах.
    /// 0        = RAM-слой выключен (работаем только по диску).
    /// u64::MAX = "full/unlimited" — не ограничиваем со своей стороны.
    pub ram_limit_bytes: u64,

    /// Импорт использовать Z-node/cheap-size (на уровне FS-импортера).
    pub fs_import_use_z: bool,
    /// Порог в блоках/условном размере для применения Z-анализа (резерв).
    pub fs_import_z_threshold: u64,

    /// Создавать ли Z-node/cheap-size при импорте.
    pub analysis_enable_znode: bool,
    /// Fallback для fs-stats:
    ///   true  = если нет Z-node, читаем payload (дорого);
    ///   false = если нет Z-node, считаем размер 0 (дёшево).
    pub analysis_fs_stats_fallback: bool,
}

impl Default for QuarxConfig {
    fn default() -> Self {
        Self {
            // базовый дефолт: 8 KiB
            l0_chunk: 8 * 1024,

            import_min_file_size: 0, // нет порога по умолчанию

            // По умолчанию:
            // - скрытые файлы НЕ пропускаем,
            // - симлинки/девайсы/спец-файлы пропускаем,
            // - нулевые файлы пропускаем.
            import_skip_hidden: false,
            import_skip_symlink: true,
            import_skip_zero: true,
            import_skip_devices: true,
            import_skip_special: true,

            // По умолчанию RAM-tier выключен.
            ram_limit_bytes: 0,

            // Импорт по умолчанию Z-node включает, с порогом 10.
            fs_import_use_z: true,
            fs_import_z_threshold: 10,

            // Аналитика/Z-node включена.
            analysis_enable_znode: true,
            // По умолчанию bytes в fs-stats считаем только по Z-node,
            // без fallback на чтение payload.
            analysis_fs_stats_fallback: false,
        }
    }
}

fn default_config_path() -> PathBuf {
    if let Ok(xdg) = env::var("XDG_CONFIG_HOME") {
        PathBuf::from(xdg).join("quarxtor/quarxctl.ini")
    } else if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".config/quarxtor/quarxctl.ini")
    } else {
        PathBuf::from("quarxctl.ini")
    }
}

fn parse_usize_simple(s: &str) -> Option<usize> {
    s.trim().parse::<usize>().ok()
}

fn parse_u64_simple(s: &str) -> Option<u64> {
    s.trim().parse::<u64>().ok()
}

fn parse_bool_simple(s: &str) -> Option<bool> {
    let v = s.trim().to_ascii_lowercase();
    match v.as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

/// Простой парсер human-friendly размеров:
///   "none", "off", "0"     -> Some(0)
///   "full", "unlimited"    -> Some(u64::MAX)
///   "16G", "1G", "256M",
///   "10M", "512K", "123"   -> байты (1024-based)
fn parse_size_bytes(s: &str) -> Option<u64> {
    let v = s.trim().to_ascii_lowercase();
    if v == "none" || v == "off" || v == "0" {
        return Some(0);
    }
    if v == "full" || v == "unlimited" {
        return Some(u64::MAX);
    }

    // число + optional суффикс
    let (num_part, suffix) = {
        let mut split_at = v.len();
        for (i, ch) in v.chars().enumerate() {
            if !ch.is_ascii_digit() {
                split_at = i;
                break;
            }
        }
        v.split_at(split_at)
    };

    if num_part.is_empty() {
        return None;
    }

    let base: u64 = num_part.parse().ok()?;
    let mul: u64 = match suffix {
        "" => 1,
        "k" | "kb" => 1024,
        "m" | "mb" => 1024 * 1024,
        "g" | "gb" => 1024 * 1024 * 1024,
        _ => return None,
    };

    base.checked_mul(mul)
}

impl QuarxConfig {
    pub fn load() -> Self {
        let mut cfg = QuarxConfig::default();

        // 1) ini (~/.config/quarxtor/quarxctl.ini)
        let path = default_config_path();
        if let Ok(text) = fs::read_to_string(&path) {
            for line in text.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
                    continue;
                }
                let mut parts = line.splitn(2, '=');
                let key = parts.next().unwrap().trim();
                let value = parts.next().unwrap_or("").trim();

                match key {
                    "l0_chunk" => {
                        if let Some(n) = parse_usize_simple(value) {
                            cfg.l0_chunk = n;
                        }
                    }

                    "import.min_file_size" => {
                        if let Some(n) = parse_u64_simple(value) {
                            cfg.import_min_file_size = n;
                        }
                    }

                    "import.skip_hidden" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.import_skip_hidden = b;
                        }
                    }
                    "import.skip_symlink" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.import_skip_symlink = b;
                        }
                    }
                    "import.skip_zero" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.import_skip_zero = b;
                        }
                    }
                    "import.skip_devices" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.import_skip_devices = b;
                        }
                    }
                    "import.skip_special" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.import_skip_special = b;
                        }
                    }

                    // RAM лимит из ini:
                    //   ram.limit=none
                    //   ram.limit=full
                    //   ram.limit=16G
                    //   ram.limit=256M
                    "ram.limit" => {
                        if let Some(n) = parse_size_bytes(value) {
                            cfg.ram_limit_bytes = n;
                        }
                    }

                    // FS-import / Z-node-порог
                    "fs_import.use_z" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.fs_import_use_z = b;
                        }
                    }
                    "fs_import.z_threshold" => {
                        if let Some(n) = parse_u64_simple(value) {
                            cfg.fs_import_z_threshold = n;
                        }
                    }

                    // Аналитика / Z-node
                    "analysis.enable_znode" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.analysis_enable_znode = b;
                        }
                    }
                    "analysis.fs_stats_fallback" => {
                        if let Some(b) = parse_bool_simple(value) {
                            cfg.analysis_fs_stats_fallback = b;
                        }
                    }

                    _ => {
                        // неизвестные ключи игнорируем
                    }
                }
            }
        }

        // 2) env overrides
        if let Ok(v) = env::var("QUARX_L0_CHUNK") {
            if let Some(n) = parse_usize_simple(&v) {
                cfg.l0_chunk = n;
            }
        }

        if let Ok(v) = env::var("QUARX_IMPORT_MIN_FILE_SIZE") {
            if let Some(n) = parse_u64_simple(&v) {
                cfg.import_min_file_size = n;
            }
        }

        if let Ok(v) = env::var("QUARX_IMPORT_SKIP_HIDDEN") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.import_skip_hidden = b;
            }
        }
        if let Ok(v) = env::var("QUARX_IMPORT_SKIP_SYMLINK") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.import_skip_symlink = b;
            }
        }
        if let Ok(v) = env::var("QUARX_IMPORT_SKIP_ZERO") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.import_skip_zero = b;
            }
        }
        if let Ok(v) = env::var("QUARX_IMPORT_SKIP_DEVICES") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.import_skip_devices = b;
            }
        }
        if let Ok(v) = env::var("QUARX_IMPORT_SKIP_SPECIAL") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.import_skip_special = b;
            }
        }

        // RAM-лимит через ENV
        if let Ok(v) = env::var("QUARX_RAM_LIMIT") {
            if let Some(n) = parse_size_bytes(&v) {
                cfg.ram_limit_bytes = n;
            }
        }

        // FS-import / Z-node
        if let Ok(v) = env::var("QUARX_FS_IMPORT_USE_Z") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.fs_import_use_z = b;
            }
        }
        if let Ok(v) = env::var("QUARX_FS_IMPORT_Z_THRESHOLD") {
            if let Some(n) = parse_u64_simple(&v) {
                cfg.fs_import_z_threshold = n;
            }
        }

        // Аналитика
        if let Ok(v) = env::var("QUARX_ANALYSIS_ENABLE_ZNODE") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.analysis_enable_znode = b;
            }
        }
        if let Ok(v) = env::var("QUARX_ANALYSIS_FS_STATS_FALLBACK") {
            if let Some(b) = parse_bool_simple(&v) {
                cfg.analysis_fs_stats_fallback = b;
            }
        }

        cfg
    }
}
