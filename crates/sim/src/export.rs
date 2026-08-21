//! Trajectory Visualization Exporter (CSV & SVG).

use crate::pipeline::TrajectoryPoint;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use anyhow::Result;

/// Exports trajectory samples to a CSV file for plotting with Python/Matplotlib/Excel.
pub fn export_to_csv(points: &[TrajectoryPoint], path: &Path) -> Result<()> {
    let mut file = File::create(path)?;
    writeln!(file, "time_s,x_mm,y_mm,z_mm,velocity_mms,accel_mms2,jerk_mms3")?;

    for pt in points {
        writeln!(
            file,
            "{:.6},{:.4},{:.4},{:.4},{:.4},{:.4},{:.4}",
            pt.time_s, pt.x, pt.y, pt.z, pt.velocity, pt.accel, pt.jerk
        )?;
    }

    Ok(())
}

/// Exports toolhead path to an SVG file with velocity-based color grading.
pub fn export_to_svg(points: &[TrajectoryPoint], width: u32, height: u32, path: &Path) -> Result<()> {
    let mut file = File::create(path)?;

    if points.is_empty() {
        writeln!(file, "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\"></svg>", width, height)?;
        return Ok(());
    }

    // Compute bounding box
    let mut min_x: f64 = f64::MAX;
    let mut max_x: f64 = f64::MIN;
    let mut min_y: f64 = f64::MAX;
    let mut max_y: f64 = f64::MIN;
    let mut max_v: f64 = 1.0;

    for pt in points {
        min_x = min_x.min(pt.x);
        max_x = max_x.max(pt.x);
        min_y = min_y.min(pt.y);
        max_y = max_y.max(pt.y);
        max_v = max_v.max(pt.velocity);
    }

    let margin = 20.0;
    let dx = (max_x - min_x).max(1.0);
    let dy = (max_y - min_y).max(1.0);
    let scale_x = (width as f64 - 2.0 * margin) / dx;
    let scale_y = (height as f64 - 2.0 * margin) / dy;
    let scale = scale_x.min(scale_y);

    writeln!(
        file,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" style=\"background:#121212;\">",
        width, height, width, height
    )?;

    // Title & Legend
    writeln!(file, "  <text x=\"20\" y=\"30\" fill=\"#00e676\" font-family=\"sans-serif\" font-size=\"14\" font-weight=\"bold\">r_klipp Planned Toolhead Trajectory</text>")?;
    writeln!(file, "  <text x=\"20\" y=\"50\" fill=\"#888\" font-family=\"sans-serif\" font-size=\"11\">Max Velocity: {:.1} mm/s | Samples: {}</text>", max_v, points.len())?;

    // Render path segments
    for i in 0..points.len().saturating_sub(1) {
        let p1 = &points[i];
        let p2 = &points[i + 1];

        let x1 = margin + (p1.x - min_x) * scale;
        let y1 = (height as f64 - margin) - (p1.y - min_y) * scale;
        let x2 = margin + (p2.x - min_x) * scale;
        let y2 = (height as f64 - margin) - (p2.y - min_y) * scale;

        // Color mapped from blue (slow) to green (cruise) to red (accel/high)
        let v_ratio = (p1.velocity / max_v).clamp(0.0, 1.0);
        let red = (v_ratio * 255.0) as u8;
        let green = ((1.0 - (v_ratio - 0.5).abs() * 2.0).max(0.0) * 255.0) as u8;
        let blue = ((1.0 - v_ratio) * 255.0) as u8;

        writeln!(
            file,
            "  <line x1=\"{:.2}\" y1=\"{:.2}\" x2=\"{:.2}\" y2=\"{:.2}\" stroke=\"rgb({},{},{})\" stroke-width=\"2.5\" stroke-linecap=\"round\" />",
            x1, y1, x2, y2, red, green, blue
        )?;
    }

    writeln!(file, "</svg>")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_csv_and_svg_export() {
        let points = vec![
            TrajectoryPoint { time_s: 0.0, x: 0.0, y: 0.0, z: 0.0, velocity: 0.0, accel: 1000.0, jerk: 0.0 },
            TrajectoryPoint { time_s: 0.5, x: 25.0, y: 12.5, z: 0.0, velocity: 50.0, accel: 0.0, jerk: 0.0 },
            TrajectoryPoint { time_s: 1.0, x: 50.0, y: 25.0, z: 0.0, velocity: 0.0, accel: -1000.0, jerk: 0.0 },
        ];

        let csv_file = NamedTempFile::new().unwrap();
        assert!(export_to_csv(&points, csv_file.path()).is_ok());

        let svg_file = NamedTempFile::new().unwrap();
        assert!(export_to_svg(&points, 800, 600, svg_file.path()).is_ok());
    }
}
