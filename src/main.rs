use std::{error::Error, io};

use clap::Parser;

use ignore::WalkBuilder;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    symbols::border,
    text::Line,
    widgets::{
        block::{Position, Title},
        Block, List, ListDirection, Widget,
    },
    DefaultTerminal, Frame,
};

use grep::{
    regex::RegexMatcher,
    searcher::{sinks::UTF8, BinaryDetection, SearcherBuilder},
};

use std::path::PathBuf;

/// doot is a todo manager
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// find all todos under this path
    #[arg(default_value = ".")]
    path: PathBuf,
}

fn init_cli() -> Cli {
    let cli = Cli::parse();

    cli
}

fn main() -> io::Result<()> {
    let cli = init_cli();

    let mut terminal = ratatui::init();
    terminal.clear()?;
    let mut app = App::default();
    app.path = cli.path;
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result
}

#[derive(Debug, Default)]
pub struct App {
    path: PathBuf,
    todos: Vec<String>,
    exit: bool,
}

impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.find_todos();
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let event::Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                if key.code == KeyCode::Char('q') {
                    self.exit = true;
                }
            }
        }

        return Ok(());
    }

    fn find_todos(&mut self) -> Result<(), Box<dyn Error>> {
        // TODO: make this regex configurable
        let matcher = RegexMatcher::new_line_matcher("^\\s*// TODO")?;
        let mut searcher = SearcherBuilder::new()
            .binary_detection(BinaryDetection::quit(b'\x00'))
            .line_number(true)
            .build();

        let walker = WalkBuilder::new(self.path.clone()).build();
        for result in walker {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };

            if !dent.file_type().unwrap().is_file() {
                continue;
            }
            let result = searcher.search_path(
                &matcher,
                dent.path(),
                UTF8(|_lnum, line| {
                    self.todos.push(
                        line.trim()
                            .strip_prefix("// TODO")
                            .unwrap_or_default()
                            .strip_prefix(":")
                            .unwrap_or_default()
                            .trim()
                            .to_string(),
                    );
                    Ok(true)
                }),
            );
            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }

        return Ok(());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" doot 💀🎺 ".bold());
        let instructions = Title::from(Line::from(vec![
            " Settings ".into(),
            "<S> ".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .border_set(border::THICK);

        List::new(self.todos.clone())
            .block(block)
            .style(Style::new().white())
            .highlight_style(Style::new().italic())
            .highlight_symbol(">>")
            .repeat_highlight_symbol(true)
            .direction(ListDirection::BottomToTop)
            .render(area, buf)
    }
}
