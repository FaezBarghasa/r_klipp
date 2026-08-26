use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub size: usize,
    pub data: String, // Base64 encoded PNG data
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct GCodeMetadata {
    pub filename: String,
    pub size: u64,
    pub modified: u64,
    pub estimated_time: Option<f64>,
    pub filament_total: Option<f64>,
    pub filament_weight_total: Option<f64>,
    pub layer_height: Option<f64>,
    pub first_layer_height: Option<f64>,
    pub layer_count: Option<u32>,
    pub object_height: Option<f64>,
    pub slicer: Option<String>,
    pub slicer_version: Option<String>,
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Clone)]
pub struct MetadataParser {
    cura_time_re: Regex,
    cura_filament_re: Regex,
    prusa_time_re: Regex,
    prusa_filament_re: Regex,
    prusa_filament_g_re: Regex,
    layer_height_re: Regex,
    first_layer_re: Regex,
    layer_count_re: Regex,
    object_height_re: Regex,
    slicer_re: Regex,
    slicer_version_re: Regex,
    thumbnail_start_re: Regex,
    thumbnail_end_re: Regex,
}

impl Default for MetadataParser {
    fn default() -> Self {
        Self::new()
    }
}

impl MetadataParser {
    pub fn new() -> Self {
        Self {
            cura_time_re: Regex::new(r";TIME:\s*(\d+)").unwrap(),
            cura_filament_re: Regex::new(r";Filament used:\s*([\d\.]+)m").unwrap(),
            prusa_time_re: Regex::new(r"(?i);\s*estimated printing time[^=]*=\s*(.*)").unwrap(),
            prusa_filament_re: Regex::new(r"(?i);\s*filament used\s*\[mm\]\s*=\s*([\d\.]+)").unwrap(),
            prusa_filament_g_re: Regex::new(r"(?i);\s*filament used\s*\[g\]\s*=\s*([\d\.]+)").unwrap(),
            layer_height_re: Regex::new(r"(?i);\s*(?:Layer height|layer_height)\s*[:=]\s*([\d\.]+)").unwrap(),
            first_layer_re: Regex::new(r"(?i);\s*(?:First layer height|first_layer_height)\s*[:=]\s*([\d\.]+)").unwrap(),
            layer_count_re: Regex::new(r"(?i);\s*(?:total_layer_count|layer_count|total layers|LAYER_COUNT)\s*[:=]\s*(\d+)").unwrap(),
            object_height_re: Regex::new(r"(?i);\s*(?:max_z_height|object_height|MAXZ)\s*[:=]\s*([\d\.]+)").unwrap(),
            slicer_re: Regex::new(r"(?i);\s*(?:Generated with|slicer)\s*(.*)").unwrap(),
            slicer_version_re: Regex::new(r"(?i);\s*(?:slicer_version|version)\s*[:=]\s*([0-9\.]+)").unwrap(),
            thumbnail_start_re: Regex::new(r";\s*thumbnail begin\s+(\d+)x(\d+)\s+(\d+)").unwrap(),
            thumbnail_end_re: Regex::new(r";\s*thumbnail end").unwrap(),
        }
    }

