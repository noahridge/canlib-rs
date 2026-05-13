//! CAN state machine example.
//!
//! Demonstrates a sensor node that runs a write-then-read loop, using
//! `VecDeque` as TX and RX queues. The controller queues commands
//! fire-and-forget into the sensor's RX queue; the sensor drains its RX
//! queue, processes each message through the state machine, and pushes
//! responses into its TX queue — which the controller later drains.
//!
//! Protocol:
//!   - INIT  (0x01) → sensor replies READY
//!   - READ  (0x02) → sensor replies with sample data
//!   - STOP  (0x03) → sensor replies STOPPED, then exits its loop
//!
//! # Run
//!
//! ```
//! cargo run --example state_machine
//! ```

use canlib::{CanMessage, MessageFlags};
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Protocol constants
// ---------------------------------------------------------------------------

const CMD_ID: u32 = 0x100;
const RSP_ID: u32 = 0x200;

const CMD_INIT: u8 = 0x01;
const CMD_READ: u8 = 0x02;
const CMD_STOP: u8 = 0x03;

const RSP_READY: u8 = 0x10;
const RSP_DATA: u8 = 0x20;
const RSP_STOPPED: u8 = 0x30;
const RSP_ERROR: u8 = 0xFF;

// ---------------------------------------------------------------------------
// Sensor state machine
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SensorState {
    Off,
    Ready,
    Stopped,
}

struct SensorNode {
    state: SensorState,
    sample_counter: u16,
    tx: VecDeque<CanMessage>,
    rx: VecDeque<CanMessage>,
}

impl SensorNode {
    fn new() -> Self {
        Self {
            state: SensorState::Off,
            sample_counter: 0,
            tx: VecDeque::new(),
            rx: VecDeque::new(),
        }
    }

    /// Main loop: drain the RX queue, process each message through the state
    /// machine, and push responses into the TX queue. Runs until the RX queue
    /// is empty and the state machine has reached `Stopped` or there is
    /// nothing left to process.
    fn run(&mut self) {
        println!("[sensor] Starting loop (state={:?})\n", self.state);

        while let Some(msg) = self.rx.pop_front() {
            if let Some(response) = self.handle_command(&msg) {
                self.tx.push_back(response);
            }

            if self.state == SensorState::Stopped {
                println!("[sensor] Stopped, exiting loop\n");
                return;
            }
        }

        println!("[sensor] RX queue empty, exiting loop\n");
    }

    /// Process a single command. Returns a response to enqueue, or None.
    fn handle_command(&mut self, cmd: &CanMessage) -> Option<CanMessage> {
        if cmd.id() != CMD_ID {
            return None;
        }

        let cmd_byte = cmd.data().first().copied().unwrap_or(0);

        let (next_state, response) = match (self.state, cmd_byte) {
            (SensorState::Off, CMD_INIT) => {
                println!("  [sensor] OFF -> READY (initialized)");
                (SensorState::Ready, vec![RSP_READY])
            }

            (SensorState::Ready, CMD_READ) => {
                self.sample_counter += 1;
                let value: u16 = 0x1234 + self.sample_counter;
                println!(
                    "  [sensor] READY -> read sample #{} (value=0x{:04X})",
                    self.sample_counter, value
                );
                (
                    SensorState::Ready,
                    vec![
                        RSP_DATA,
                        (self.sample_counter >> 8) as u8,
                        (self.sample_counter & 0xFF) as u8,
                        (value >> 8) as u8,
                        (value & 0xFF) as u8,
                    ],
                )
            }

            (SensorState::Ready, CMD_STOP) => {
                println!("  [sensor] READY -> STOPPED (shutdown)");
                (SensorState::Stopped, vec![RSP_STOPPED])
            }

            (state, cmd) => {
                println!(
                    "  [sensor] ERROR: unexpected cmd=0x{:02X} in state {:?}",
                    cmd, state
                );
                (self.state, vec![RSP_ERROR, cmd])
            }
        };

        self.state = next_state;
        Some(CanMessage::new(RSP_ID, &response).expect("valid response"))
    }
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    println!("=== CAN State Machine Example ===\n");

    let mut sensor = SensorNode::new();

    // Queue up all commands fire-and-forget into the sensor's RX queue
    let commands: &[(u8, &str)] = &[
        (CMD_INIT, "INIT"),
        (CMD_READ, "READ"),
        (CMD_READ, "READ"),
        (CMD_READ, "READ"),
        (CMD_STOP, "STOP"),
        (CMD_READ, "READ (after STOP, expect error)"),
    ];

    for &(cmd_byte, label) in commands {
        println!("[controller] Queuing {} (0x{:02X})", label, cmd_byte);
        let msg = CanMessage::new(CMD_ID, &[cmd_byte]).expect("valid command");
        sensor.rx.push_back(msg);
    }

    println!();

    // Run the sensor state machine (processes all queued commands)
    sensor.run();

    // Drain responses from the sensor's TX queue
    println!("[controller] Reading responses:");
    while let Some(rsp) = sensor.tx.pop_front() {
        let status = rsp.data().first().copied().unwrap_or(0);
        let label = match status {
            RSP_READY => "READY",
            RSP_DATA => "DATA",
            RSP_STOPPED => "STOPPED",
            RSP_ERROR => "ERROR",
            _ => "UNKNOWN",
        };
        println!(
            "  id=0x{:03X}  status={} ({})  payload={:02X?}",
            rsp.id(),
            label,
            if rsp.flags().contains(MessageFlags::STD) { "STD" } else { "EXT" },
            rsp.data(),
        );
    }

    println!("\nSensor final state: {:?}", sensor.state);
    println!("Done.");
}
