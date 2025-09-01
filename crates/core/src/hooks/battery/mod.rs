//! Battery Status Hook
//!
//! This module provides a `use_battery` hook that tracks battery status,
//! similar to the web API's Battery Status API. It provides real-time
//! information about battery level, charging status, and time estimates.

use crate::hooks::{effect::use_effect, interval::use_interval, state::use_state};
use battery::{Manager, State};
use std::time::{Duration, SystemTime};

/// Battery status information
#[derive(Debug, Clone, PartialEq)]
pub struct BatteryStatus {
    /// Whether battery API is supported on this platform
    pub is_supported: bool,
    /// Battery charge level (0.0 to 1.0, where 1.0 is 100%)
    pub level: f64,
    /// Whether the battery is currently charging
    pub charging: bool,
    /// Time remaining until battery is discharged (in seconds)
    /// None if unknown or not applicable
    pub discharging_time: Option<f64>,
    /// Time remaining until battery is fully charged (in seconds)
    /// None if unknown or not applicable
    pub charging_time: Option<f64>,
    /// Last update timestamp
    pub last_updated: SystemTime,
}

impl Default for BatteryStatus {
    fn default() -> Self {
        Self {
            is_supported: false,
            level: 0.0,
            charging: false,
            discharging_time: None,
            charging_time: None,
            last_updated: SystemTime::now(),
        }
    }
}

impl BatteryStatus {
    /// Create a new battery status with current timestamp
    pub fn new(
        is_supported: bool,
        level: f64,
        charging: bool,
        discharging_time: Option<f64>,
        charging_time: Option<f64>,
    ) -> Self {
        Self {
            is_supported,
            level: level.clamp(0.0, 1.0), // Ensure level is between 0.0 and 1.0
            charging,
            discharging_time,
            charging_time,
            last_updated: SystemTime::now(),
        }
    }

    /// Get battery level as a percentage (0-100)
    pub fn level_percentage(&self) -> f64 {
        self.level * 100.0
    }

    /// Check if battery is critically low (below 10%)
    pub fn is_critical(&self) -> bool {
        self.level < 0.1
    }

    /// Check if battery is low (below 20%)
    pub fn is_low(&self) -> bool {
        self.level < 0.2
    }

    /// Get formatted time remaining for discharging
    pub fn discharging_time_formatted(&self) -> String {
        match self.discharging_time {
            Some(seconds) => format_duration(seconds),
            None => "Unknown".to_string(),
        }
    }

    /// Get formatted time remaining for charging
    pub fn charging_time_formatted(&self) -> String {
        match self.charging_time {
            Some(seconds) => format_duration(seconds),
            None => "Unknown".to_string(),
        }
    }

    /// Get battery status description
    pub fn status_description(&self) -> String {
        if !self.is_supported {
            return "Battery API not supported".to_string();
        }

        let level_desc = format!("{:.1}%", self.level_percentage());

        if self.charging {
            format!("Charging ({})", level_desc)
        } else if self.is_critical() {
            format!("Critical ({})", level_desc)
        } else if self.is_low() {
            format!("Low ({})", level_desc)
        } else {
            format!("Discharging ({})", level_desc)
        }
    }
}

/// Format duration in seconds to human-readable string
fn format_duration(seconds: f64) -> String {
    if seconds.is_infinite() || seconds.is_nan() {
        return "Unknown".to_string();
    }

    let total_seconds = seconds as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;

    if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

/// Get battery information from the system
fn get_battery_info() -> BatteryStatus {
    match Manager::new() {
        Ok(manager) => {
            match manager.batteries() {
                Ok(batteries) => {
                    // Get the first battery (most common case)
                    if let Some(battery) = batteries.flatten().next() {
                        let level = battery.state_of_charge().value as f64;
                        let charging = matches!(battery.state(), State::Charging);

                        let discharging_time = if !charging {
                            battery
                                .time_to_empty()
                                .map(|duration| duration.value as f64)
                        } else {
                            None
                        };

                        let charging_time = if charging {
                            battery.time_to_full().map(|duration| duration.value as f64)
                        } else {
                            None
                        };

                        return BatteryStatus::new(
                            true,
                            level,
                            charging,
                            discharging_time,
                            charging_time,
                        );
                    }

                    // No batteries found
                    BatteryStatus::new(false, 0.0, false, None, None)
                }
                Err(_) => BatteryStatus::new(false, 0.0, false, None, None),
            }
        }
        Err(_) => BatteryStatus::new(false, 0.0, false, None, None),
    }
}

/// Professional use_battery hook that tracks battery status
///
/// This hook provides real-time battery information similar to the web API's
/// Battery Status API. It automatically updates the battery status at regular
/// intervals and provides comprehensive battery information.
///
/// # Returns
///
/// Returns a `BatteryStatus` struct containing:
/// - `is_supported`: Whether battery API is supported
/// - `level`: Battery charge level (0.0 to 1.0)
/// - `charging`: Whether the battery is currently charging
/// - `discharging_time`: Time remaining until discharged (in seconds)
/// - `charging_time`: Time remaining until fully charged (in seconds)
///
/// # Examples
///
/// ```rust,no_run
/// use pulse_core::hooks::battery::use_battery;
///
/// // Example usage in a component
/// let battery = use_battery();
/// let level_percentage = battery.level_percentage();
/// let status = battery.status_description();
///
/// println!("Battery Level: {:.1}%", level_percentage);
/// println!("Status: {}", status);
///
/// if battery.charging {
///     println!("Charging time: {}", battery.charging_time_formatted());
/// } else {
///     println!("Discharging time: {}", battery.discharging_time_formatted());
/// }
/// ```
///
/// # Performance Notes
///
/// - Updates every 5 seconds by default to balance accuracy and performance
/// - Uses efficient polling to minimize system resource usage
/// - Caches battery information between updates
/// - Designed for TUI applications with minimal overhead
///
/// # Platform Support
///
/// Supports all major platforms through the `battery` crate:
/// - Windows
/// - macOS
/// - Linux
/// - FreeBSD
///
/// If battery information is not available, `is_supported` will be `false`.
///
pub fn use_battery() -> BatteryStatus {
    let (battery_status, set_battery_status) = use_state(BatteryStatus::default);

    // Update battery status every 5 seconds
    use_interval(
        {
            let set_battery_status = set_battery_status.clone();
            move || {
                let new_status = get_battery_info();
                set_battery_status.set(new_status);
            }
        },
        Duration::from_secs(5),
    );

    // Initial battery status fetch
    use_effect(
        {
            let set_battery_status = set_battery_status.clone();
            move || {
                let initial_status = get_battery_info();
                set_battery_status.set(initial_status);
                None::<Box<dyn FnOnce() + Send>> // Return None for cleanup function
            }
        },
        (),
    );

    battery_status.get()
}