    /// Parse human-readable duration strings like "1d 2h 3m 4s", "2h 30m", "45m 12s", "123s" into seconds.
    pub fn parse_duration_string(s: &str) -> Option<f64> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }

        if let Ok(sec) = s.parse::<f64>() {
            return Some(sec);
        }

        let mut total_secs = 0.0;
        let mut matched = false;

        let re = Regex::new(r"(\d+(?:\.\d+)?)\s*([dhms])").ok()?;
        for caps in re.captures_iter(s) {
            let val = caps[1].parse::<f64>().ok()?;
            let unit = &caps[2];
            matched = true;
            match unit {
                "d" => total_secs += val * 86400.0,
                "h" => total_secs += val * 3600.0,
                "m" => total_secs += val * 60.0,
                "s" => total_secs += val,
                _ => {}
            }
        }

        if matched {
            Some(total_secs)
        } else {
            None
        }
    }

    /// Parse metadata from G-Code content (reads header and footer up to ~512KB).
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<GCodeMetadata> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let mut file = File::open(path)?;
        let file_len = metadata.len();

        let mut header_buf = vec![0u8; 256 * 1024];
        let header_read = file.read(&mut header_buf)?;
        header_buf.truncate(header_read);
        let header_str = String::from_utf8_lossy(&header_buf);

        let footer_str = if file_len > 256 * 1024 {
            let footer_seek = file_len.saturating_sub(256 * 1024);
            let _ = file.seek(SeekFrom::Start(footer_seek));
            let mut footer_buf = vec![0u8; 256 * 1024];
            let footer_read = file.read(&mut footer_buf).unwrap_or(0);
            footer_buf.truncate(footer_read);
            String::from_utf8_lossy(&footer_buf).to_string()
        } else {
            String::new()
        };

        let mut meta = GCodeMetadata {
            filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            size: file_len,
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        };

        let combined = format!("{}\n{}", header_str, footer_str);

        // 1. Time parsing (Cura first, then Prusa/Orca/Bambu)
        if let Some(caps) = self.cura_time_re.captures(&combined) {
            if let Ok(sec) = caps[1].parse::<f64>() {
                meta.estimated_time = Some(sec);
            }
        } else if let Some(caps) = self.prusa_time_re.captures(&combined) {
            meta.estimated_time = Self::parse_duration_string(&caps[1]);
        }

        // 2. Filament parsing
        if let Some(caps) = self.cura_filament_re.captures(&combined) {
            if let Ok(m) = caps[1].parse::<f64>() {
                meta.filament_total = Some(m * 1000.0); // convert meters to mm
            }
        } else if let Some(caps) = self.prusa_filament_re.captures(&combined) {
            if let Ok(mm) = caps[1].parse::<f64>() {
                meta.filament_total = Some(mm);
            }
        }

        if let Some(caps) = self.prusa_filament_g_re.captures(&combined) {
            if let Ok(g) = caps[1].parse::<f64>() {
                meta.filament_weight_total = Some(g);
            }
        }

        // 3. Layer heights
        if let Some(caps) = self.layer_height_re.captures(&combined) {
            if let Ok(h) = caps[1].parse::<f64>() {
                meta.layer_height = Some(h);
            }
        }

        if let Some(caps) = self.first_layer_re.captures(&combined) {
            if let Ok(h) = caps[1].parse::<f64>() {
                meta.first_layer_height = Some(h);
            }
        }

        // 4. Layer count & object height
        if let Some(caps) = self.layer_count_re.captures(&combined) {
            if let Ok(count) = caps[1].parse::<u32>() {
                meta.layer_count = Some(count);
            }
        }

        if let Some(caps) = self.object_height_re.captures(&combined) {
            if let Ok(h) = caps[1].parse::<f64>() {
                meta.object_height = Some(h);
            }
        }

        // 5. Slicer name and version
        if let Some(caps) = self.slicer_re.captures(&combined) {
            meta.slicer = Some(caps[1].trim().to_string());
        }

        if let Some(caps) = self.slicer_version_re.captures(&combined) {
            meta.slicer_version = Some(caps[1].trim().to_string());
        }

        // 6. Extract Thumbnails
        self.extract_thumbnails(&combined, &mut meta);

        Ok(meta)
    }

    fn extract_thumbnails(&self, content: &str, meta: &mut GCodeMetadata) {
        let mut lines = content.lines();
        while let Some(line) = lines.next() {
            if let Some(caps) = self.thumbnail_start_re.captures(line) {
                let width = caps[1].parse::<u32>().unwrap_or(0);
                let height = caps[2].parse::<u32>().unwrap_or(0);
                let size = caps[3].parse::<usize>().unwrap_or(0);

                let mut base64_data = String::new();
                for inner_line in lines.by_ref() {
                    if self.thumbnail_end_re.is_match(inner_line) {
                        break;
                    }
                    let cleaned = inner_line.trim_start_matches(';').trim();
                    base64_data.push_str(cleaned);
                }

                if !meta.thumbnails.iter().any(|t| t.width == width && t.height == height) {
                    meta.thumbnails.push(Thumbnail {
                        width,
                        height,
                        size,
                        data: base64_data,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_thumbnail_extraction() {
        let sample_gcode = r#"
;Generated with Cura_SteamEngine 5.4.0
;Layer height: 0.2
;First layer height: 0.28
;TIME: 1234
;Filament used: 1.45m
; thumbnail begin 32x32 400
; iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0
; AAAAFElEQVR42mNk+M9QDwMMAgEQwEwA
; thumbnail end
G28
G1 X100 Y100 E10
"#;

        let parser = MetadataParser::new();
        let mut meta = GCodeMetadata::default();
        if let Some(caps) = parser.cura_time_re.captures(sample_gcode) {
            meta.estimated_time = caps[1].parse::<f64>().ok();
        }
        if let Some(caps) = parser.layer_height_re.captures(sample_gcode) {
            meta.layer_height = caps[1].parse::<f64>().ok();
        }
        parser.extract_thumbnails(sample_gcode, &mut meta);

        assert_eq!(meta.estimated_time, Some(1234.0));
        assert_eq!(meta.layer_height, Some(0.2));
        assert_eq!(meta.thumbnails.len(), 1);
        assert_eq!(meta.thumbnails[0].width, 32);
        assert_eq!(meta.thumbnails[0].height, 32);
        assert!(meta.thumbnails[0].data.contains("iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0"));
    }

    #[test]
    fn test_prusa_slicer_time_and_filament_parsing() {
        assert_eq!(MetadataParser::parse_duration_string("1d 2h 3m 4s"), Some(86400.0 + 7200.0 + 180.0 + 4.0));
        assert_eq!(MetadataParser::parse_duration_string("2h 30m"), Some(7200.0 + 1800.0));
        assert_eq!(MetadataParser::parse_duration_string("45m"), Some(2700.0));
        assert_eq!(MetadataParser::parse_duration_string("120"), Some(120.0));

        let prusa_gcode = r#"
; generated by PrusaSlicer 2.7.1 on 2024-01-15
; estimated printing time (normal mode) = 1h 15m 30s
; filament used [mm] = 2345.67
; filament used [g] = 7.05
; layer_height = 0.20
; first_layer_height = 0.25
; total_layer_count = 150
"#;
        let parser = MetadataParser::new();
        let mut meta = GCodeMetadata::default();
        if let Some(caps) = parser.prusa_time_re.captures(prusa_gcode) {
            meta.estimated_time = MetadataParser::parse_duration_string(&caps[1]);
        }
        if let Some(caps) = parser.prusa_filament_re.captures(prusa_gcode) {
            meta.filament_total = caps[1].parse().ok();
        }
        if let Some(caps) = parser.prusa_filament_g_re.captures(prusa_gcode) {
            meta.filament_weight_total = caps[1].parse().ok();
        }

        assert_eq!(meta.estimated_time, Some(3600.0 + 900.0 + 30.0));
        assert_eq!(meta.filament_total, Some(2345.67));
        assert_eq!(meta.filament_weight_total, Some(7.05));
    }
}
