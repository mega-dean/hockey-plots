use std::{collections::HashMap, fs};

use eframe::egui;
use egui::RichText;
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};

fn main() -> Result<(), eframe::Error> {
  env_logger::init();
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 960.0]),
    ..Default::default()
  };
  eframe::run_native("hockey plots", options, Box::new(|_cc| Box::<App>::default()))
}

mod full_json {
  use serde::Deserialize;

  #[derive(Deserialize, Copy, Clone)]
  pub struct GameTeam {
    pub id: i32,
    pub score: Option<i8>,
  }

  #[derive(Deserialize, Copy, Clone)]
  #[allow(clippy::upper_case_acronyms)]
  pub enum PeriodType {
    SO,
    REG,
    OT,
  }

  #[derive(Deserialize, Copy, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct GameOutcome {
    pub last_period_type: PeriodType,
  }

  #[derive(Deserialize, Copy, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct Game {
    pub id: i32,
    pub home_team: GameTeam,
    pub away_team: GameTeam,
    pub game_type: u8,
    pub game_outcome: Option<GameOutcome>,
  }

  #[derive(Deserialize)]
  pub struct TeamSchedule {
    pub games: Vec<Game>,
  }
}

struct App {
  games: HashMap<i32, Vec<full_json::Game>>,
  average_points_per_game: f32,
}

fn ids_by_team_abbrev() -> HashMap<String, i32> {
  HashMap::from([
    (String::from("EDM"), 22),
    (String::from("WPG"), 52),
    (String::from("VAN"), 23),
    (String::from("CGY"), 20),
    (String::from("SEA"), 55),
    (String::from("NSH"), 18),
    (String::from("PHI"), 4),
    (String::from("MIN"), 30),
    (String::from("NYR"), 3),
    (String::from("DAL"), 25),
    (String::from("SJS"), 28),
    (String::from("NYI"), 2),
    (String::from("TBL"), 14),
    (String::from("FLA"), 13),
    (String::from("CAR"), 12),
    (String::from("WSH"), 15),
    (String::from("ANA"), 24),
    (String::from("VGK"), 54),
    (String::from("NJD"), 1),
    (String::from("CHI"), 16),
    (String::from("LAK"), 26),
    (String::from("OTT"), 9),
    (String::from("DET"), 17),
    (String::from("MTL"), 8),
    (String::from("TOR"), 10),
    (String::from("CBJ"), 29),
    (String::from("STL"), 19),
    (String::from("ARI"), 53),
    (String::from("BOS"), 6),
    (String::from("PIT"), 5),
    (String::from("BUF"), 7),
    (String::from("COL"), 21),
  ])
}

impl Default for App {
  fn default() -> Self {
    let ids = ids_by_team_abbrev();
    let mut games = HashMap::new();
    for (abbrev, id) in ids {
      let json_contents = fs::read_to_string(format!("../data/{abbrev}.json")).unwrap();
      let parsed: full_json::TeamSchedule = serde_json::from_str(&json_contents).unwrap();
      let mut team_games = vec![];
      for game in parsed.games.iter() {
        if game.game_type == 2 {
          team_games.push(*game)
        }
      }
      games.insert(id, team_games);
    }
    Self {
      games,
      average_points_per_game: 1.0,
    }
  }
}

fn _txt(s: &str) -> egui::widget_text::RichText {
  RichText::new(s).size(24.0)
}

impl eframe::App for App {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::SidePanel::left("options").show(ctx, |ui| {
      ui.add(egui::Slider::new(&mut self.average_points_per_game, 0.0..=2.0).text("ppg"))
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      egui_plot::Plot::new("plot")
        .legend(Legend::default().text_style(egui::TextStyle::Heading))
        .show(ui, |plot_ui| {
          fn get_abbrev(team_id: i32) -> Option<String> {
            for (abbrev, id) in ids_by_team_abbrev() {
              if id == team_id {
                return Some(abbrev);
              }
            }

            None
          }

          for (team_id, games_) in &self.games {
            let mut games: Vec<PlotPoint> = vec![];
            let mut points_so_far = 0.0;

            games.push(PlotPoint { x: 0.0, y: 0.0 });

            for (idx, game) in games_.iter().enumerate() {
              let (edm, opponent) = if game.home_team.id == *team_id {
                (game.home_team, game.away_team)
              } else {
                (game.away_team, game.home_team)
              };
              fn points_for_loss(outcome: full_json::GameOutcome) -> f32 {
                match outcome.last_period_type {
                  full_json::PeriodType::SO | full_json::PeriodType::OT => 1.0,
                  full_json::PeriodType::REG => 0.0,
                }
              }
              if let Some(outcome) = game.game_outcome {
                let points = if edm.score > opponent.score {
                  2.0
                } else {
                  points_for_loss(outcome)
                };
                points_so_far += points - self.average_points_per_game;
                games.push(PlotPoint {
                  x: (1 + idx) as f64,
                  y: points_so_far as f64,
                })
              }
            }
            let game_points = PlotPoints::Owned(games.clone());

            let team_abbrev = get_abbrev(*team_id).unwrap();

            plot_ui.line(Line::new(game_points).name(team_abbrev));
          }
        });

      if ui.input(|i| i.modifiers.shift_only() && i.key_pressed(egui::Key::Q)) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
      }
    });
  }
}
