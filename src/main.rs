use std::{
    io,
    time::{Duration, Instant},
};

use chrono::{Local, Utc};
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
pub struct Clock {}

impl Widget for &Clock {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let tz = Local::now().naive_local();

        let time_str = tz.format("%H:%M:%S").to_string();

        let time = Paragraph::new(time_str);

        time.render(area, buf)
    }
}

#[derive(Debug)]
pub struct App<'a> {
    name: String,
    clock: Clock,
    system: &'a mut System,
    exit: bool,
}

impl App<'_> {
    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut tui::Tui, tick_rate: Duration) -> io::Result<()> {
        let last_tick = Instant::now();

        loop {
            if self.exit {
                return Ok(());
            }

            terminal.draw(|frame| self.render_frame(frame))?;

            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or(Duration::ZERO);

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    if let KeyCode::Char('q') = key.code {
                        self.exit = true;
                    }
                }
            }
        }
    }

    fn render_frame(&mut self, frame: &mut Frame) {
        let mut cols: Vec<Constraint> = vec![];

        std::thread::sleep(Duration::from_secs(1));

        self.system.refresh_cpu_all();

        for _cpu in self.system.cpus() {
            let col_size: usize = 100 / self.system.cpus().len();
            cols.push(Constraint::Percentage(col_size.try_into().unwrap()));
        }

        let outer_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(5), Constraint::Percentage(95)])
            .split(frame.size());

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(cols)
            .split(outer_layout[1]);

        self.render_clock(frame, outer_layout[0]);

        for (index, cpu) in self.system.cpus().iter().enumerate() {
            self.render_cpu(frame, cpu, inner_layout[index]);
        }
    }

    fn render_cpu(&self, frame: &mut Frame, cpu: &Cpu, area: Rect) {
        let cpu_block = Block::new().title(cpu.name()).borders(Borders::ALL);
        let cpu_widget = Paragraph::new(cpu.cpu_usage().to_string()).block(cpu_block);

        frame.render_widget(cpu_widget, area)
    }

    fn render_clock(&self, frame: &mut Frame, area: Rect) {
        let tz = Local::now().naive_local();

        let time_str = tz.format("%H:%M:%S").to_string();

        let time = Paragraph::new(time_str);

        frame.render_widget(time, area)
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

    let tick_rate = Duration::from_millis(250);

    let mut app = App {
        clock: Clock {},
        name: System::host_name().expect("Could not get name of host."),
        system: &mut sys,
        exit: false,
    };

    let mut terminal = tui::init()?;
    let app_result = app.run(&mut terminal, tick_rate);
    tui::restore()?;
    app_result
}
