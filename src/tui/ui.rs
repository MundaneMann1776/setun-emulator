//! UI rendering for the debugger.

use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, List, ListItem},
    style::{Color, Style, Modifier},
};
use crate::Trit;
use super::app::DebuggerApp;

/// Main draw function.
pub fn draw(frame: &mut Frame, app: &DebuggerApp) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(frame.area());
    
    // Left side: code and status
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(8),
            Constraint::Length(3),
        ])
        .split(chunks[0]);
    
    draw_disassembly(frame, left_chunks[0], app);
    draw_registers(frame, left_chunks[1], app);
    draw_status(frame, left_chunks[2], app);
    
    // Right side: memory and help
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(6),
        ])
        .split(chunks[1]);
    
    draw_memory(frame, right_chunks[0], app);
    draw_help(frame, right_chunks[1]);
}

/// Draw disassembly view with colored trits.
fn draw_disassembly(frame: &mut Frame, area: Rect, app: &DebuggerApp) {
    let disasm = app.get_disassembly(area.height as usize - 2);
    
    let items: Vec<ListItem> = disasm
        .iter()
        .map(|(addr, instr, is_current)| {
            let prefix = if *is_current { "▶ " } else { "  " };
            let bp = if app.breakpoints.contains(addr) { "●" } else { " " };
            let text = format!("{}{:03}: {}", prefix, addr, instr);
            
            let style = if *is_current {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if app.breakpoints.contains(addr) {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };
            
            ListItem::new(format!("{} {}", bp, text)).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title(" Disassembly ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)));
    
    frame.render_widget(list, area);
}

/// Draw register state with trit coloring.
fn draw_registers(frame: &mut Frame, area: Rect, app: &DebuggerApp) {
    
    let content = vec![
        Line::from(vec![
            Span::raw("S: "),
            Span::styled(format!("{:>20}", app.cpu.regs.s), Style::default().fg(Color::White)),
            Span::raw(format!(" = {}", app.cpu.regs.s.to_i64())),
        ]),
        Line::from(vec![
            Span::raw("R: "),
            Span::styled(format!("{:>20}", app.cpu.regs.r), Style::default().fg(Color::White)),
            Span::raw(format!(" = {}", app.cpu.regs.r.to_i64())),
        ]),
        Line::from(vec![
            Span::raw("F: "),
            Span::styled(format!("{:>5}", app.cpu.regs.f.to_i32()), Style::default().fg(Color::White)),
            Span::raw("   C: "),
            Span::styled(format!("{}", app.cpu.regs.c.to_i32()), Style::default().fg(Color::Yellow)),
            Span::raw("   ω: "),
            Span::styled(format!("{:?}", app.cpu.regs.omega), trit_style(app.cpu.regs.omega)),
        ]),
        Line::from(vec![
            Span::raw("Cycles: "),
            Span::styled(format!("{}", app.cpu.cycles), Style::default().fg(Color::Cyan)),
            Span::raw("   State: "),
            Span::styled(format!("{:?}", app.cpu.state), 
                if app.cpu.is_running() { 
                    Style::default().fg(Color::Green) 
                } else { 
                    Style::default().fg(Color::Red) 
                }),
        ]),
    ];
    
    let paragraph = Paragraph::new(content)
        .block(Block::default()
            .title(" Registers ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Green)));
    
    frame.render_widget(paragraph, area);
}

/// Draw memory view.
fn draw_memory(frame: &mut Frame, area: Rect, app: &DebuggerApp) {
    let visible_rows = (area.height as usize).saturating_sub(2);
    let start = app.mem_scroll;
    let end = (start + visible_rows).min(162);
    
    let items: Vec<ListItem> = (start..end)
        .map(|idx| {
            let value = app.cpu.mem.read(idx);
            let addr = idx as i32 - 81;
            let is_pc = addr == app.cpu.regs.c.to_i32();
            
            let text = format!("{:03}: {} = {}", addr, value, value.to_i32());
            
            let style = if is_pc {
                Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
            } else if !value.is_zero() {
                Style::default().fg(Color::White)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            ListItem::new(text).style(style)
        })
        .collect();
    
    let list = List::new(items)
        .block(Block::default()
            .title(" Memory ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta)));
    
    frame.render_widget(list, area);
}

/// Draw status bar.
fn draw_status(frame: &mut Frame, area: Rect, app: &DebuggerApp) {
    let status = Paragraph::new(app.status.clone())
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .title(" Status ")
            .borders(Borders::ALL));
    
    frame.render_widget(status, area);
}

/// Draw help panel.
fn draw_help(frame: &mut Frame, area: Rect) {
    let help = Paragraph::new(vec![
        Line::from("s: Step  r: Run  p: Pause  b: Breakpoint"),
        Line::from("x: Reset  ↑↓: Scroll memory  q: Quit"),
    ])
    .style(Style::default().fg(Color::DarkGray))
    .block(Block::default()
        .title(" Help ")
        .borders(Borders::ALL));
    
    frame.render_widget(help, area);
}

/// Get color style for a trit.
fn trit_style(t: Trit) -> Style {
    match t {
        Trit::N => Style::default().fg(Color::Red),
        Trit::O => Style::default().fg(Color::Gray),
        Trit::P => Style::default().fg(Color::Green),
    }
}
