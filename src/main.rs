use clap::{Parser, Subcommand};
use rusqlite::{params, Connection, Result};
use chrono::{ Local};

// Command-line argument parser
#[derive(Parser)]
#[command(name = "time-tracking")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(value_name = "PROJECT")]
        project: String,
    },
    Stop,
    Report,
}

// Function to initialize the database and create table if not exists
fn initialize_db() -> Result<Connection> {
    let conn = Connection::open("time_tracking.db")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS time_entries (
            id INTEGER PRIMARY KEY,
            project TEXT NOT NULL,
            start_time INTEGER NOT NULL,
            stop_time INTEGER
        )",
        [],
    )?;
    Ok(conn)
}

// Start tracking time for a project
fn start_tracking(conn: &Connection, project: &str) -> Result<()> {
    let start_time = Local::now().timestamp();
    conn.execute(
        "INSERT INTO time_entries (project, start_time) VALUES (?1, ?2)",
        params![project, start_time],
    )?;
    println!("Started tracking time for project '{}'", project);
    Ok(())
}

// Stop tracking time for the most recent entry without a stop time
fn stop_tracking(conn: &Connection) -> Result<()> {
    let stop_time = Local::now().timestamp();

    let mut stmt = conn.prepare(
        "SELECT id, project, start_time FROM time_entries
         WHERE stop_time IS NULL
         ORDER BY start_time DESC
         LIMIT 1",
    )?;
    let entry = stmt.query_row([], |row| {
        let id: i32 = row.get(0)?;
        let project: String = row.get(1)?;
        let start_time: i64 = row.get(2)?;
        Ok((id, project, start_time))
    });

    match entry {
        Ok((id, project, start_time)) => {
            conn.execute(
                "UPDATE time_entries SET stop_time = ?1 WHERE id = ?2",
                params![stop_time, id],
            )?;
            let duration = stop_time - start_time;
            println!(
                "Stopped tracking for project '{}'. Time tracked: {} seconds",
                project, duration
            );
        }
        Err(_) => println!("No active project found to stop."),
    }
    Ok(())
}

// Report total time spent on each project
fn report(conn: &Connection) -> Result<()> {
    let mut stmt = conn.prepare(
        "SELECT project, SUM(stop_time - start_time) AS total_time
         FROM time_entries
         WHERE stop_time IS NOT NULL
         GROUP BY project",
    )?;
    let mut rows = stmt.query([])?;

    println!("Time spent on each project:");
    while let Some(row) = rows.next()? {
        let project: String = row.get(0)?;
        let total_time: i64 = row.get(1)?;
        println!("- {}: {:.2} hours", project, total_time as f64 / 3600.0);
    }
    Ok(())
}

// Main function
fn main() -> Result<()> {
    let args = Cli::parse();
    let conn = initialize_db()?;

    match args.command {
        Commands::Start { project } => start_tracking(&conn, &project)?,
        Commands::Stop => stop_tracking(&conn)?,
        Commands::Report => report(&conn)?,
    }
    Ok(())
}
