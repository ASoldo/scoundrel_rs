use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::widgets::block::BorderType;

use crate::cards::{Card, Rank, Suit};
use crate::game::{Game, GameEvent, GamePhase};

pub fn draw(f: &mut Frame, game: &Game) {
    let size = f.area();
    match game.phase {
        GamePhase::Menu => {
            draw_menu(f, size, game);
            if game.show_help { draw_help(f, centered_rect(70, 70, size)); }
        }
        GamePhase::NameEntry => {
            draw_name_entry(f, size, game);
            if game.show_help { draw_help(f, centered_rect(70, 70, size)); }
        }
        GamePhase::Leaderboard => {
            draw_leaderboard(f, size, game);
            if game.show_help { draw_help(f, centered_rect(70, 70, size)); }
        }
        GamePhase::GameOver => {
            draw_game_over(f, size, game);
            if game.show_help { draw_help(f, centered_rect(70, 70, size)); }
        }
        GamePhase::Running => {
            // Outer bordered frame for consistent visual identity
            let outer = Block::default()
                .borders(Borders::ALL)
                .title("Scoundrel")
                .border_style(Style::default().fg(Color::White));
            let inner = outer.inner(size);
            f.render_widget(outer, size);

            // Room, Status, Equipped (no inline help hint)
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(60),
                    Constraint::Length(3),
                    Constraint::Percentage(37),
                ])
                .split(inner);
            draw_room(f, layout[0], game);
            draw_status(f, layout[1], game);
            draw_equipped(f, layout[2], game);
            if game.show_help { draw_help(f, centered_rect(70, 70, inner)); }
            // Bottom-border right-aligned help hint on the outer frame
            let border_hint_area = Rect {
                x: size.x.saturating_add(1),
                y: size.y.saturating_add(size.height.saturating_sub(1)),
                width: size.width.saturating_sub(2),
                height: 1,
            };
            let hint = Paragraph::new(Span::styled("? - help", Style::default().fg(Color::Gray))).alignment(Alignment::Right);
            f.render_widget(hint, border_hint_area);
        }
    }
}

fn draw_menu(f: &mut Frame, area: Rect, game: &Game) {
    // Outer frame
    let outer = Block::default()
        .borders(Borders::ALL)
        .title("Scoundrel")
        .border_style(Style::default().fg(Color::White));
    let inner = outer.inner(area);
    f.render_widget(outer, area);
    // fill subtle dots across the entire Scoundrel box background
    render_subtle_pattern(f, inner);

    // Center a box with ASCII art + options and render subtle background dots inside it
    let content = centered_rect_fixed(54, 12, inner);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // ASCII art (5 lines)
            Constraint::Length(1), // spacer
            Constraint::Length(1), // option 1
            Constraint::Length(1), // option 2
            Constraint::Length(1), // option 3
            Constraint::Min(0),
        ])
        .split(content);

    // ASCII Art Title (provided)
    let art = vec![
        "  ____                            _          _ ",
        " / ___|  ___ ___  _   _ _ __   __| |_ __ ___| |",
        " \\___ \\ / __/ _ \\| | | | '_ \\ / _` | '__/ _ \\ |",
        "  ___) | (_| (_) | |_| | | | | (_| | | |  __/ |",
        " |____/ \\___\\___/ \\__,_|_| |_|\\__,_|_|  \\___|_|",
    ];
    let art_lines: Vec<Line> = art.into_iter().map(|s| Line::from(Span::raw(s))).collect();
    let p_art = Paragraph::new(Text::from(art_lines)).alignment(Alignment::Center);
    f.render_widget(p_art, v[0]);

    // Options
    let opts = ["New Game", "Leaderboard", "Quit"];
    for (i, label) in opts.iter().enumerate() {
        let style = if game.menu_selected == i {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };
        let p = Paragraph::new(Line::from(Span::styled(*label, style))).alignment(Alignment::Center);
        f.render_widget(p, v[2 + i]);
    }

    // Bottom-right minimal help hint on the border line itself
    let border_hint_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let hint = Paragraph::new(Span::styled("? - help", Style::default().fg(Color::Gray))).alignment(Alignment::Right);
    f.render_widget(hint, border_hint_area);

    // No menu helpers at bottom-right; help is available via '?'
}

