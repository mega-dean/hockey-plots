use std::collections::{HashMap, HashSet};

use eframe::egui;
use egui::{Color32, RichText};
use egui_plot::{Legend, Line, PlotPoint, PlotPoints};

use reqwest as request;

fn main() -> Result<(), eframe::Error> {
  env_logger::init();

  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 960.0]),
    ..Default::default()
  };

  let rt = tokio::runtime::Runtime::new().expect("tokio Runtime::new() failure");

  let _enter = rt.enter();

  // CLEANUP not sure if this is necessary
  std::thread::spawn(move || {
    rt.block_on(async {
      loop {
        tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
      }
    })
  });

  eframe::run_native(
    "hockey plots",
    options,
    Box::new(|_cc| Box::<App>::default()),
  )
}

mod db {
  #[derive(Clone, Debug, serde::Deserialize)]
  pub struct Division {
    pub id: i32,
    pub name: String,
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  pub struct LastPeriodType {
    pub id: i32,
    pub name: String,
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  pub struct Team {
    pub id: i32,
    pub api_id: i32,
    pub r: i32,
    pub g: i32,
    pub b: i32,
    pub abbrev: String,
    pub division_id: i32,
  }

  impl Team {
    pub fn db_to_crate(self, db_divisions: &[Division]) -> crate::Team {
      fn find_division(id: i32, db_divisions: &[Division]) -> crate::Division {
        let db_division = db_divisions
          .iter()
          .find(|db_division| db_division.id == id)
          .unwrap_or_else(|| panic!("could not find division with id: {id}"));
        match db_division.name.as_str() {
          "Metropolitan" => crate::Division::Metropolitan,
          "Pacific" => crate::Division::Pacific,
          "Atlantic" => crate::Division::Atlantic,
          "Central" => crate::Division::Central,
          _ => panic!("Found division with invalid name: {}", db_division.name),
        }
      }

      crate::Team {
        db_id: self.id,
        api_id: self.api_id,
        color: egui::Color32::from_rgb(self.r as u8, self.g as u8, self.b as u8),
        abbrev: self.abbrev,
        division: find_division(self.division_id, db_divisions),
      }
    }
  }

  #[derive(Copy, Clone, Debug, serde::Deserialize)]
  pub struct Score {
    pub id: i32,
    pub home: i32,
    pub away: i32,
    pub last_period_type_id: i32,
  }

  #[derive(Clone, Debug, serde::Deserialize)]
  pub struct Game {
    pub api_id: i32,
    pub home_team_id: i32,
    pub away_team_id: i32,
    pub score_id: Option<i32>,
  }

  impl Game {
    pub fn db_to_crate(
      self: Game,
      game_outcome: Option<(Score, LastPeriodType)>,
      this_team: &crate::Team,
    ) -> crate::Game {
      let points = if let Some((score, last_period_type)) = game_outcome {
        let (this_team_score, opponent_score) = if self.home_team_id == this_team.db_id {
          (score.home, score.away)
        } else {
          (score.away, score.home)
        };

        if this_team_score > opponent_score {
          Some(2.0)
        } else {
          Some(match last_period_type.name.as_str() {
            "Regulation" => 0.0,
            "Overtime" | "Shootout" => 1.0,
            other => panic!("found unknown last_period_type in db: '{other}'"),
          })
        }
      } else {
        None
      };

      crate::Game { points }
    }
  }
}

mod json {
  use serde::Deserialize;

  #[derive(Debug, Deserialize, Copy, Clone)]
  pub struct GameTeam {
    pub id: i32,
    pub score: Option<i32>,
  }

  #[derive(Debug, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
  #[allow(clippy::upper_case_acronyms)]
  pub enum PeriodType {
    SO,
    REG,
    OT,
  }

  #[derive(Debug, Deserialize, Copy, Clone, PartialEq)]
  #[allow(clippy::upper_case_acronyms)]
  pub enum GameState {
    FINAL, // preseason
    OFF,   // regular season
    FUT,   // regular season, future
  }

  #[derive(Debug, Deserialize, Copy, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct GameOutcome {
    pub last_period_type: PeriodType,
  }

