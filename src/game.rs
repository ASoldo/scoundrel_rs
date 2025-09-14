use crate::cards::{Card, Suit, Rank};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::deck::Deck;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Menu,
    NameEntry,
    Leaderboard,
    Running,
    GameOver,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub hp: i32,
    pub max_hp: i32,
    pub weapon: Option<WeaponState>,
}

impl Player {
    pub fn new() -> Self {
        Self { hp: 20, max_hp: 20, weapon: None }
    }
}

#[derive(Debug, Clone)]
pub struct WeaponState {
    pub value: u8,                  // weapon power (2..=10)
    pub last_monster: Option<u8>,   // last monster value fought with this weapon
    pub stack: Vec<Card>,           // monsters stacked on this weapon (for UI)
}

impl WeaponState {
    pub fn new(value: u8) -> Self {
        Self { value, last_monster: None, stack: Vec::new() }
    }
    pub fn can_use_on(&self, monster_value: u8) -> bool {
        match self.last_monster {
            None => true,
            Some(prev) => monster_value <= prev,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Game {
    pub phase: GamePhase,
    pub player: Player,
    pub deck: Deck,
    pub room: [Option<Card>; 4],
    pub selected: usize,
    pub choices_this_turn: u8,
    pub avoided_last_turn: bool,
    pub potion_used_this_turn: bool,
    pub discard: Vec<Card>,
    pub log: Vec<String>,
    pub show_help: bool,
    pub score: Option<i32>,
    pub last_card_potion_value: Option<u8>,
    pub menu_selected: usize,
    pub name_input: String,
    pub player_name: String,
    pub history: Vec<GameEvent>,
    pub leaderboard: Vec<ScoreEntry>,
    pub new_rank_pos: Option<usize>,
    pub room_number: u32,
    pub game_over_scroll: u16,
}

impl Game {
    pub fn new() -> Self {
        let mut deck = Deck::scoundrel_deck();
        deck.shuffle();
        let leaderboard = Self::load_leaderboard();
        Self {
            phase: GamePhase::Menu,
            player: Player::new(),
            deck,
            room: [None, None, None, None],
            selected: 0,
            choices_this_turn: 0,
            avoided_last_turn: false,
            potion_used_this_turn: false,
            discard: Vec::new(),
            log: vec![
                "Welcome to Scoundrel (terminal)!".into(),
                "Press 'n' for quick new run or use menu.".into(),
            ],
            show_help: false,
            score: None,
            last_card_potion_value: None,
            menu_selected: 0,
            name_input: String::new(),
            player_name: String::from("Scoundrel"),
            history: Vec::new(),
            leaderboard,
            new_rank_pos: None,
            room_number: 0,
            game_over_scroll: 0,
        }
    }

    pub fn new_run(&mut self) {
        self.player = Player::new();
        self.deck = Deck::scoundrel_deck();
        self.deck.shuffle();
        self.room = [None, None, None, None];
        self.selected = 0;
        self.choices_this_turn = 0;
        self.avoided_last_turn = false;
        self.potion_used_this_turn = false;
        self.discard.clear();
        self.score = None;
        self.last_card_potion_value = None;
        self.history.clear();
        self.phase = GamePhase::Running;
        self.log.clear();
        self.log.push("A fresh dungeon awaits...".into());
        self.refill_room();
        self.room_number = 1;
        self.history.push(GameEvent::RoomStart { number: self.room_number });
    }

    pub fn toggle_help(&mut self) { self.show_help = !self.show_help; }

    pub fn move_selection(&mut self, dx: i32, _dy: i32) {
        if self.phase != GamePhase::Running { return; }
        // 1x4 layout: move horizontally only
        let ns = (self.selected as i32 + dx).clamp(0, 3);
        self.selected = ns as usize;
    }

    pub fn take_selected_default(&mut self) { self.take_selected(UseMode::Default); }
    pub fn take_selected_barehand(&mut self) { self.take_selected(UseMode::Barehand); }
    pub fn take_selected_weapon(&mut self) { self.take_selected(UseMode::Weapon); }

    pub fn avoid_room(&mut self) {
        if self.phase != GamePhase::Running { return; }
        if self.avoided_last_turn {
            self.log.push("You cannot avoid two rooms in a row.".into());
            return;
        }
        if self.visible_count() < 4 {
            self.log.push("You may only avoid when 4 cards are visible.".into());
            return;
        }
        // Scoop all four to bottom in visible order (top-left, top-right, bottom-left, bottom-right)
        for i in 0..4 {
            if let Some(card) = self.room[i].take() {
                self.deck.push_bottom(card);
            }
        }
        self.avoided_last_turn = true;
        self.potion_used_this_turn = false;
        self.choices_this_turn = 0;
        self.log.push("You avoid the room, slipping past the dangers.".into());
        self.history.push(GameEvent::Avoid);
        self.refill_room();
        if matches!(self.phase, GamePhase::Running) {
            self.room_number += 1;
            self.history.push(GameEvent::RoomStart { number: self.room_number });
        }
    }

    fn visible_count(&self) -> usize { self.room.iter().filter(|c| c.is_some()).count() }

    fn refill_room(&mut self) {
        // Leave any remaining room card(s) in place; draw until there are 4 or deck empty
        for slot in 0..4 {
            if self.room[slot].is_none() && let Some(c) = self.deck.draw() {
                self.room[slot] = Some(c);
            }
        }
        // Reset selection to first non-empty
        if self.room[self.selected].is_none()
            && let Some((idx, _)) = self.room.iter().enumerate().find(|(_, c)| c.is_some())
        {
            self.selected = idx;
        }
        if self.deck.is_empty() && self.visible_count() == 0 {
            self.finish_victory();
        }
    }

    fn end_turn(&mut self) {
        // Keep one remaining card (if any) on table; refill to 4 for next turn
        self.avoided_last_turn = false;
        self.potion_used_this_turn = false;
        self.choices_this_turn = 0;
        self.refill_room();
        if matches!(self.phase, GamePhase::Running) {
            self.room_number += 1;
            self.history.push(GameEvent::RoomStart { number: self.room_number });
        }
    }

    fn finish_victory(&mut self) {
        self.phase = GamePhase::GameOver;
        let mut score = self.player.hp;
        if self.player.hp == self.player.max_hp
            && let Some(v) = self.last_card_potion_value
        { score += v as i32; }
        self.score = Some(score);
        self.log.push(format!("You clear the dungeon. Final score: {}.", score));
        self.push_score_and_rank(true);
        self.game_over_scroll = 0;
    }

    fn finish_death(&mut self) {
        self.phase = GamePhase::GameOver;
        // Sum remaining monsters in deck and room
        let mut penalty = 0i32;
        for c in &self.deck.cards { if c.is_monster() { penalty += c.monster_value() as i32; } }
        for card in self.room.iter().flatten() {
            if card.is_monster() { penalty += card.monster_value() as i32; }
        }
        let score = self.player.hp - penalty; // hp is <= 0
        self.score = Some(score);
        self.log.push(format!("You fall... Final score: {}.", score));
        self.push_score_and_rank(false);
        self.game_over_scroll = 0;
    }

    fn take_selected(&mut self, mode: UseMode) {
        if self.phase != GamePhase::Running { return; }
        if self.choices_this_turn >= 3 && self.visible_count() >= 2 {
            self.log.push("You've already taken 3 cards. Ending turn.".into());
            self.end_turn();
            return;
        }
        let idx = self.selected;
        let Some(card) = self.room[idx].take() else { return; };
        // Determine how many picks allowed this turn based on initial room size; default = 3, but when fewer cards visible, allow all but one
        self.resolve_card(card, mode);

        if self.player.hp <= 0 { self.finish_death(); return; }

        // Count pick and decide if turn ends: leave exactly one card if possible
        self.choices_this_turn += 1;
        if self.visible_count() <= 1 {
            // if only one remains (or none), end turn and refill
            self.end_turn();
        } else if self.choices_this_turn >= 3 {
            self.end_turn();
        }
    }

    fn resolve_card(&mut self, card: Card, mode: UseMode) {
        match card.suit {
            Suit::Hearts => {
                if !self.potion_used_this_turn {
                    let val = card.monster_value(); // 2..10
                    let heal = val as i32;
                    let before = self.player.hp;
                    self.player.hp = (self.player.hp + heal).min(self.player.max_hp);
                    self.log.push(format!("You drink a potion ({}). HP {}→{}.", heal, before, self.player.hp));
                    self.last_card_potion_value = Some(val);
                    self.history.push(GameEvent::Potion { value: val, hp_before: before, hp_after: self.player.hp });
                } else {
                    self.log.push("You already used a potion this turn; this one is discarded.".into());
                    self.history.push(GameEvent::PotionDiscarded { value: card.monster_value() });
                }
                self.potion_used_this_turn = true;
                self.discard.push(card);
            }
            Suit::Diamonds => {
                // Bind weapon: equip and discard previous weapon + its monsters
                if let Some(w) = self.player.weapon.take() {
                    // discard previous weapon card and monsters on it
                    self.discard.push(Card::new(Suit::Diamonds, Rank::new(w.value)));
                    for m in w.stack { self.discard.push(m); }
                }
                let val = card.monster_value(); // 2..10
                self.player.weapon = Some(WeaponState::new(val));
                self.log.push(format!("You equip a weapon ({}).", val));
                self.history.push(GameEvent::Weapon { value: val });
                // The weapon card stays equipped (not in discard)
            }
            Suit::Clubs | Suit::Spades => {
                let mval = card.monster_value();
                let mut use_weapon = false;
                if let Some(w) = &self.player.weapon && w.can_use_on(mval) {
                    use_weapon = match mode {
                        UseMode::Default => true,
                        UseMode::Weapon => true,
                        UseMode::Barehand => false,
                    };
                }
                if use_weapon {
                    let w = self.player.weapon.as_mut().unwrap();
                    let dmg = (mval as i32 - w.value as i32).max(0);
                    if dmg > 0 {
                        let before = self.player.hp;
                        self.player.hp -= dmg;
                        self.log.push(format!("You strike with {}. Monster {} hits back ({} dmg). HP {}→{}.", w.value, mval, dmg, before, self.player.hp));
                        self.history.push(GameEvent::Fight { monster: mval, with_weapon: Some(w.value), damage_taken: dmg as u8 });
                    } else {
                        self.log.push(format!("You strike with {}. Monster {} falls.", w.value, mval));
                        self.history.push(GameEvent::Fight { monster: mval, with_weapon: Some(w.value), damage_taken: 0 });
                    }
                    w.stack.push(card);
                    w.last_monster = Some(mval);
                } else {
                    let before = self.player.hp;
                    self.player.hp -= mval as i32;
                    self.log.push(format!("You fight barehanded. Monster {} hits you ({} dmg). HP {}→{}.", mval, mval, before, self.player.hp));
                    self.discard.push(card);
                     self.history.push(GameEvent::Fight { monster: mval, with_weapon: None, damage_taken: mval });
                }
            }
        }
        // Track last-card scoring: only keep if a potion was the most recent resolution
        match card.suit { Suit::Hearts => { /* keep set above */ } _ => { self.last_card_potion_value = None; } }
    }

    pub fn tick(&mut self) { /* future animations */ }

    pub fn select_menu_up(&mut self) { if self.menu_selected > 0 { self.menu_selected -= 1; } }
    pub fn select_menu_down(&mut self) { if self.menu_selected < 2 { self.menu_selected += 1; } }
    pub fn menu_activate(&mut self) {
        match self.menu_selected {
            0 => { self.phase = GamePhase::NameEntry; self.name_input.clear(); }
            1 => { self.phase = GamePhase::Leaderboard; }
            2 => { /* handled in app loop by 'q' */ }
            _ => {}
        }
    }

    pub fn name_input_char(&mut self, ch: char) {
        if (ch.is_ascii_graphic() || ch == ' ') && self.name_input.len() < 20 {
            self.name_input.push(ch);
        }
    }
    pub fn name_input_backspace(&mut self) { self.name_input.pop(); }
    pub fn name_input_submit(&mut self) {
        if !self.name_input.trim().is_empty() { self.player_name = self.name_input.trim().to_string(); }
        self.new_run();
    }

    fn scores_path() -> &'static str { "scoundrel_scores.json" }
    fn load_leaderboard() -> Vec<ScoreEntry> {
        let p = Path::new(Self::scores_path());
        if let Ok(text) = fs::read_to_string(p)
            && let Ok(v) = serde_json::from_str::<Vec<ScoreEntry>>(&text)
        { return v; }
        Vec::new()
    }
    fn save_leaderboard(&self) {
        let _ = fs::write(Self::scores_path(), serde_json::to_string_pretty(&self.leaderboard).unwrap_or_else(|_| "[]".into()));
    }
    fn push_score_and_rank(&mut self, won: bool) {
        let score = self.score.unwrap_or(0);
        let entry = ScoreEntry { name: self.player_name.clone(), score, won, ts: now_ts() };
        self.leaderboard.push(entry);
        // Sort descending by score
        self.leaderboard.sort_by(|a,b| b.score.cmp(&a.score));
        // Find position of most recent by name & ts & score
        let last = self.leaderboard.iter().enumerate().find(|(_, e)| e.name == self.player_name && e.score == score && e.won == won).map(|(i,_)| i);
        self.new_rank_pos = last;
        self.save_leaderboard();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UseMode { Default, Barehand, Weapon }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreEntry {
    pub name: String,
    pub score: i32,
    pub won: bool,
    pub ts: u64,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    RoomStart { number: u32 },
    Potion { value: u8, hp_before: i32, hp_after: i32 },
    PotionDiscarded { value: u8 },
    Weapon { value: u8 },
    Fight { monster: u8, with_weapon: Option<u8>, damage_taken: u8 },
    Avoid,
}

fn now_ts() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs()
}