fn draw_name_entry(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default()
        .title("Enter your name")
        .borders(Borders::ALL);
    let outer_inner = block.inner(area);
    f.render_widget(block, area);
    // Subtle background dots across the entire name box
    render_subtle_pattern(f, outer_inner);
    // Compact inline input box
    let inner = centered_rect_fixed(48, 5, area);
    let name = game.name_input.to_string();
    let p = Paragraph::new(Text::from(vec![
        Line::from("Type your run name and press Enter"),
        Line::from(""),
        Line::from(Span::styled(
            name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
    ]))
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::ALL));
    f.render_widget(p, inner);
    // Bottom-border right-aligned help hint
    let border_hint_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let hint = Paragraph::new(Span::styled("? - help", Style::default().fg(Color::Gray))).alignment(Alignment::Right);
    f.render_widget(hint, border_hint_area);
}

fn draw_leaderboard(f: &mut Frame, area: Rect, game: &Game) {
    // Outer box with dots background
    let block = Block::default().title("Leaderboard (Top 10)").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    render_subtle_pattern(f, inner);

    // Center a content region within the leaderboard box
    let content = centered_rect(80, 70, inner);

    // Vertical layout: 1st (top), spacer, 2nd+3rd row, spacer, list, bottom help
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // 1st place box
            Constraint::Length(1),
            Constraint::Length(7), // row with 2nd and 3rd
            Constraint::Length(1),
            Constraint::Min(6), // list
            Constraint::Length(1), // bottom-right help
        ])
        .split(content);

    let entries = &game.leaderboard;

    // Center the 1st place box horizontally
    let first_w: u16 = content.width.clamp(24, 40);
    let first_hsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((layout[0].width.saturating_sub(first_w)) / 2),
            Constraint::Length(first_w),
            Constraint::Min((layout[0].width.saturating_sub(first_w)) / 2),
        ])
        .split(layout[0]);
    draw_podium_box(f, first_hsplit[1], entries.first(), 1, Color::Yellow);

    // Row with 2nd and 3rd, centered as a pair
    let box_w: u16 = ((layout[2].width as f32 * 0.35) as u16).clamp(18, 32);
    let pair_w = box_w * 2 + 2; // include a small gap
    let row_hsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((layout[2].width.saturating_sub(pair_w)) / 2),
            Constraint::Length(box_w),
            Constraint::Length(2), // gap
            Constraint::Length(box_w),
            Constraint::Min((layout[2].width.saturating_sub(pair_w)) / 2),
        ])
        .split(layout[2]);
    draw_podium_box(f, row_hsplit[1], entries.get(1), 2, Color::Gray);
    draw_podium_box(f, row_hsplit[3], entries.get(2), 3, Color::Rgb(205, 127, 50));

    // Remaining list (4..=10), centered block
    let mut lines: Vec<Line> = Vec::new();
    if entries.len() <= 3 {
        lines.push(Line::from("No more scores."));
    } else {
        for (i, entry) in entries.iter().enumerate().skip(3).take(7) {
            let pos = i + 1;
            let emoji = if entry.won { "🏆" } else { "💀" };
            lines.push(Line::from(format!("{:>2}. {} {}  {}", pos, emoji, entry.score, entry.name)));
        }
    }
    let lw: u16 = layout[4].width.clamp(40, 60);
    let list_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((layout[4].width.saturating_sub(lw)) / 2),
            Constraint::Length(lw),
            Constraint::Min((layout[4].width.saturating_sub(lw)) / 2),
        ])
        .split(layout[4]);
    let list_p = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: true });
    f.render_widget(list_p, list_center[1]);

    // Bottom-border right-aligned help hint
    let border_hint_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let hint = Paragraph::new(Span::styled("? - help", Style::default().fg(Color::Gray))).alignment(Alignment::Right);
    f.render_widget(hint, border_hint_area);
}