  #[derive(Debug, Deserialize, Copy, Clone)]
  #[serde(rename_all = "camelCase")]
  pub struct Game {
    pub id: i32,
    pub home_team: GameTeam,
    pub away_team: GameTeam,
    pub game_type: u8,
    pub game_outcome: Option<GameOutcome>,
    pub game_state: GameState,
  }

  impl Game {
    pub fn is_preseason(self) -> bool {
      self.game_type == 1
    }

    // TODO Consolidate with db::db_to_crate.
    pub fn api_to_crate(self, this_team: &crate::Team) -> crate::Game {
      let points = if let Some(outcome) = self.game_outcome {
        let (this_team_score, opponent_score) = if self.home_team.id == this_team.api_id {
          (self.home_team.score, self.away_team.score)
        } else {
          (self.away_team.score, self.home_team.score)
        };

        if this_team_score.is_some() {
          if this_team_score > opponent_score {
            Some(2.0)
          } else {
            match outcome.last_period_type {
              PeriodType::REG => Some(0.0),
              PeriodType::OT | PeriodType::SO => Some(1.0),
            }
          }
        } else {
          None
        }
      } else {
        None
      };

      crate::Game { points }
    }
  }

  #[derive(Clone, Debug, Deserialize)]
  pub struct TeamSchedule {
    pub games: Vec<Game>,
  }

  #[derive(Clone, Debug, Deserialize)]
  pub struct ApiResponse {
    // Key is team.api_id.
    pub schedules: std::collections::HashMap<i32, TeamSchedule>,
  }

  pub mod api {
    use crate::*;
    pub async fn load_games(teams: Vec<Team>) -> Result<json::ApiResponse, request::Error> {
      let mut api_response = json::ApiResponse {
        schedules: std::collections::HashMap::new(),
      };
      for team in teams {
        let json_contents = request::get(format!(
          "https://api-web.nhle.com/v1/club-schedule-season/{}/20232024",
          team.abbrev
        ))
        .await?
        .text()
        .await?;
        let schedule: json::TeamSchedule = serde_json::from_str(&json_contents).expect("no");
        api_response.schedules.insert(team.api_id, schedule);
      }
      Ok(api_response)
    }
  }
}

#[derive(Debug)]
struct DB {
  conn: rusqlite::Connection,
}

use std::path::Path;

impl DB {
  fn all<T: for<'a> serde::Deserialize<'a>>(&self, table_name: &str) -> Vec<T> {
    // TODO Not sure why this can't use rusqlite's `?1` params - getting SqlInputError with `code: Unknown`.
    let mut statement = self
      .conn
      .prepare(&format!("SELECT * FROM {};", table_name))
      .unwrap_or_else(|_| panic!("error from DB::all for table {table_name}"));
    let res = serde_rusqlite::from_rows::<T>(statement.query([]).unwrap());
    res.flatten().collect()
  }

  fn all_scores(&self) -> Vec<db::Score> {
    self.all::<db::Score>("scores")
  }

  fn all_divisions(&self) -> Vec<db::Division> {
    self.all::<db::Division>("divisions")
  }

  fn all_last_period_types(&self) -> Vec<db::LastPeriodType> {
    self.all::<db::LastPeriodType>("last_period_types")
  }

  fn all_teams(&self) -> Vec<db::Team> {
    self.all::<db::Team>("teams")
  }

  fn all_games(&self, teams: &[Team]) -> GamesByTeam {
    let mut games: GamesByTeam = HashMap::new();
    let scores = self.all_scores();
    let last_period_types = self.all_last_period_types();

    for team in teams {
      let mut team_games = vec![];
      let mut statement = self
        .conn
        .prepare("SELECT * FROM games WHERE games.home_team_id = ?1 OR games.away_team_id = ?2;")
        .unwrap();
      let res = serde_rusqlite::from_rows::<db::Game>(
        statement
          .query([team.db_id, team.db_id])
          .unwrap_or_else(|e| panic!("error while SELECTing games: {:?}", e)),
      );
      fn find_score(
        score_id: i32,
        scores: &[db::Score],
        last_period_types: &[db::LastPeriodType],
      ) -> Option<(db::Score, db::LastPeriodType)> {
        if let Some(score) = scores.iter().find(|score| score.id == score_id) {
          let lpt: db::LastPeriodType = last_period_types
            .iter()
            .find(|period_type| period_type.id == score.last_period_type_id)
            .unwrap()
            .clone();
          Some((*score, lpt))
        } else {
          None
        }
      }
      for db_game in res.flatten() {
        let game_outcome = if let Some(score_id) = db_game.score_id {
          find_score(score_id, &scores, &last_period_types)
        } else {
          None
        };
        let team_game: Game = db_game.db_to_crate(game_outcome, team);
        team_games.push(team_game)
      }

      games.insert(team.api_id, team_games);
    }
    games
  }

