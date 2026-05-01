mod add;
mod app;
mod info;
mod search;
mod ui;

use app::{App, CrateDetail, CrateInfo, Focus};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{io, path::PathBuf, sync::mpsc, time::Duration};

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.in_rust_project = find_rust_project_root().is_some();

    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    result
}

fn find_rust_project_root() -> Option<PathBuf> {
    let mut dir = std::env::current_dir().ok()?;
    loop {
        if dir.join("Cargo.toml").exists() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}

fn run(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> io::Result<()> {
    let mut search_rx: Option<mpsc::Receiver<Result<Vec<CrateInfo>, String>>> = None;
    let mut info_rx: Option<mpsc::Receiver<Result<CrateDetail, String>>> = None;
    let mut add_rx: Option<mpsc::Receiver<Result<String, String>>> = None;

    loop {
        terminal.draw(|f| ui::ui(f, app))?;

        if let Some(rx) = &search_rx {
            match rx.try_recv() {
                Ok(Ok(results)) => {
                    app.selected = 0;
                    app.results = results;
                    app.is_searching = false;
                    app.error = None;
                    if !app.results.is_empty() {
                        app.focus = Focus::Results;
                    }
                    search_rx = None;
                }
                Ok(Err(e)) => {
                    app.error = Some(e);
                    app.is_searching = false;
                    search_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.is_searching = false;
                    search_rx = None;
                }
            }
        }

        if let Some(rx) = &info_rx {
            match rx.try_recv() {
                Ok(result) => {
                    let pending = app.pending_feature_select;
                    app.detail = Some(result);
                    app.is_loading_detail = false;
                    app.pending_feature_select = false;
                    info_rx = None;
                    if pending && app.detail.as_ref().map(|r| r.is_ok()).unwrap_or(false) {
                        app.enter_feature_select();
                    }
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.is_loading_detail = false;
                    app.pending_feature_select = false;
                    info_rx = None;
                }
            }
        }

        if let Some(rx) = &add_rx {
            match rx.try_recv() {
                Ok(result) => {
                    app.add_result = Some(result);
                    app.is_adding = false;
                    app.feature_select = None;
                    app.focus = Focus::Results;
                    add_rx = None;
                }
                Err(mpsc::TryRecvError::Empty) => {}
                Err(mpsc::TryRecvError::Disconnected) => {
                    app.is_adding = false;
                    add_rx = None;
                }
            }
        }

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('c') && key.modifiers == KeyModifiers::CONTROL {
                    return Ok(());
                }

                match app.focus {
                    Focus::Input => match (key.code, key.modifiers) {
                        (KeyCode::Esc, _) => return Ok(()),
                        (KeyCode::Enter, _) => {
                            let query = app.input.trim().to_string();
                            if !query.is_empty() && !app.is_searching {
                                app.is_searching = true;
                                app.results.clear();
                                app.detail = None;
                                app.add_result = None;
                                app.error = None;
                                let (tx, rx) = mpsc::channel();
                                search_rx = Some(rx);
                                std::thread::spawn(move || {
                                    let _ = tx.send(search::search(&query));
                                });
                            }
                        }
                        (KeyCode::Down, _) if !app.results.is_empty() => {
                            app.focus = Focus::Results;
                        }
                        (KeyCode::Backspace, _) => app.delete_char(),
                        (KeyCode::Char(c), _) => app.insert_char(c),
                        _ => {}
                    },

                    Focus::Results => match key.code {
                        KeyCode::Esc => {
                            app.focus = Focus::Input;
                        }
                        KeyCode::Up => {
                            if app.selected == 0 {
                                app.focus = Focus::Input;
                            } else {
                                app.add_result = None;
                                app.select_prev();
                            }
                        }
                        KeyCode::Down => {
                            app.add_result = None;
                            app.select_next();
                        }
                        KeyCode::Enter => {
                            if !app.is_loading_detail {
                                if let Some(crate_info) = app.results.get(app.selected) {
                                    let name = crate_info.name.clone();
                                    app.is_loading_detail = true;
                                    app.detail = None;
                                    app.add_result = None;
                                    app.pending_feature_select = false;
                                    let (tx, rx) = mpsc::channel();
                                    info_rx = Some(rx);
                                    std::thread::spawn(move || {
                                        let _ = tx.send(info::get_info(&name));
                                    });
                                }
                            }
                        }
                        KeyCode::Char('a') if app.in_rust_project => {
                            if app.is_loading_detail {
                                app.pending_feature_select = true;
                            } else {
                                match &app.detail {
                                    Some(Ok(_)) => {
                                        app.add_result = None;
                                        app.enter_feature_select();
                                    }
                                    _ => {
                                        if let Some(crate_info) = app.results.get(app.selected) {
                                            let name = crate_info.name.clone();
                                            app.is_loading_detail = true;
                                            app.detail = None;
                                            app.add_result = None;
                                            app.pending_feature_select = true;
                                            let (tx, rx) = mpsc::channel();
                                            info_rx = Some(rx);
                                            std::thread::spawn(move || {
                                                let _ = tx.send(info::get_info(&name));
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    },

                    Focus::FeatureSelect => {
                        if !app.is_adding {
                            match key.code {
                                KeyCode::Esc => {
                                    app.feature_select = None;
                                    app.focus = Focus::Results;
                                }
                                KeyCode::Up => {
                                    if let Some(fs) = &mut app.feature_select {
                                        fs.move_up();
                                    }
                                }
                                KeyCode::Down => {
                                    if let Some(fs) = &mut app.feature_select {
                                        fs.move_down();
                                    }
                                }
                                KeyCode::Char(' ') => {
                                    if let Some(fs) = &mut app.feature_select {
                                        fs.toggle_current();
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(fs) = &app.feature_select {
                                        let name = fs.crate_name.clone();
                                        let features = fs.selected_features();
                                        let no_default = fs.no_default_features;
                                        app.is_adding = true;
                                        let (tx, rx) = mpsc::channel();
                                        add_rx = Some(rx);
                                        std::thread::spawn(move || {
                                            let _ = tx
                                                .send(add::add_crate(&name, &features, no_default));
                                        });
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
    }
}