fn draw_podium_box(
    f: &mut Frame,
    area: Rect,
    entry_opt: Option<&crate::game::ScoreEntry>,
    rank: usize,
    color: Color,
) {
    let title = match rank {
        1 => "1st",
        2 => "2nd",
        3 => "3rd",
        _ => "",
    };
    let b = Block::default()
        .title(Span::styled(title, Style::default().fg(color).add_modifier(Modifier::BOLD)))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color));
    let inner = b.inner(area);
    f.render_widget(b, area);
    // Content
    let mut lines: Vec<Line> = Vec::new();
    if let Some(entry) = entry_opt {
        let medal = match rank { 1 => "🥇", 2 => "🥈", 3 => "🥉", _ => "" };
        lines.push(Line::from(vec![
            Span::styled(medal, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(entry.score.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::raw(entry.name.clone()),
        ]));
    } else {
        lines.push(Line::from("—"));
    }
    let p = Paragraph::new(Text::from(lines)).alignment(Alignment::Center);
    f.render_widget(p, inner);
}

fn draw_game_over(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default().title("Game Over").borders(Borders::ALL);
    let inner = block.inner(area);
    f.render_widget(block, area);
    // subtle dotted background across the game over box
    render_subtle_pattern(f, inner);
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Length(3),
        ])
        .split(inner);

    // Header with result and score and rank
    let score = game.score.unwrap_or(0);
    let rank_text = if let Some(pos) = game.new_rank_pos {
        format!("New rank: #{}", pos + 1)
    } else {
        String::new()
    };
    let title = Paragraph::new(Text::from(vec![
        Line::from(vec![Span::styled(
            format!(
                "{} {} — Score {}",
                if score >= 0 { "🏆" } else { "💀" },
                game.player_name,
                score
            ),
            Style::default().add_modifier(Modifier::BOLD),
        )]),
        Line::from(rank_text),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(title, v[0]);

    // History list grouped by rooms, scrollable
    let history_area = v[1];
    let lines: Vec<Line> = build_history_indented_lines(&game.history);
    // Center the container and center-align text
    let content_w: u16 = history_area.width.clamp(40, 80);
    let hsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min((history_area.width.saturating_sub(content_w)) / 2),
            Constraint::Length(content_w),
            Constraint::Min((history_area.width.saturating_sub(content_w)) / 2),
        ])
        .split(history_area);
    let col_area = hsplit[1];
    let max_scroll = lines.len().saturating_sub(col_area.height as usize) as u16;
    let scroll = game.game_over_scroll.min(max_scroll);
    let hist = Paragraph::new(Text::from(lines))
        .wrap(Wrap { trim: true })
        .alignment(Alignment::Center)
        .scroll((scroll, 0));
    f.render_widget(hist, hsplit[1]);

    // Bottom-border right-aligned help hint
    let border_hint_area = Rect {
        x: area.x.saturating_add(1),
        y: area.y.saturating_add(area.height.saturating_sub(1)),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    let hint = Paragraph::new(Span::styled("? - help", Style::default().fg(Color::Gray))).alignment(Alignment::Right);
    f.render_widget(hint, border_hint_area);
}


fn draw_room(f: &mut Frame, area: Rect, game: &Game) {
    // No enclosing room box; use provided area directly
    let inner = area;

    // 1x4 horizontal layout
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(inner);
    for i in 0..4 {
        let area = cols[i];
        if let Some(card) = game.room[i] {
            let inner_block = if i == game.selected {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow))
            } else {
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Gray))
            };
            f.render_widget(inner_block.clone(), area);
            let inner = inner_block.inner(area);
            // subtle background in cell
            render_subtle_pattern(f, inner);
            let ca = centered_rect_fixed(CARD_W as u16, CARD_H as u16, inner);
            draw_card_box(f, ca, card);
            // Caption label beneath card
            let label_y = (ca.y.saturating_add(ca.height)).min(inner.y.saturating_add(inner.height.saturating_sub(1)));
            let label_area = Rect { x: inner.x, y: label_y, width: inner.width, height: 1 };
            let (label, col) = match card.suit {
                Suit::Hearts => ("Potion", Color::LightRed),
                Suit::Diamonds => ("Weapon", Color::LightBlue),
                Suit::Clubs | Suit::Spades => ("Enemy", Color::LightMagenta),
            };
            let caption = Paragraph::new(Span::styled(label, Style::default().fg(col))).alignment(Alignment::Center);
            f.render_widget(caption, label_area);
            // Overlay selection numbers: top-left and bottom-right inside the cell
            let num = (i + 1).to_string();
            let top_left = Rect { x: inner.x, y: inner.y, width: 2, height: 1 };
            let bot_right = Rect {
                x: inner.x.saturating_add(inner.width.saturating_sub(2)),
                y: inner.y.saturating_add(inner.height.saturating_sub(1)),
                width: 2,
                height: 1,
            };
            let num_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
            f.render_widget(Paragraph::new(Span::styled(num.clone(), num_style)), top_left);
            f.render_widget(Paragraph::new(Span::styled(num, num_style)).alignment(Alignment::Right), bot_right);
        } else {
            let mut b = Block::default()
                .borders(Borders::ALL)
                .title("Empty")
                .border_style(Style::default().fg(Color::Gray));
            if i == game.selected {
                b = b.border_style(Style::default().fg(Color::Red));
            }
            let inner = b.inner(area);
            f.render_widget(b, area);
            render_subtle_pattern(f, inner);
            // Also render quick-pick numbers for empty cells
            let num = (i + 1).to_string();
            let top_left = Rect { x: inner.x, y: inner.y, width: 2, height: 1 };
            let bot_right = Rect {
                x: inner.x.saturating_add(inner.width.saturating_sub(2)),
                y: inner.y.saturating_add(inner.height.saturating_sub(1)),
                width: 2,
                height: 1,
            };
            let num_style = Style::default().fg(Color::Gray).add_modifier(Modifier::BOLD);
            f.render_widget(Paragraph::new(Span::styled(num.clone(), num_style)), top_left);
            f.render_widget(Paragraph::new(Span::styled(num, num_style)).alignment(Alignment::Right), bot_right);
        }
    }
}