  fn execute_file(&self, path: &str) {
    use std::io::Read;

    let mut f = std::fs::File::open(path)
      .unwrap_or_else(|_| panic!("could not find sql file at path: {path}"));
    let mut buffer = String::new();
    f.read_to_string(&mut buffer).unwrap().to_string();

    match self.conn.execute_batch(&buffer) {
      Ok(_) => println!("initialized db"),
      Err(_) => println!("db initialization skipped"),
    };
  }

  pub fn initialize(db_path: &Path) -> Self {
    let conn = rusqlite::Connection::open(db_path)
      .unwrap_or_else(|_| panic!("could not find db file at path: {:?}", db_path));
    let db = Self { conn };
    db.execute_file("../data/init.sql");
    db
  }

  fn get_teams(&self) -> Vec<Team> {
    let db_divisions = self.all_divisions();
    let db_teams: Vec<db::Team> = self.all_teams();
    db_teams
      .iter()
      .map(|db_team| db_team.clone().db_to_crate(&db_divisions))
      .collect()
  }
}

#[derive(Copy, Clone, Debug)]
struct ShowDivision {
  central: bool,
  pacific: bool,
  atlantic: bool,
  metro: bool,
}

// Key is the Team's api_id.
type GamesByTeam = HashMap<i32, Vec<Game>>;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
enum Division {
  Central,
  Pacific,
  Atlantic,
  Metropolitan,
}

#[derive(Clone, Debug)]
struct Team {
  db_id: i32,
  api_id: i32,
  color: Color32,
  abbrev: String,
  division: Division,
}

#[derive(Copy, Clone, Debug)]
struct Game {
  points: Option<f32>,
}

#[derive(Clone, Debug)]
struct AppData {
  games: GamesByTeam,
  teams: Vec<Team>,
}

#[derive(Debug)]
struct App {
  db: DB,
  tx: std::sync::mpsc::Sender<json::ApiResponse>,
  rx: std::sync::mpsc::Receiver<json::ApiResponse>,
  data: AppData,
  show: ShowDivision,
}

impl Default for App {
  fn default() -> Self {
    let (tx, rx) = std::sync::mpsc::channel();

    let path = std::path::Path::new("../data/hockeyplots.db");
    let db = DB::initialize(path);

    let teams = db.get_teams();
    let games = db.all_games(&teams);

    Self {
      db,
      tx,
      rx,
      data: AppData { games, teams },
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
    fn make_app_data(api_response: &json::ApiResponse, teams: &[Team]) -> AppData {
      let mut games: GamesByTeam = HashMap::new();
      for (team_api_id, schedule) in &api_response.schedules {
        let team = teams
          .iter()
          .find(|team| team.api_id == *team_api_id)
          .unwrap();
        let mut team_games = vec![];

        for game in &schedule.games {
          if !game.is_preseason() {
            team_games.push(game.api_to_crate(team));
          }
        }
        games.insert(team.api_id, team_games);
      }

      AppData {
        games,
        teams: teams.to_vec(),
      }
    }

    if let Ok(api_response) = self.rx.try_recv() {
      let new_data = make_app_data(&api_response, &self.data.teams);
      self.data = new_data.clone();
      let mut game_statement = self
        .db
        .conn
        .prepare("INSERT INTO games (api_id, home_team_id, away_team_id, score_id) VALUES (:api_id, :home_team_id, :away_team_id, :score_id);")
        .unwrap();
      let mut score_statement = self
        .db
        .conn
        .prepare("INSERT INTO scores (home, away, last_period_type_id) VALUES (:home, :away, :last_period_type_id);")
        .unwrap();
      fn get_period_type_id(db: &DB, name: &str) -> i32 {
        db.conn
          .query_row(
            "SELECT id FROM last_period_types WHERE name = ?1",
            [name],
            |r| r.get(0),
          )
          .unwrap()
      }
      let mut period_type_ids: HashMap<json::PeriodType, i32> = HashMap::new();
      period_type_ids.insert(
        json::PeriodType::REG,
        get_period_type_id(&self.db, "Regulation"),
      );
      period_type_ids.insert(
        json::PeriodType::SO,
        get_period_type_id(&self.db, "Shootout"),
      );
      period_type_ids.insert(
        json::PeriodType::OT,
        get_period_type_id(&self.db, "Overtime"),
      );
      let mut game_ids: HashSet<i32> = HashSet::new();
      for (_team_api_id, schedule) in api_response.schedules {
        for json_game in schedule.games {
          let mut score_id = None;
          if let Some(outcome) = json_game.game_outcome {
            if !json_game.is_preseason() && game_ids.get(&json_game.id).is_none() {
              game_ids.insert(json_game.id);
              fn get_score(team: json::GameTeam, team_str: &str) -> i32 {
                team.score.unwrap_or_else(|| {
                  panic!("json_game with an outcome should have a score for the {team_str} team")
                })
              }
              score_id = Some(
                score_statement
                  .insert(rusqlite::named_params! {
                    ":home": get_score(json_game.home_team, "home"),
                    ":away": get_score(json_game.away_team, "away"),
                    ":last_period_type_id": *period_type_ids.get(&outcome.last_period_type).unwrap(),
                  })
                  .unwrap(),
              );
            }
          } else {
            println!("got no gameOutcome for {}", json_game.id);
          }
          fn get_db_id(game_team: json::GameTeam, teams: &[Team]) -> i32 {
            teams
              .iter()
              .find(|team| team.api_id == game_team.id)
              .unwrap()
              .db_id
          }
          if !json_game.is_preseason() {
            match game_statement.execute(rusqlite::named_params! {
              ":api_id": json_game.id,
              ":home_team_id": get_db_id(json_game.home_team, &self.data.teams),
              ":away_team_id": get_db_id(json_game.away_team, &self.data.teams),
              ":score_id": score_id,
            }) {
              Ok(_) => (),
              Err(e) => {
                println!(
                  "error on game_statement.execute: {:?}\n with params:\n{}, {}, {}, {:?}",
                  e, json_game.id, json_game.home_team.id, json_game.away_team.id, score_id,
                )
              }
            }
          }
        }
      }
    }

    egui::SidePanel::left("options").show(ctx, |ui| {
      ui.collapsing("Divisions", |cui| {
        cui.checkbox(&mut self.show.metro, "Metro");
        cui.checkbox(&mut self.show.pacific, "Pacific");
        cui.checkbox(&mut self.show.atlantic, "Atlantic");
        cui.checkbox(&mut self.show.central, "Central");
      });

      if ui.button("update").clicked() {
        let tx: std::sync::mpsc::Sender<json::ApiResponse> = self.tx.clone();
        let ctx_ = ctx.clone();

        let teams = self.db.get_teams();
        tokio::spawn(async move {
          match json::api::load_games(teams).await {
            Ok(api_response) => {
              let _ = tx.send(api_response);
              ctx_.request_repaint();
            },
            Err(e) => println!("error from NHL api: {:?}", e),
          }
        });
      }
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
          for (team_id, games_) in &self.data.games {
            let team: &Team = self
              .data
              .teams
              .iter()
              .find(|team| team.api_id == *team_id)
              .unwrap();
            if show_team(self.show, team) {
              let mut games: Vec<PlotPoint> = vec![];
              let mut points_so_far = 0.0;

              games.push(PlotPoint { x: 0.0, y: 0.0 });

              for (idx, game) in games_.iter().enumerate() {
                if let Some(points_) = game.points {
                  points_so_far += points_ - 1.0;
                  games.push(PlotPoint {
                    x: (1 + idx) as f64,
                    y: points_so_far as f64,
                  })
                }
              }

              let game_points = PlotPoints::Owned(games.clone());
              plot_ui.line(
                Line::new(game_points)
                  .name(team.abbrev.clone())
                  .color(team.color),
              );
            }
          }
        });

      if ui.input(|i| i.modifiers.shift_only() && i.key_pressed(egui::Key::Q)) {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close)
      }
    });
  }
}
