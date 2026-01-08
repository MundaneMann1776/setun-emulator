//! Debugger application state and logic.

use crate::{Cpu, Tryte9, Instruction};
use crate::asm::disasm::disassemble_instruction;
use crate::cpu::decode::encode;
use std::collections::HashSet;

/// Debugger application state.
pub struct DebuggerApp {
    /// The CPU being debugged.
    pub cpu: Cpu,
    /// Original program for reference.
    pub program: Vec<Tryte9>,
    /// Breakpoints (by address).
    pub breakpoints: HashSet<i32>,
    /// Is the debugger running continuously?
    pub running: bool,
    /// Should we quit?
    pub should_quit: bool,
    /// Status message to display.
    pub status: String,
    /// Memory view scroll offset.
    pub mem_scroll: usize,
    /// Selected memory address.
    pub selected_addr: usize,
}

impl DebuggerApp {
    /// Create a new debugger with a loaded program.
    pub fn new(program: Vec<Tryte9>) -> Self {
        let mut cpu = Cpu::new();
        let _ = cpu.load_program(&program);
        
        Self {
            cpu,
            program,
            breakpoints: HashSet::new(),
            running: false,
            should_quit: false,
            status: "Ready. Press 's' to step, 'r' to run, 'q' to quit.".into(),
            mem_scroll: 0,
            selected_addr: 81, // Address 0 (middle of memory)
        }
    }
    
    /// Step one instruction.
    pub fn step(&mut self) {
        if !self.cpu.is_running() {
            self.status = format!("CPU halted: {:?}", self.cpu.state);
            self.running = false;
            return;
        }
        
        let pc = self.cpu.regs.c.to_i32();
        match self.cpu.step() {
            Ok(instr) => {
                let disasm = disassemble_instruction(encode(&instr));
                self.status = format!("PC={:03}: {}", pc, disasm);
            }
            Err(e) => {
                self.status = format!("Error: {}", e);
                self.running = false;
            }
        }
    }
    
    /// Run until halt, breakpoint, or error.
    pub fn run(&mut self) {
        self.running = true;
        self.status = "Running...".into();
    }
    
    /// Run one iteration of continuous execution.
    pub fn tick(&mut self) {
        if !self.running {
            return;
        }
        
        if !self.cpu.is_running() {
            self.running = false;
            self.status = format!("Halted after {} cycles", self.cpu.cycles);
            return;
        }
        
        // Check for breakpoint
        let pc = self.cpu.regs.c.to_i32();
        if self.breakpoints.contains(&pc) {
            self.running = false;
            self.status = format!("Breakpoint at PC={}", pc);
            return;
        }
        
        self.step();
    }
    
    /// Toggle breakpoint at current PC or selected address.
    pub fn toggle_breakpoint(&mut self) {
        let pc = self.cpu.regs.c.to_i32();
        if self.breakpoints.contains(&pc) {
            self.breakpoints.remove(&pc);
            self.status = format!("Removed breakpoint at PC={}", pc);
        } else {
            self.breakpoints.insert(pc);
            self.status = format!("Set breakpoint at PC={}", pc);
        }
    }
    
    /// Reset CPU to initial state.
    pub fn reset(&mut self) {
        self.cpu = Cpu::new();
        let _ = self.cpu.load_program(&self.program);
        self.running = false;
        self.status = "Reset. Ready.".into();
    }
    
    /// Get disassembly around current PC.
    pub fn get_disassembly(&self, lines: usize) -> Vec<(i32, String, bool)> {
        let pc = self.cpu.regs.c.to_i32();
        let start = (pc - (lines as i32 / 2)).max(-81);
        
        (0..lines as i32)
            .filter_map(|i| {
                let addr = start + i;
                let idx = (addr + 81) as usize;
                if idx < 162 {
                    let instr = self.cpu.mem.read(idx);
                    let disasm = disassemble_instruction(instr);
                    let is_current = addr == pc;
                    Some((addr, disasm, is_current))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Run the debugger with a program.
pub fn run_debugger(program: Vec<Tryte9>) -> std::io::Result<()> {
    use crossterm::{
        event::{self, Event, KeyCode, KeyEventKind},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    };
    use ratatui::prelude::*;
    use std::io::stdout;
    use std::time::Duration;
    
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    
    // Create app
    let mut app = DebuggerApp::new(program);
    
    // Main loop
    loop {
        // Draw
        terminal.draw(|frame| {
            super::ui::draw(frame, &app);
        })?;
        
        // Handle input
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') => app.should_quit = true,
                        KeyCode::Char('s') => {
                            app.running = false;
                            app.step();
                        }
                        KeyCode::Char('r') => app.run(),
                        KeyCode::Char('p') => {
                            app.running = false;
                            app.status = "Paused.".into();
                        }
                        KeyCode::Char('b') => app.toggle_breakpoint(),
                        KeyCode::Char('x') => app.reset(),
                        KeyCode::Up => {
                            if app.mem_scroll > 0 {
                                app.mem_scroll -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if app.mem_scroll < 150 {
                                app.mem_scroll += 1;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        
        // Tick for continuous running
        if app.running {
            app.tick();
        }
        
        if app.should_quit {
            break;
        }
    }
    
    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    
    Ok(())
}
