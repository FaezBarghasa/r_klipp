
slint::include_modules!();

pub fn main() {
    let main_window = MainWindow::new().unwrap();

    // This would be updated from the host's state
    main_window.set_connection_status("PREDICTIVE MODE (Tier 1)".into());
    main_window.set_status_color("green".into());

    main_window.run().unwrap();
}

#[cfg(test)]
mod tests {
    // Slint UI tests are typically done via their own testing framework,
    // which is outside the scope of this phase.
}
