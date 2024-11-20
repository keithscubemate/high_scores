use std::sync::{Arc, Mutex};

use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use sqlite::{self, Connection, Row};

use axum::{extract::{Path, State}, routing::get, Json, Router};

#[tokio::main]
async fn main() -> Result<()> {
    let db = GameDataBase::new("db.lite");

    db.try_create_table()?;
    db.insert(Game { game: "snake".to_string(), player_name: "austin".to_string(), score: 10 })?;
    db.insert(Game { game: "snake".to_string(), player_name: "alec".to_string(), score: 19 })?;
    db.insert(Game { game: "snake".to_string(), player_name: "keith".to_string(), score: 15 })?;
    db.insert(Game { game: "snake".to_string(), player_name: "karen".to_string(), score: 16 })?;
    db.insert(Game { game: "breakout".to_string(), player_name: "austin".to_string(), score: 35 })?;
    db.insert(Game { game: "breakout".to_string(), player_name: "alec".to_string(), score: 30 })?;
    db.insert(Game { game: "breakout".to_string(), player_name: "keith".to_string(), score: 32 })?;
    db.insert(Game { game: "breakout".to_string(), player_name: "karen".to_string(), score: 33 })?;

    let db_share = Arc::new(Mutex::new(db));

    let app = Router::new()
        .route("/games/", get(get_all_game))
        .route("/games/:game_name", get(get_game_by_name))
        .with_state(db_share);

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await?;

    Ok(())
}

async fn get_all_game(
    State(db_share): State<Arc<Mutex<GameDataBase>>>,
) -> Json<serde_json::Value> {
    let db = db_share.lock().unwrap();

    let data = db.get_all_for_game("*").unwrap();

    Json(json!(data))
}

async fn get_game_by_name(
    Path(id): Path<String>,
    State(db_share): State<Arc<Mutex<GameDataBase>>>,
) -> Json<serde_json::Value> {
    let db = db_share.lock().unwrap();

    let data = db.get_all_for_game(&id).unwrap();

    Json(json!(data))
}

#[derive(Serialize)]
struct Game {
    game: String,
    player_name: String,
    score: u64,
}

impl Game {
    fn from_row(row: &Row) -> Self {
        let game = row.read::<&str, _>("game").to_owned();
        let score = row.read::<i64, _>("score") as u64;
        let player_name = row.read::<&str, _>("player_name").to_owned();

        Self {
            game,
            score,
            player_name,
        }
    }
}

struct GameDataBase {
    connection: Connection,
}

impl GameDataBase {
    fn new(path: &str) -> Self {
        let connection = sqlite::open(path).unwrap();

        Self { connection }
    }

    fn try_create_table(&self) -> Result<()> {
        let query =
            "CREATE TABLE if not exists scores (game TEXT, score INTEGER, player_name TEXT);";

        self.connection.execute(query)?;

        Ok(())
    }

    fn insert(&self, game: Game) -> Result<()> {
        let query = format!(
            "INSERT INTO scores VALUES ('{}',{}, '{}');",
            game.game, game.score, game.player_name
        );

        self.connection.execute(query)?;

        Ok(())
    }

    fn get_all_for_game(&self, game: &str) -> Result<Vec<Game>> {
        let query = "SELECT * FROM scores WHERE game = ? order by score desc;";

        Ok(self
            .connection
            .prepare(query)?
            .into_iter()
            .bind((1, game))?
            .filter(|row| row.is_ok())
            .map(|row| row.unwrap())
            .map(|row| Game::from_row(&row))
            .collect())
    }
}
