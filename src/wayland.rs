use std::error::Error;

use smithay_client_toolkit::{
    delegate_output, delegate_registry,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
};
use wayland_client::{
    globals::registry_queue_init, protocol::wl_output, Connection, EventQueue, QueueHandle,
};

use crate::config;

#[derive(Debug)]
pub struct WaylandHandle {
    _connection: Connection, // Keeps the connection alive
    event_queue: EventQueue<WaylandState>,
    state: WaylandState,
}

#[derive(Debug)]
struct WaylandState {
    registry_state: RegistryState,
    output_state: OutputState,
    outputs: Vec<config::MonitorInfo>,
}

impl WaylandHandle {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let conn = Connection::connect_to_env()?;
        let (globals, mut event_queue) = registry_queue_init(&conn)?;
        let qh = event_queue.handle();

        let mut state = WaylandState {
            registry_state: RegistryState::new(&globals),
            output_state: OutputState::new(&globals, &qh),
            outputs: Vec::new(),
        };

        // NOTE: Hi future me, this double roundtrip is required, don't you dare touch it.
        event_queue.roundtrip(&mut state)?;
        event_queue.roundtrip(&mut state)?;

        Ok(Self {
            _connection: conn,
            event_queue,
            state,
        })
    }

    pub fn get_outputs(&mut self) -> &[config::MonitorInfo] {
        if self.event_queue.dispatch_pending(&mut self.state).is_ok() {
            let _ = self.event_queue.roundtrip(&mut self.state);
        }
        &self.state.outputs
    }
}

impl OutputHandler for WaylandState {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }

    fn new_output(&mut self, _: &Connection, _: &QueueHandle<Self>, o: wl_output::WlOutput) {
        if let Some(info) = self.output_state.info(&o) {
            if let Some(mode) = info
                .modes
                .iter()
                .find(|m| m.current || m.preferred)
                .or_else(|| info.modes.first())
            {
                let monitor = config::MonitorInfo {
                    refresh_rate: mode.refresh_rate as f32 / 1000.0,
                    resolution: config::Resolution {
                        width: mode.dimensions.0,
                        height: mode.dimensions.1,
                    },
                    id: info.id,
                };
                self.outputs.push(monitor);
            }
        }
    }

    fn update_output(&mut self, _: &Connection, _: &QueueHandle<Self>, o: wl_output::WlOutput) {
        if let Some(info) = self.output_state.info(&o) {
            if let Some(mode) = info
                .modes
                .iter()
                .find(|m| m.current || m.preferred)
                .or_else(|| info.modes.first())
            {
                let monitor = config::MonitorInfo {
                    refresh_rate: mode.refresh_rate as f32 / 1000.0,
                    resolution: config::Resolution {
                        width: mode.dimensions.0,
                        height: mode.dimensions.1,
                    },
                    id: info.id,
                };

                let exists = self.outputs.iter_mut().any(|existing| {
                    if existing.id == info.id {
                        existing.refresh_rate = monitor.refresh_rate;
                        existing.resolution = monitor.resolution;
                        true
                    } else {
                        false
                    }
                });

                if !exists {
                    self.outputs.push(monitor);
                }
            }
        }
    }

    fn output_destroyed(&mut self, _: &Connection, _: &QueueHandle<Self>, o: wl_output::WlOutput) {
        if let Some(info) = self.output_state.info(&o) {
            self.outputs.retain(|m| m.id != info.id);
        }
    }
}

impl ProvidesRegistryState for WaylandState {
    registry_handlers!(OutputState);

    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
}

delegate_output!(WaylandState);
delegate_registry!(WaylandState);

#[cfg(test)]
mod tests {
    use super::*;

    // This test would fail if no monitors are detected/connected
    // I see no reason to change that behaviour as is
    #[test]
    fn test_find_monitor() {
        let mut wlhandle = WaylandHandle::new().expect("Failed to create handle");

        let outputs = wlhandle.get_outputs();

        assert!(!outputs.is_empty());
        println!("{:#?}", outputs);
    }

    #[test]
    fn test_wayland_connection() {
        assert!(WaylandHandle::new().is_ok());
    }
}