fn draw_status(f: &mut Frame, area: Rect, game: &Game) {
    // Build status line: HP, Weapon, Deck. Include projected damage if selecting a monster.
    let player = &game.player;
    let mut hp_proj = String::new();
    if let GamePhase::Running = game.phase
        && let Some(card) = game.room[game.selected]
        && matches!(card.suit, Suit::Clubs | Suit::Spades)
    {
        let mval = card.monster_value() as i32;
        let dmg = if let Some(w) = &player.weapon {
            if w.can_use_on(card.monster_value()) { (mval - w.value as i32).max(0) } else { mval }
        } else { mval };
        if dmg > 0 { hp_proj = format!(" (-{})", dmg); }
    }
    let weapon_str = if let Some(w) = &player.weapon {
        format!("{} (≤ {})", w.value, w.last_monster.map(|v| v.to_string()).unwrap_or_else(|| "∞".into()))
    } else { "-".into() };
    // Determine HP color by percentage: 100% green, >=75% yellow, >=50% orange, else red
    let max_hp = player.max_hp.max(1) as f32;
    let pct = (player.hp as f32 / max_hp).clamp(0.0, 1.0);
    let hp_color = if (pct - 1.0).abs() < f32::EPSILON {
        Color::LightGreen
    } else if pct >= 0.75 {
        Color::Yellow
    } else if pct >= 0.5 {
        Color::Rgb(255, 165, 0) // orange
    } else {
        Color::LightRed
    };
    let mut status_spans: Vec<Span> = Vec::new();
    // HP value with colored status
    status_spans.push(Span::styled(
        format!("HP: {}/{}", player.hp, player.max_hp),
        Style::default().fg(hp_color).add_modifier(Modifier::BOLD),
    ));
    // Damage preview always in red (if present)
    if !hp_proj.is_empty() {
        status_spans.push(Span::styled(hp_proj.clone(), Style::default().fg(Color::LightRed).add_modifier(Modifier::BOLD)));
    }
    status_spans.push(Span::raw("  |  "));
    status_spans.push(Span::styled(format!("Weapon: {}", weapon_str), Style::default().fg(Color::LightBlue)));
    status_spans.push(Span::raw("  |  "));
    status_spans.push(Span::styled(format!("Deck: {}", game.deck.len()), Style::default().fg(Color::Gray)));
    status_spans.push(Span::raw("  |  "));
    status_spans.push(Span::styled(format!("Room {}", game.room_number), Style::default().fg(Color::Gray)));
    let line = Line::from(status_spans);
    // Draw status block and background pattern, then center content inside
    let block = Block::default()
        .title("Status")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let inner = block.inner(area);
    f.render_widget(block, area);
    render_subtle_pattern(f, inner);
    let p = Paragraph::new(Text::from(vec![line])).alignment(Alignment::Center);
    f.render_widget(p, inner);
}

