use std::io::{self, Write};
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::game::{Game, GamePhase};
use crate::ui::draw;

pub fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(1000 / 30);
    let mut last_tick = Instant::now();

    let mut game = Game::new();

    let res = loop {
        terminal.draw(|f| draw(f, &game)).ok();

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    // ignore key repeats from holding a key
                    if key.kind != KeyEventKind::Press {
                        continue;
                    }
                    let in_name = matches!(game.phase, GamePhase::NameEntry);
                    match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break Ok(()),
                    // While entering name, any Char is treated as input
                    KeyCode::Char(c) if in_name => { game.name_input_char(c); }
                    KeyCode::Char('n') if !in_name => {
                        match game.phase {
                            GamePhase::Menu => { game.phase = GamePhase::NameEntry; game.name_input.clear(); }
                            GamePhase::GameOver | GamePhase::Leaderboard => { game.phase = GamePhase::NameEntry; game.name_input.clear(); }
                            _ => game.new_run(),
                        }
                    }
                    KeyCode::Char('?') => game.toggle_help(),
                    // Menu navigation and Game Over scroll
                    KeyCode::Down => {
                        match game.phase {
                            GamePhase::Menu => game.select_menu_down(),
                            GamePhase::GameOver => { game.game_over_scroll = game.game_over_scroll.saturating_add(1); }
                            _ => {}
                        }
                    }
                    KeyCode::Up => {
                        match game.phase {
                            GamePhase::Menu => game.select_menu_up(),
                            GamePhase::GameOver => { game.game_over_scroll = game.game_over_scroll.saturating_sub(1); }
                            _ => {}
                        }
                    }
                    KeyCode::Enter => {
                        match game.phase {
                            GamePhase::Menu => game.menu_activate(),
                            GamePhase::NameEntry => game.name_input_submit(),
                            GamePhase::Running => game.take_selected_default(),
                            GamePhase::Leaderboard | GamePhase::GameOver => { /* no-op */ }
                        }
                    }
                    KeyCode::Backspace => {
                        if matches!(game.phase, GamePhase::NameEntry) { game.name_input_backspace(); }
                    }
                    KeyCode::Char(' ') if !in_name => { if matches!(game.phase, GamePhase::Running) { game.take_selected_default(); } }
                    KeyCode::Char('v') if !in_name => if matches!(game.phase, GamePhase::Running) { game.avoid_room() },
                    KeyCode::Right => if matches!(game.phase, GamePhase::Running) { game.move_selection(1, 0) },
                    KeyCode::Left => if matches!(game.phase, GamePhase::Running) { game.move_selection(-1, 0) },
                    KeyCode::Char('b') if !in_name => if matches!(game.phase, GamePhase::Running) { game.take_selected_barehand() },
                    KeyCode::Char('w') if !in_name => if matches!(game.phase, GamePhase::Running) { game.take_selected_weapon() },
                    // Quick pick shortcuts: 1-4 select slot and take default action
                    KeyCode::Char('1') if !in_name => { if matches!(game.phase, GamePhase::Running) { game.selected = 0; game.take_selected_default(); } },
                    KeyCode::Char('2') if !in_name => { if matches!(game.phase, GamePhase::Running) { game.selected = 1; game.take_selected_default(); } },
                    KeyCode::Char('3') if !in_name => { if matches!(game.phase, GamePhase::Running) { game.selected = 2; game.take_selected_default(); } },
                    KeyCode::Char('4') if !in_name => { if matches!(game.phase, GamePhase::Running) { game.selected = 3; game.take_selected_default(); } },
                    KeyCode::Char('l') if !in_name => { game.phase = GamePhase::Leaderboard; },
                    KeyCode::Char('m') if !in_name => { game.phase = GamePhase::Menu; },
                    KeyCode::Char('r') if !in_name => game = Game::new(),
                    KeyCode::PageUp => { if matches!(game.phase, GamePhase::GameOver) { game.game_over_scroll = game.game_over_scroll.saturating_sub(10); } },
                    KeyCode::PageDown => { if matches!(game.phase, GamePhase::GameOver) { game.game_over_scroll = game.game_over_scroll.saturating_add(10); } },
                    KeyCode::Home => { if matches!(game.phase, GamePhase::GameOver) { game.game_over_scroll = 0; } },
                    KeyCode::End => { if matches!(game.phase, GamePhase::GameOver) { game.game_over_scroll = u16::MAX; } },
                    _ => {}
                }
                }
                Event::Mouse(me) => {
                    match me.kind {
                        MouseEventKind::ScrollUp => {
                            if matches!(game.phase, GamePhase::GameOver) {
                                game.game_over_scroll = game.game_over_scroll.saturating_sub(3);
                            }
                        }
                        MouseEventKind::ScrollDown => {
                            if matches!(game.phase, GamePhase::GameOver) {
                                game.game_over_scroll = game.game_over_scroll.saturating_add(3);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
            game.tick();
            if matches!(game.phase, GamePhase::GameOver) {
                // keep running until user presses 'n' or 'q'
            }
        }
    };

    // Restore terminal
    cleanup_terminal()
        .and(res)
}

fn cleanup_terminal() -> Result<()> {
    disable_raw_mode()?;
    let mut stdout = io::stdout();
    queue!(stdout, LeaveAlternateScreen, DisableMouseCapture)?;
    stdout.flush()?;
    Ok(())
}
