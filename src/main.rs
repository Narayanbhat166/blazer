use blazer::app::model::Model;



use tuirealm::{PollStrategy, Update};

// fn main() -> Result<()> {
//     stdout().execute(EnterAlternateScreen)?;
//     enable_raw_mode()?;
//     let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
//     terminal.clear()?;

//     loop {
//         terminal.draw(|frame| {
//             let area = frame.size();
//             frame.render_widget(
//                 Paragraph::new("Wow such empty!").alignment(ratatui::layout::Alignment::Center),
//                 area,
//             );
//         })?;

//         if event::poll(std::time::Duration::from_millis(16))? {
//             if let event::Event::Key(key) = event::read()? {
//                 if key.kind == KeyEventKind::Press && key.code == KeyCode::Char('q') {
//                     break;
//                 }
//             }
//         }
//     }

//     stdout().execute(LeaveAlternateScreen)?;
//     disable_raw_mode()?;
//     Ok(())
// }

fn main() {
    // Setup model
    let mut model = Model::default();
    // Enter alternate screen
    let _ = model.terminal.enter_alternate_screen();
    let _ = model.terminal.enable_raw_mode();
    // Main loop
    // NOTE: loop until quit; quit is set in update if AppClose is received from counter
    while !model.quit {
        // Tick
        match model.app.tick(PollStrategy::Once) {
            Err(_err) => {}
            Ok(messages) if messages.len() > 0 => {
                // NOTE: redraw if at least one msg has been processed
                model.redraw = true;
                for msg in messages.into_iter() {
                    let mut msg = Some(msg);
                    while msg.is_some() {
                        msg = model.update(msg);
                    }
                }
            }
            _ => {}
        }
        // Redraw
        if model.redraw {
            model.view();
            model.redraw = false;
        }
    }
    // Terminate terminal
    let _ = model.terminal.leave_alternate_screen();
    let _ = model.terminal.disable_raw_mode();
    let _ = model.terminal.clear_screen();
}
