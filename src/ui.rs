use circular_buffer::CircularBuffer;
use eframe::egui::Context;
use eframe::Frame;
use gba::cpu::opcodes::Opcode;
use gba::cpu::EXECUTED_OPCODE_EVENT_ID;
use gba::events::Event;
use gba::gba::Gba;
use std::sync::{Arc, Mutex};

pub struct GbaApp {
    trace_opcode_viewer: TraceOpcodeViewer,
}

impl GbaApp {
    pub fn new() -> Self {
        Self {
            trace_opcode_viewer: TraceOpcodeViewer::new()
        }
    }

    pub fn register_handlers(&mut self, emulator_ctx: &mut Gba) {
        self.trace_opcode_viewer.register_handlers(emulator_ctx);
    }
}

impl eframe::App for GbaApp {
    fn update(&mut self, _ctx: &Context, _frame: &mut Frame) {
    }
}

struct TraceOpcodeViewer {
    executed_opcodes: Arc<Mutex<CircularBuffer<10, Opcode>>>
}

impl TraceOpcodeViewer {
    pub fn new() -> Self {
        Self {
            executed_opcodes: Arc::new(Mutex::new(CircularBuffer::new())),
        }
    }

    pub fn register_handlers(&mut self, emulator_ctx: &mut Gba) {
        let executed_opcodes = self.executed_opcodes.clone();
        emulator_ctx.event_bus.register_handler(EXECUTED_OPCODE_EVENT_ID, Arc::new(move |event: &dyn Event| {
            executed_opcodes.lock().unwrap().push_back(*event.payload().unwrap().get_ref().unwrap());
        }));
    }
}
