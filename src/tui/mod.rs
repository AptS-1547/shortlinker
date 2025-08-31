use std::{error::Error, io};

use ratatui::{
    Terminal,
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};

mod app;
mod ui;
use app::{App, CurrentScreen, CurrentlyEditing};
use ui::ui;

pub async fn run_tui() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stderr = io::stderr(); // This is a special case. Normally using stdout is fine
    execute!(stderr, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stderr);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let mut app = App::new().await?;
    let res = run_app(&mut terminal, &mut app).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

async fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = event::read()? {
            match app.current_screen {
                CurrentScreen::Main => match key.code {
                    KeyCode::Up => {
                        app.move_selection_up();
                    }
                    KeyCode::Down => {
                        app.move_selection_down();
                    }
                    KeyCode::Char('a') => {
                        app.current_screen = CurrentScreen::AddLink;
                        app.currently_editing = Some(CurrentlyEditing::ShortCode);
                        app.clear_inputs();
                    }
                    KeyCode::Char('e') => {
                        if !app.links.is_empty() {
                            app.current_screen = CurrentScreen::EditLink;
                            app.currently_editing = Some(CurrentlyEditing::TargetUrl);
                            app.clear_inputs();
                        }
                    }
                    KeyCode::Char('d') => {
                        if !app.links.is_empty() {
                            app.current_screen = CurrentScreen::DeleteConfirm;
                        }
                    }
                    KeyCode::Char('x') => {
                        app.current_screen = CurrentScreen::ExportImport;
                    }
                    KeyCode::Char('q') => {
                        app.current_screen = CurrentScreen::Exiting;
                    }
                    _ => {}
                },
                CurrentScreen::AddLink => match key.code {
                    KeyCode::Enter => {
                        if let Err(e) = app.save_new_link().await {
                            app.set_error(format!("Failed to save link: {}", e));
                        } else {
                            app.set_status("Link added successfully!".to_string());
                            app.current_screen = CurrentScreen::Main;
                            if let Err(e) = app.refresh_links().await {
                                app.set_error(format!("Failed to refresh links: {}", e));
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::ShortCode => {
                                    app.short_code_input.pop();
                                }
                                CurrentlyEditing::TargetUrl => {
                                    app.target_url_input.pop();
                                }
                                CurrentlyEditing::ExpireTime => {
                                    app.expire_time_input.pop();
                                }
                                CurrentlyEditing::Password => {
                                    app.password_input.pop();
                                }
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.clear_inputs();
                    }
                    KeyCode::Tab => {
                        app.toggle_editing();
                    }
                    KeyCode::Char(' ') => {
                        if matches!(app.currently_editing, Some(CurrentlyEditing::ShortCode)) {
                            app.force_overwrite = !app.force_overwrite;
                        }
                    }
                    KeyCode::Char(value) => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::ShortCode => {
                                    app.short_code_input.push(value);
                                }
                                CurrentlyEditing::TargetUrl => {
                                    app.target_url_input.push(value);
                                }
                                CurrentlyEditing::ExpireTime => {
                                    app.expire_time_input.push(value);
                                }
                                CurrentlyEditing::Password => {
                                    app.password_input.push(value);
                                }
                            }
                        }
                    }
                    _ => {}
                },
                CurrentScreen::EditLink => match key.code {
                    KeyCode::Enter => {
                        if let Err(e) = app.update_selected_link().await {
                            app.set_error(format!("Failed to update link: {}", e));
                        } else {
                            app.set_status("Link updated successfully!".to_string());
                            app.current_screen = CurrentScreen::Main;
                            if let Err(e) = app.refresh_links().await {
                                app.set_error(format!("Failed to refresh links: {}", e));
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TargetUrl => {
                                    app.target_url_input.pop();
                                }
                                CurrentlyEditing::ExpireTime => {
                                    app.expire_time_input.pop();
                                }
                                CurrentlyEditing::Password => {
                                    app.password_input.pop();
                                }
                                _ => {}
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                        app.clear_inputs();
                    }
                    KeyCode::Tab => {
                        app.toggle_editing();
                    }
                    KeyCode::Char(value) => {
                        if let Some(editing) = &app.currently_editing {
                            match editing {
                                CurrentlyEditing::TargetUrl => {
                                    app.target_url_input.push(value);
                                }
                                CurrentlyEditing::ExpireTime => {
                                    app.expire_time_input.push(value);
                                }
                                CurrentlyEditing::Password => {
                                    app.password_input.push(value);
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                },
                CurrentScreen::DeleteConfirm => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Err(e) = app.delete_selected_link().await {
                            app.set_error(format!("Failed to delete link: {}", e));
                        } else {
                            app.set_status("Link deleted successfully!".to_string());
                            if let Err(e) = app.refresh_links().await {
                                app.set_error(format!("Failed to refresh links: {}", e));
                            }
                        }
                        app.current_screen = CurrentScreen::Main;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
                CurrentScreen::ExportImport => match key.code {
                    KeyCode::Char('e') => {
                        if let Err(e) = app.export_links().await {
                            app.set_error(format!("Failed to export links: {}", e));
                        } else {
                            app.set_status(format!("Links exported to: {}", app.export_path));
                        }
                    }
                    KeyCode::Char('i') => {
                        if let Err(e) = app.import_links().await {
                            app.set_error(format!("Failed to import links: {}", e));
                        } else {
                            app.set_status("Links imported successfully!".to_string());
                            if let Err(e) = app.refresh_links().await {
                                app.set_error(format!("Failed to refresh links: {}", e));
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
                CurrentScreen::Exiting => match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        return Ok(());
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        app.current_screen = CurrentScreen::Main;
                    }
                    _ => {}
                },
            }
        }
    }
}
