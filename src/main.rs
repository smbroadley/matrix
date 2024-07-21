use crossterm::event::poll;
use rand::{seq::SliceRandom, RngCore};
use std::time::Duration;
use tui::{backend::CrosstermBackend, widgets::StatefulWidget, *};

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
struct RGBf32 {
    r: f32,
    g: f32,
    b: f32,
}

impl RGBf32 {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    pub fn lerp(&self, o: RGBf32, w: f32) -> RGBf32 {
        let x = 1.0 - w;

        RGBf32 {
            r: (self.r * w) + (o.r * x),
            g: (self.g * w) + (o.g * x),
            b: (self.b * w) + (o.b * x),
        }
    }
}

impl From<RGBf32> for tui::style::Color {
    fn from(value: RGBf32) -> Self {
        tui::style::Color::Rgb(
            (value.r * 255.0) as u8,
            (value.g * 255.0) as u8,
            (value.b * 255.0) as u8,
        )
    }
}

type GradStops = Vec<(f32, RGBf32)>;

trait SampleLinear {
    type Output;
    fn sample(&self, v: f32) -> Self::Output;
}

impl SampleLinear for std::ops::Range<f32> {
    type Output = Option<f32>;

    fn sample(&self, v: f32) -> Self::Output {
        if v < self.start || v > self.end {
            None
        } else {
            Some((v - self.start) / (self.end - self.start))
        }
    }
}

impl SampleLinear for GradStops {
    type Output = RGBf32;

    fn sample(&self, s: f32) -> Self::Output {
        let stops = self.len();
        if stops == 0 {
            return RGBf32::new(0.0, 0.0, 0.0);
        }

        if stops == 1 {
            return self[0].1;
        }

        for (i, &(v, c)) in self.iter().enumerate() {
            if v >= s {
                // we want to select the previoeous two
                // indexes at the interpolation values...
                //
                let i0 = i.saturating_sub(1);

                let (s0, c0) = self[i0];
                let (s1, c1) = (v, c);

                // calculate the weight for the two color
                // to blend (the weight for the seconds one
                // if the reciprical of the first.... simples)
                //
                let w = (s - s0) / (s1 - s0);

                return c1.lerp(c0, w);
            }
        }

        // we must be sampling past the last stop...
        //
        return self[stops - 1].1;
    }
}

struct MatrixWidget {}
struct MatrixWidgetState {
    frame: usize,
    drops: Vec<Drop>,
    tail: i32,
    grad: GradStops,
    cset: Vec<String>,
    rng: rand::prelude::ThreadRng,
}

impl StatefulWidget for MatrixWidget {
    type State = MatrixWidgetState;

    fn render(self, area: layout::Rect, buf: &mut buffer::Buffer, state: &mut Self::State) {
        state.frame += 1;

        // update the raindrop effect...
        //
        for d in state.drops.iter_mut() {
            if state.frame % d.speed == 0 {
                d.pos += 1;
                if d.pos > area.height as i32 + state.tail as i32 {
                    d.pos -= area.height as i32 * 2;
                }
            }
        }

        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.get_mut(x, y);
                let p = state.drops.get(x as usize).unwrap().pos;

                // should we update the symbol?
                //
                let is_head = y as i32 == p;
                let is_rand = state.rng.next_u32() % 16 == 0;

                if is_head || is_rand {
                    cell.symbol = state.cset.choose(&mut state.rng).unwrap().clone();
                }

                // calculate proximity to raindrop...
                //
                let r = ((p - state.tail) as f32)..(p as f32);

                if let Some(v) = r.sample(y as f32) {
                    cell.fg = state.grad.sample(v).into();
                } else {
                    cell.fg = tui::style::Color::Black;
                }
            }
        }
    }
}

struct Drop {
    pos: i32,
    speed: usize,
}

fn main() -> std::io::Result<()> {
    // initialiaz the TUI system; we use TUI to do the
    // updates in an efficient way, and use its buffers
    // to make sure all changes are flushed at once...
    //
    let stdout = std::io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut rng = rand::thread_rng();

    terminal.hide_cursor()?;

    // create the gradient for use in the rain-fx.
    //
    let grad: GradStops = vec![
        (0.0, RGBf32::new(0.0, 0.0, 0.0)),
        (0.8, RGBf32::new(0.0, 1.0, 0.0)),
        (1.0, RGBf32::new(1.0, 1.0, 1.0)),
    ];

    // create the character set that we are going to
    // use for the character-swap-fx.jk
    //
    let cset = "8=ｱｲｳｷｸｵﾔﾃﾂﾕ"
        .chars()
        .into_iter()
        .map(|c| c.to_string())
        .collect::<Vec<String>>();

    // create the initial raindrops above the top of
    // the screen so that we start with a blank screen,
    // then see drops appear...
    //
    let mut drops = Vec::new();
    for _ in 0..terminal.size()?.width {
        let r = rand::RngCore::next_u32(&mut rng) as usize;
        let d = Drop {
            pos: -((r % (terminal.size()?.height as usize * 2 + 15)) as i32),
            speed: 1 + (r % 3),
        };
        drops.push(d);
    }

    // create the MatrixWidget and initialize with default settings
    //
    let mut state = MatrixWidgetState {
        frame: 0,
        drops,
        tail: 15,
        grad,
        cset,
        rng,
    };

    // run the effect until a key is pressed
    //
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
