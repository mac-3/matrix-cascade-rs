use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use matrix_cascade::{widget::MatrixWidget, Matrix};
use rand::seq::IteratorRandom;
use std::{io::Stdout, ops::Range, time::Duration};
use tui::{
    backend::CrosstermBackend,
    buffer::Cell,
    layout::{Constraint, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, Clear, Paragraph},
    Terminal,
};

const CHAR_TABLE: &str = "abcdefghijklmnopqrstuvwxyz0987654321@%/$#";
const LIGHTER_GREEN: Color = Color::Rgb(30, 255, 48);
const DARKER_GREEN: Color = Color::Rgb(18, 166, 31);
const BLACK: Color = Color::Black;
const SLEEP_TIME: Duration = Duration::from_millis(80);

const SPAWN_CHANCE: f64 = 0.01;
const LENGTH_INTERVAL: Range<u16> = 3..15;
const SPEED_INTERVAL: Range<u16> = 1..3;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let size = terminal.size().unwrap();
    let rng = rand::thread_rng();
    let mut matrix_state: Matrix<Cell> = Matrix::new(
        size.height,
        size.width,
        SPAWN_CHANCE,
        LENGTH_INTERVAL,
        SPEED_INTERVAL,
        Box::new(rng),
    );
    let (tcx, rcx) = std::sync::mpsc::channel::<()>();
    let exit_thread = std::thread::spawn(move || loop {
        if let Event::Key(key) = event::read().unwrap() {
            if let KeyCode::Char('q') = key.code {
                tcx.send(()).unwrap();
                break;
            }
        }
    });

    loop {
        terminal.draw(|f| {
            let mut rng = rand::thread_rng();
            let widget = MatrixWidget::new(|c| match c {
                Some(c) if c == 0 => tui::buffer::Cell {
                    symbol: CHAR_TABLE.chars().choose(&mut rng).unwrap().into(),
                    fg: LIGHTER_GREEN,
                    bg: BLACK,
                    ..Default::default()
                },
                Some(_) => tui::buffer::Cell {
                    symbol: CHAR_TABLE.chars().choose(&mut rng).unwrap().into(),
                    fg: DARKER_GREEN,
                    bg: BLACK,
                    ..Default::default()
                },
                None => tui::buffer::Cell {
                    symbol: ' '.into(),
                    bg: BLACK,
                    ..Default::default()
                },
            });
            f.render_stateful_widget(widget, f.size(), &mut matrix_state);
            render_bottom_bar(f);
        })?;

        if rcx.try_recv().is_ok() {
            break;
        }

        std::thread::sleep(SLEEP_TIME);
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();

    exit_thread.join().unwrap();
    Ok(())
}

fn render_bottom_bar(f: &mut tui::Frame<CrosstermBackend<Stdout>>) {
    let columns = Layout::default()
        .direction(tui::layout::Direction::Vertical)
        .constraints(
            [
                Constraint::Length(if f.size().height >= 3 {
                    f.size().height - 3
                } else {
                    0
                }),
                Constraint::Length(3),
            ]
            .as_ref(),
        )
        .split(f.size());
    let exit_hint_block = Block::default().borders(Borders::ALL).style(Style {
        fg: Some(Color::Reset),
        bg: Some(BLACK),
        ..Default::default()
    });
    f.render_widget(Clear, exit_hint_block.inner(columns[1]));
    let exit_hint_span = Span::raw("Press 'q' to exit");
    let exit_hint_paragraph = Paragraph::new(exit_hint_span).block(exit_hint_block);
    f.render_widget(exit_hint_paragraph, columns[1]);
}
