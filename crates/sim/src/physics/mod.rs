//! Deterministic Physical Machine & Thermal Dynamics Simulator.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisPhysicsModel {
    pub mass_kg: f64,
    pub damping_coeff: f64,
    pub motor_torque_constant: f64,
    pub position_mm: f64,
    pub velocity_mms: f64,
}

impl AxisPhysicsModel {
    pub fn new(mass_kg: f64, damping_coeff: f64, motor_torque_constant: f64) -> Self {
        Self {
            mass_kg: mass_kg.max(0.01),
            damping_coeff: damping_coeff.max(0.0),
            motor_torque_constant: motor_torque_constant.max(0.01),
            position_mm: 0.0,
            velocity_mms: 0.0,
        }
    }

    /// Advances physics simulation by `dt` seconds given applied motor current/force in Newtons.
    pub fn step(&mut self, motor_force_n: f64, dt_s: f64) {
        if dt_s <= 0.0 {
            return;
        }

        // Net force = Motor Force - Damping Force
        let damping_force = self.damping_coeff * self.velocity_mms;
        let net_force = motor_force_n - damping_force;
        let accel_mms2 = (net_force / self.mass_kg) * 1000.0;

        self.velocity_mms += accel_mms2 * dt_s;
        self.position_mm += self.velocity_mms * dt_s;
    }
}

/// 1D Lumped-capacitance Thermal Plant Model for Hotends and Heated Beds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThermalPlantModel {
    pub thermal_capacitance_j_per_k: f64,
    pub thermal_resistance_k_per_w: f64,
    pub ambient_temp_c: f64,
    pub current_temp_c: f64,
}

impl ThermalPlantModel {
    pub fn new(capacitance: f64, resistance: f64, ambient: f64) -> Self {
        Self {
            thermal_capacitance_j_per_k: capacitance.max(0.1),
            thermal_resistance_k_per_w: resistance.max(0.1),
            ambient_temp_c: ambient,
            current_temp_c: ambient,
        }
    }

    /// Steps thermal model given applied electrical heating power (Watts) over `dt` seconds.
    pub fn step(&mut self, heater_power_w: f64, dt_s: f64) -> f64 {
        if dt_s <= 0.0 {
            return self.current_temp_c;
        }

        let heat_loss_w = (self.current_temp_c - self.ambient_temp_c) / self.thermal_resistance_k_per_w;
        let net_heat_w = heater_power_w - heat_loss_w;
        let delta_t = (net_heat_w / self.thermal_capacitance_j_per_k) * dt_s;

        self.current_temp_c += delta_t;
        self.current_temp_c
    }
}
