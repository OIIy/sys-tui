use std::io;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::{Line, Span, Text},
    widgets::{
        block::{Position, Title},
        Block, Paragraph, Widget, Wrap,
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
        frame.render_widget(self, frame.size());
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

impl Widget for &App<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(self.name.clone().bold());
        let text = vec![
            Line::from(vec![
                Span::raw("Total Memory (GB): "),
                Span::styled(
                    to_gigabytes(self.system.total_memory()).to_string(),
                    Style::new().green().italic(),
                ),
            ]),
            Line::from(vec![
                Span::raw("Used Memory (GB): "),
                Span::styled(
                    to_gigabytes(self.system.used_memory()).to_string(),
                    Style::new().green().italic(),
                ),
            ]),
        ];
        let info = Paragraph::new(text)
            .block(Block::bordered().title("Paragraph"))
            .style(Style::new().white().on_black())
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });

        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        let info_block = info.block(block);

        info_block.render(area, buf)
    }
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
