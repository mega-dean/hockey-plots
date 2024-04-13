use std::{collections::HashMap, fs};

use eframe::egui;
use egui::{Color32, RichText};
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};

fn main() -> Result<(), eframe::Error> {
  env_logger::init();
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 960.0]),
    ..Default::default()
  };
  eframe::run_native(
    "hockey plots",
    options,
    Box::new(|_cc| Box::<App>::default()),
  )
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

#[derive(Copy, Clone)]
struct ShowDivision {
  central: bool,
  pacific: bool,
  atlantic: bool,
  metro: bool,
}

struct App {
  games: HashMap<i32, Vec<full_json::Game>>,
  show: ShowDivision,
}

enum Division {
  Central,
  Pacific,
  Atlantic,
  Metropolitan,
}

struct Team {
  id: i32,
  color: Color32,
  abbrev: String,
  division: Division,
}

impl Team {
  fn new(id: i32, abbrev: &str, (r, g, b): (u8, u8, u8), division: Division) -> Self {
    Self {
      id,
      color: Color32::from_rgb(r, g, b),
      abbrev: String::from(abbrev),
      division,
    }
  }
}

// TODO Some of these colors are too dark.
fn all_teams() -> Vec<Team> {
  vec![
    // Western
    Team::new(24, "ANA", (252, 76, 2), Division::Pacific),
    Team::new(20, "CGY", (210, 0, 28), Division::Pacific),
    Team::new(22, "EDM", (4, 30, 66), Division::Pacific),
    Team::new(26, "LAK", (162, 170, 173), Division::Pacific),
    Team::new(55, "SEA", (104, 162, 185), Division::Pacific),
    Team::new(28, "SJS", (0, 109, 117), Division::Pacific),
    Team::new(23, "VAN", (0, 32, 91), Division::Pacific),
    Team::new(54, "VGK", (185, 151, 91), Division::Pacific),
    Team::new(53, "ARI", (140, 38, 51), Division::Central),
    Team::new(16, "CHI", (207, 10, 44), Division::Central),
    Team::new(21, "COL", (111, 38, 61), Division::Central),
    Team::new(25, "DAL", (0, 104, 71), Division::Central),
    Team::new(30, "MIN", (2, 73, 48), Division::Central),
    Team::new(18, "NSH", (255, 184, 28), Division::Central),
    Team::new(19, "STL", (0, 47, 135), Division::Central),
    Team::new(52, "WPG", (4, 30, 66), Division::Central),
    // Eastern
    Team::new(6, "BOS", (252, 181, 20), Division::Atlantic),
    Team::new(7, "BUF", (0, 48, 135), Division::Atlantic),
    Team::new(17, "DET", (206, 17, 38), Division::Atlantic),
    Team::new(13, "FLA", (185, 151, 91), Division::Atlantic),
    Team::new(8, "MTL", (175, 30, 45), Division::Atlantic),
    Team::new(9, "OTT", (183, 146, 87), Division::Atlantic),
    Team::new(14, "TBL", (0, 40, 104), Division::Atlantic),
    Team::new(10, "TOR", (0, 32, 91), Division::Atlantic),
    Team::new(12, "CAR", (206, 17, 38), Division::Metropolitan),
    Team::new(29, "CBJ", (0, 38, 84), Division::Metropolitan),
    Team::new(1, "NJD", (206, 17, 38), Division::Metropolitan),
    Team::new(2, "NYI", (0, 83, 155), Division::Metropolitan),
    Team::new(3, "NYR", (0, 56, 168), Division::Metropolitan),
    Team::new(4, "PHI", (247, 73, 2), Division::Metropolitan),
    Team::new(5, "PIT", (252, 181, 20), Division::Metropolitan),
    Team::new(15, "WSH", (200, 16, 46), Division::Metropolitan),
  ]
}

fn find_team(id: i32) -> Option<Team> {
  all_teams().into_iter().find(|team| team.id == id)
}

impl Default for App {
  fn default() -> Self {
    let mut games = HashMap::new();
    for team in all_teams() {
      let json_contents = fs::read_to_string(format!("../data/{}.json", team.abbrev)).unwrap();
      let parsed: full_json::TeamSchedule = serde_json::from_str(&json_contents).unwrap();
      let mut team_games = vec![];
      for game in parsed.games.iter() {
        if game.game_type == 2 {
          team_games.push(*game)
        }
      }
      games.insert(team.id, team_games);
    }
    Self {
      games,
      show: ShowDivision {
        central: true,
        pacific: true,
        atlantic: true,
        metro: true,
      },
    }
  }
}

fn _txt(s: &str) -> egui::widget_text::RichText {
  RichText::new(s).size(24.0)
}

impl eframe::App for App {
  fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
    egui::SidePanel::left("options").show(ctx, |ui| {
      ui.checkbox(&mut self.show.metro, "Metro");
      ui.checkbox(&mut self.show.pacific, "Pacific");
      ui.checkbox(&mut self.show.atlantic, "Atlantic");
      ui.checkbox(&mut self.show.central, "Central");
    });
    egui::CentralPanel::default().show(ctx, |ui| {
      egui_plot::Plot::new("plot")
        .legend(Legend::default().text_style(egui::TextStyle::Heading))
        .show(ui, |plot_ui| {
          fn show_team(show: ShowDivision, team: &Team) -> bool {
            match team.division {
              Division::Metropolitan => show.metro,
              Division::Central => show.central,
              Division::Pacific => show.pacific,
              Division::Atlantic => show.atlantic,
            }
          }
          for (team_id, games_) in &self.games {
            let team: Team = find_team(*team_id).unwrap();
            if show_team(self.show, &team) {
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
                  points_so_far += points - 1.0;
                  games.push(PlotPoint {
                    x: (1 + idx) as f64,
                    y: points_so_far as f64,
                  })
                }
              }

              let game_points = PlotPoints::Owned(games.clone());
              plot_ui.line(Line::new(game_points).name(team.abbrev).color(team.color));
            }
          }
        });

      if ui.input(|i| i.modifiers.shift_only() && i.key_pressed(egui::Key::Q)) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
      }
    });
  }
}