fn draw_equipped(f: &mut Frame, area: Rect, game: &Game) {
    let block = Block::default()
        .title("Equipped & Slain")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    f.render_widget(block.clone(), area);
    let inner = block.inner(area);
    // subtle background across the equipped box
    render_subtle_pattern(f, inner);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(16), Constraint::Min(10)])
        .split(inner);

    // Left: big equipped card centered
    if let Some(w) = &game.player.weapon {
        let eq_card = Card {
            suit: Suit::Diamonds,
            rank: Rank::new(w.value),
        };
        let ca = centered_rect_fixed(CARD_W as u16, CARD_H as u16, cols[0]);
        draw_card_box(f, ca, eq_card);
    } else {
        let ca = centered_rect_fixed(CARD_W as u16, CARD_H as u16, cols[0]);
        draw_empty_card_box(f, ca);
    }

    // Right: draw slain mini-cards directly beside equipped card, left-to-right (kept small and vertically centered)
    if let Some(w) = &game.player.weapon {
        let mini_w = MINI_W as u16;
        let mini_h = MINI_H as u16;
        let available = cols[1];
        // Center a single row vertically with height MINI_H
        let vcenter = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min((available.height.saturating_sub(mini_h)) / 2),
                Constraint::Length(mini_h),
                Constraint::Min((available.height.saturating_sub(mini_h)) / 2),
            ])
            .split(available);
        let row_area = vcenter[1];
        // Build left-to-right mini-card slots within row_area
        let mut constraints: Vec<Constraint> = Vec::new();
        constraints.push(Constraint::Length(2)); // small left offset
        let n_fit = ((row_area.width.saturating_sub(2)) / mini_w).clamp(0, 24);
        for _ in 0..n_fit { constraints.push(Constraint::Length(mini_w)); }
        constraints.push(Constraint::Min(0));
        let row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(constraints)
            .split(row_area);
        let mut iter = w.stack.iter().rev(); // most recent first
        for i in 0..n_fit as usize {
            if let Some(m) = iter.next() {
                let cell = row[1 + i];
                draw_mini_card_box(f, cell, *m);
            }
        }
    } else {
        // no message; keep subtle background only
    }
}

fn render_subtle_pattern(f: &mut Frame, area: Rect) {
    if area.width == 0 || area.height == 0 { return; }
    // Build a faint dot pattern (e.g., ". · . ·") with alternating rows
    let mut lines: Vec<Line> = Vec::with_capacity(area.height as usize);
    let w = area.width as usize;
    for y in 0..area.height {
        let offset = (y % 2) as usize;
        let mut s = String::with_capacity(w);
        for x in 0..w {
            // pattern: place a dot every 2 columns, stagger by row
            if (x + offset) % 2 == 0 { s.push('·'); } else { s.push(' '); }
        }
        lines.push(Line::from(Span::styled(s, Style::default().fg(Color::DarkGray))));
    }
    f.render_widget(Paragraph::new(Text::from(lines)), area);
}


