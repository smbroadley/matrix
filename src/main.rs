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

fn main() -> std::io::Result<()> {
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.hide_cursor()?;

    let grad: GradStops = vec![
        (0.0, RGBf32::BLACK),
        (0.8, RGBf32::GREEN),
        (1.0, RGBf32::WHITE),
    ];

    let mut state = MatrixWidgetState::new(20, "8=ｱｲｳｷｸｵﾔﾃﾂﾕ", grad);

    loop {
        if poll(Duration::from_millis(60))? {
            break;
        }

        let mut f = terminal.get_frame();

        f.render_stateful_widget(MatrixWidget {}, f.size(), &mut state);

        terminal.flush()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_grad() -> GradStops {
        vec![
            (0.0, RGBf32::new(0.0, 0.0, 0.0)),
            (0.5, RGBf32::new(0.0, 1.0, 0.0)),
            (1.0, RGBf32::new(1.0, 1.0, 1.0)),
        ]
    }

    #[test]
    fn test_grad() {
        let grad = create_grad();
        let white = RGBf32::new(1.0, 1.0, 1.0);

        assert_eq!(grad.sample(1.0), white);
    }
}
