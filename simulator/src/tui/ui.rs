use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::disasm;
use crate::tui::app::{App, AppMode};

pub fn render(f: &mut Frame, app: &App) {
    let area = f.area();

    // ── Outer vertical split ──────────────────────────────────────────────────
    // [header 1][body *][output 8][cmdbar 3]
    let outer = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(8),
            Constraint::Length(3),
        ])
        .split(area);

    render_header(f, app, outer[0]);

    // ── Body: registers (left) + memory (right) ───────────────────────────────
    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(26), Constraint::Min(20)])
        .split(outer[1]);

    render_registers(f, app, body[0]);
    render_memory(f, app, body[1]);
    render_output(f, app, outer[2]);
    render_cmdbar(f, app, outer[3]);
}

// ── Header ────────────────────────────────────────────────────────────────────

fn render_header(f: &mut Frame, app: &App, area: Rect) {
    let waiting = if app.machine.waiting_for_input {
        "  [waiting for input]"
    } else {
        ""
    };
    let text = format!(
        " LC-3 Simulator  │  steps: {}  │  {}{}",
        app.machine.step_count, app.status, waiting
    );
    let style = if app.machine.halted {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else if app.running {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::Cyan)
    };
    f.render_widget(Paragraph::new(text).style(style), area);
}

// ── Registers panel ───────────────────────────────────────────────────────────

fn render_registers(f: &mut Frame, app: &App, area: Rect) {
    let m = &app.machine;
    let mut lines: Vec<Line> = (0..8u8)
        .map(|i| {
            let v = m.regs.gpr[i as usize];
            Line::from(format!(" R{i}  x{v:04X}  {:>6}", signed_repr(v)))
        })
        .collect();

    lines.push(Line::from(""));
    lines.push(Line::from(format!(" PC  x{:04X}", m.regs.pc)));
    lines.push(Line::from(format!(" CC  {}", m.regs.cc)));

    if !m.breakpoints.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            " Breakpoints:",
            Style::default().fg(Color::Yellow),
        )));
        let mut bps: Vec<u16> = m.breakpoints.iter().copied().collect();
        bps.sort_unstable();
        for bp in bps {
            lines.push(Line::from(format!("  x{bp:04X}")));
        }
    }

    let block = Block::default().title(" Registers ").borders(Borders::ALL);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

/// Show a u16 as its signed decimal equivalent for the register panel.
fn signed_repr(v: u16) -> String {
    let s = v as i16;
    if s < 0 {
        format!("({s})")
    } else {
        format!("{s}")
    }
}

// ── Memory panel ──────────────────────────────────────────────────────────────

fn render_memory(f: &mut Frame, app: &App, area: Rect) {
    // Inner height = panel height minus top/bottom border lines.
    let visible = area.height.saturating_sub(2);

    let items: Vec<ListItem> = (0..visible)
        .map(|i| {
            let addr = app.mem_scroll.wrapping_add(i);
            let word = app.machine.mem.raw(addr);
            let dis = disasm::disassemble(word, addr, Some(&app.sym_table));
            let is_pc = addr == app.machine.regs.pc;
            let is_bp = app.machine.breakpoints.contains(&addr);

            let arrow = if is_pc { "►" } else { " " };
            let bp_mark = if is_bp { "●" } else { " " };
            let text = format!("{bp_mark}{arrow} x{addr:04X}  {word:04X}  {dis}");

            let style = if is_pc {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else if is_bp {
                Style::default().fg(Color::Red)
            } else {
                Style::default()
            };

            ListItem::new(text).style(style)
        })
        .collect();

    let block = Block::default()
        .title(" Memory  [↑↓ scroll · g x<addr> go] ")
        .borders(Borders::ALL);
    f.render_widget(List::new(items).block(block), area);
}

// ── Output panel ──────────────────────────────────────────────────────────────

fn render_output(f: &mut Frame, app: &App, area: Rect) {
    let inner_h = area.height.saturating_sub(2) as usize;
    let m = &app.machine;

    // Collect all completed lines plus the partial current line.
    let mut all: Vec<&str> = m.output_lines.iter().map(|s| s.as_str()).collect();
    if !m.output_buf.is_empty() {
        all.push(&m.output_buf);
    }

    // Show only the last `inner_h` lines.
    let skip = all.len().saturating_sub(inner_h);
    let lines: Vec<Line> = all[skip..].iter().map(|s| Line::from(*s)).collect();

    let block = Block::default().title(" Output ").borders(Borders::ALL);
    f.render_widget(Paragraph::new(lines).block(block), area);
}

// ── Command bar ───────────────────────────────────────────────────────────────

fn render_cmdbar(f: &mut Frame, app: &App, area: Rect) {
    let hint = " [s]tep  [c]ont  [p]ause  [r]eset  [b]reak  [g]oto  [q]uit";

    let input_line = match app.mode {
        AppMode::CommandInput => format!("> {}_", app.cmd_input),
        AppMode::Normal => {
            if app.machine.waiting_for_input {
                "> (type a character for GETC)".into()
            } else {
                String::new()
            }
        }
    };

    let lines = vec![Line::from(hint), Line::from(input_line)];
    let block = Block::default().title(" Command ").borders(Borders::ALL);
    f.render_widget(Paragraph::new(lines).block(block), area);
}