const CARD_W: usize = 11;
const CARD_H: usize = 7; // top, rank_l, empty, suit, empty, rank_r, bottom
const MINI_W: usize = 5;
const MINI_H: usize = 4; // content box target height

// Draw a rounded, white-bordered card with colored suit and ranks
fn draw_card_box(f: &mut Frame, area: Rect, card: Card) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    f.render_widget(block, area);
    // Suit centered in the card (both horizontally and vertically)
    let suit_style = Style::default().fg(card.suit.color()).add_modifier(Modifier::BOLD);
    let suit_p = Paragraph::new(Line::from(Span::styled(card.suit.to_string(), suit_style)))
        .alignment(Alignment::Center);
    let center_area = centered_rect_fixed(inner.width, 1, inner);
    f.render_widget(suit_p, center_area);
    // Rank label (number/letter) in top-left and bottom-right
    let rank_style = Style::default().fg(card.suit.color()).add_modifier(Modifier::BOLD);
    let tl = Rect { x: inner.x, y: inner.y, width: inner.width.min(4), height: 1 };
    f.render_widget(Paragraph::new(Span::styled(card.rank.to_string(), rank_style)), tl);
    let br = Rect {
        x: inner.x.saturating_add(inner.width.saturating_sub(4)),
        y: inner.y.saturating_add(inner.height.saturating_sub(1)),
        width: 4,
        height: 1,
    };
    f.render_widget(Paragraph::new(Span::styled(card.rank.to_string(), rank_style)).alignment(Alignment::Right), br);
}

fn draw_empty_card_box(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let p = Paragraph::new(Span::styled("×", Style::default().fg(Color::DarkGray).add_modifier(Modifier::BOLD)))
        .alignment(Alignment::Center);
    let center_area = centered_rect_fixed(inner.width, 1, inner);
    f.render_widget(p, center_area);
}

// (no big room card rendering; use the same compact card style for room)

fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - pct_y) / 2),
            Constraint::Percentage(pct_y),
            Constraint::Percentage((100 - pct_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - pct_x) / 2),
            Constraint::Percentage(pct_x),
            Constraint::Percentage((100 - pct_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn centered_rect_fixed(w: u16, h: u16, area: Rect) -> Rect {
    // Center a fixed-size rect within area
    let v = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(((area.height as i32 - h as i32).max(0) as u16) / 2),
            Constraint::Length(h),
            Constraint::Min(((area.height as i32 - h as i32).max(0) as u16) / 2),
        ])
        .split(area);
    let hsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(((area.width as i32 - w as i32).max(0) as u16) / 2),
            Constraint::Length(w),
            Constraint::Min(((area.width as i32 - w as i32).max(0) as u16) / 2),
        ])
        .split(v[1]);
    hsplit[1]
}

fn draw_mini_card_box(f: &mut Frame, area: Rect, card: Card) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::White));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let sym = match card.suit { Suit::Clubs => '♣', Suit::Spades => '♠', Suit::Diamonds => '♦', Suit::Hearts => '♥' };
    let col = match card.suit { Suit::Hearts | Suit::Diamonds => Color::Red, _ => Color::Gray };
    // number/value top-left
    let num = if matches!(card.suit, Suit::Clubs | Suit::Spades) { card.monster_value().to_string() } else { card.rank.to_string() };
    let tl = Rect { x: inner.x, y: inner.y, width: inner.width.min(4), height: 1 };
    f.render_widget(Paragraph::new(Span::styled(num, Style::default().fg(col).add_modifier(Modifier::BOLD))), tl);
    // suit bottom-right
    let br = Rect { x: inner.x.saturating_add(inner.width.saturating_sub(2)), y: inner.y.saturating_add(inner.height.saturating_sub(1)), width: 2, height: 1 };
    f.render_widget(Paragraph::new(Span::styled(sym.to_string(), Style::default().fg(col).add_modifier(Modifier::BOLD))).alignment(Alignment::Right), br);
}

