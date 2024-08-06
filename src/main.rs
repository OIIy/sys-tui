use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Borders, Paragraph, Widget, Wrap,
    },
    Frame,
};

use sysinfo::{Cpu, System};

mod tui;

#[derive(Debug)]
pub struct App<'a> {
    name: String,
    system: &'a System,
    exit: bool,
}

impl App<'_> {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        // frame.render_widget(self, frame.size());
        let mut cols: Vec<Constraint> = vec![];

        for cpu in self.system.cpus() {
            let col_size: usize = 100 / self.system.cpus().len();
            cols.push(Constraint::Percentage(col_size.try_into().unwrap()));
        }

        // Horizontal for columns, Vertical for rows
        let outer_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(cols)
            .split(frame.size());

        for (index, cpu) in self.system.cpus().iter().enumerate() {
            frame.render_widget(
                Paragraph::new(cpu.name()).block(Block::new().borders(Borders::ALL)),
                outer_layout[index],
            );
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.exit();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

fn to_gigabytes(bytes: u64) -> f32 {
    ((bytes as f32 / 1024.0) / 1024.0) / 1024.0
}

fn main() -> io::Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let mut app = App {
        name: System::host_name().expect("Could not get name of host."),
        system: &sys,
        exit: false,
    };

    let mut terminal = tui::init()?;
    let app_result = app.run(&mut terminal);
    tui::restore()?;
    app_result
}
