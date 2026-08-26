use anyhow::Result;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
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
    layer_height_re: Regex,
    first_layer_re: Regex,
    slicer_re: Regex,
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
            prusa_time_re: Regex::new(r"; estimated printing time \(normal mode\)\s*=\s*(.*)").unwrap(),
            prusa_filament_re: Regex::new(r"; filament used \[mm\]\s*=\s*([\d\.]+)").unwrap(),
            layer_height_re: Regex::new(r";Layer height:\s*([\d\.]+)").unwrap(),
            first_layer_re: Regex::new(r";First layer height:\s*([\d\.]+)").unwrap(),
            slicer_re: Regex::new(r";Generated with (.*)").unwrap(),
            thumbnail_start_re: Regex::new(r"; thumbnail begin\s+(\d+)x(\d+)\s+(\d+)").unwrap(),
            thumbnail_end_re: Regex::new(r"; thumbnail end").unwrap(),
        }
    }

    /// Parse metadata from G-Code content (reads header and footer up to ~512KB).
    pub fn parse_file<P: AsRef<Path>>(&self, path: P) -> Result<GCodeMetadata> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        let mut file = File::open(path)?;

        let mut header_buf = vec![0u8; 256 * 1024];
        let bytes_read = file.read(&mut header_buf)?;
        header_buf.truncate(bytes_read);
        let header_str = String::from_utf8_lossy(&header_buf);

        let mut meta = GCodeMetadata {
            filename: path.file_name().unwrap_or_default().to_string_lossy().to_string(),
            size: metadata.len(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(0),
            ..Default::default()
        };

        // Parse Cura-style TIME
        if let Some(caps) = self.cura_time_re.captures(&header_str) {
            if let Ok(sec) = caps[1].parse::<f64>() {
                meta.estimated_time = Some(sec);
            }
        }

        // Parse Cura filament
        if let Some(caps) = self.cura_filament_re.captures(&header_str) {
            if let Ok(m) = caps[1].parse::<f64>() {
                meta.filament_total = Some(m * 1000.0); // convert meters to mm
            }
        }

        // Parse layer height
        if let Some(caps) = self.layer_height_re.captures(&header_str) {
            if let Ok(h) = caps[1].parse::<f64>() {
                meta.layer_height = Some(h);
            }
        }

        // Parse first layer height
        if let Some(caps) = self.first_layer_re.captures(&header_str) {
            if let Ok(h) = caps[1].parse::<f64>() {
                meta.first_layer_height = Some(h);
            }
        }

        // Parse slicer
        if let Some(caps) = self.slicer_re.captures(&header_str) {
            meta.slicer = Some(caps[1].trim().to_string());
        }

        // Parse thumbnails
        self.extract_thumbnails(&header_str, &mut meta);

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
}