// (header/footer helpers removed after layout refactor)

fn draw_help(f: &mut Frame, area: Rect) {
    let text = Text::from(vec![
        Line::from(Span::styled("Scoundrel (terminal)", Style::default().add_modifier(Modifier::BOLD).fg(Color::White))),
        Line::from(""),
        Line::from(
            "Setup: Remove red faces + red aces. Deck = 26 black monsters, 9 diamonds (weapons 2-10), 9 hearts (potions 2-10).",
        ),
        Line::from(
            "Room: 4 face-up. Avoid with v (not twice). Take any 3; 1 carries to next room.",
        ),
        Line::from("Potions: use at most 1 per turn; extra potions are discarded. Max HP 20."),
        Line::from("Weapons: binding. Damage = value; remaining monster damage hits you."),
        Line::from("Rule: after weapon use, can only be used on monsters ≤ last monster’s value."),
        Line::from(
            "End: HP<=0 lose (score = hp - remaining monsters). Empty dungeon win (score = hp, or 20+potion if last was potion).",
        ),
        Line::from(""),
        Line::from(Span::styled("Controls:", Style::default().fg(Color::Gray))),
        Line::from("  Menu: Up/Down + Enter"),
        Line::from("  Game: Left/Right select, Enter take, 1-4 quick pick, w weapon, b barehand, v avoid, ? help, q quit"),
    ]);
    let block = Block::default()
        .title("Help")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Gray));
    let area_inner = block.inner(area);
    f.render_widget(Clear, area);
    f.render_widget(block, area);
    // Dimmer help text for readability across screens
    let help_para = Paragraph::new(text)
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(help_para, area_inner);
}

fn game_event_line(ev: &GameEvent) -> String {
    match ev {
        GameEvent::RoomStart { number } => format!("Room {}", number),
        GameEvent::Potion {
            value,
            hp_before,
            hp_after,
        } => format!("🧪 +{} HP {}→{}", value, hp_before, hp_after),
        GameEvent::PotionDiscarded { value } => format!("🧪 discarded {}", value),
        GameEvent::Weapon { value } => format!("🗡️ equip {}", value),
        GameEvent::Fight {
            monster,
            with_weapon,
            damage_taken,
        } => {
            let w = with_weapon
                .map(|v| format!(" with {}", v))
                .unwrap_or_else(|| " barehand".into());
            let skulls = if *damage_taken > 0 { " 💥" } else { " ✅" };
            format!("👾 {}{} → dmg {}{}", monster, w, damage_taken, skulls)
        }
        GameEvent::Avoid => "🌀 avoid room".into(),
    }
}

fn build_history_indented_lines(history: &Vec<GameEvent>) -> Vec<Line<'static>> {
    if history.is_empty() {
        return vec![Line::from("No battles happened.")];
    }
    let mut groups: Vec<(u32, Vec<String>)> = Vec::new();
    let mut current_room: Option<u32> = None;
    for ev in history {
        match ev {
            GameEvent::RoomStart { number } => {
                current_room = Some(*number);
                groups.push((*number, Vec::new()));
            }
            _ => {
                let s = game_event_line(ev);
                if let Some((_, vec)) = groups.last_mut() {
                    vec.push(s);
                } else {
                    let n = current_room.unwrap_or(1);
                    groups.push((n, vec![s]));
                }
            }
        }
    }
    let mut lines: Vec<Line> = Vec::new();
    for (room, evs) in groups {
        lines.push(Line::from(Span::styled(format!("Room {}", room), Style::default().add_modifier(Modifier::BOLD))));
        if evs.is_empty() {
            lines.push(Line::from("  (no actions)"));
        } else {
            for s in evs {
                lines.push(Line::from(format!("  {}", s)));
            }
        }
        lines.push(Line::from(""));
    }
    lines
}
