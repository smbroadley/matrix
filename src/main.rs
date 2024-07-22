use crossterm::event::poll;
use std::time::Duration;
use tui::backend::CrosstermBackend;
use tui::Terminal;

mod gradient;
mod rgbf32;
mod widget;
use crate::gradient::*;
use crate::rgbf32::*;
use crate::widget::*;

// tracking allocator (used in 'Debug' target)
//
use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use std::alloc::System;

#[cfg_attr(Debug, global_allocator)]
static GLOBAL: &StatsAlloc<System> = &INSTRUMENTED_SYSTEM;

fn main() -> std::io::Result<()> {
    let reg = Region::new(&GLOBAL);

    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let gradient: GradStops = vec![
        (0.0, RGBf32::BLACK),
        (0.8, RGBf32::new(0.0, 1.0, 0.4)),
        (1.0, RGBf32::new(0.8, 1.0, 0.8)),
    ];

    let mut state =
        MatrixWidgetState::new(20, "STEPHENBROADLEY249=ｱｲｳｷｸｵﾔﾃﾂﾕﾐﾑﾒﾓﾕﾖﾗﾘﾛﾜﾄｿｽｻ", gradient);

    loop {
        let mut frame = terminal.get_frame();

        frame.render_stateful_widget(MatrixWidget {}, frame.size(), &mut state);

        terminal.flush()?;

        if poll(Duration::from_millis(60))? {
            break;
        }
    }

    if cfg!(Debug) {
        println!("\nStats at exit: {:#?}", reg.change());
    }

    Ok(())
}
