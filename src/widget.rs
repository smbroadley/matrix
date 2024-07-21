use crate::GradStops;
use crate::{gradient::SampleLinear, RGBf32};
use rand::{seq::SliceRandom, RngCore};
use tui::{buffer::Buffer, layout::Rect, widgets::StatefulWidget};

impl From<RGBf32> for tui::style::Color {
    fn from(value: RGBf32) -> Self {
        tui::style::Color::Rgb(
            (value.r * 255.0) as u8,
            (value.g * 255.0) as u8,
            (value.b * 255.0) as u8,
        )
    }
}

pub struct Drop {
    pub pos: i32,
    pub speed: u16,
}

pub struct MatrixWidget {}
pub struct MatrixWidgetState {
    frame: u16,
    drops: Vec<Drop>,
    width: u16,
    tail: u16,
    grad: GradStops,
    cset: Vec<String>,
    rng: rand::prelude::ThreadRng,
}

impl MatrixWidgetState {
    pub fn new(tail: u16, chars: &str, grad: GradStops) -> Self {
        // create the character set that we are going to
        // use for the character-swap-fx.jk
        //
        let cset = chars
            .chars()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        Self {
            frame: 0,
            width: 0,
            drops: Vec::new(),
            tail,
            grad,
            cset,
            rng: rand::thread_rng(),
        }
    }
}

impl StatefulWidget for MatrixWidget {
    type State = MatrixWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.width != area.width {
            let mut drops = Vec::new();
            for _ in 0..area.width {
                let r = rand::RngCore::next_u32(&mut state.rng) as u16;
                let d = Drop {
                    pos: -((r % (area.height * 2 + state.tail)) as i32),
                    speed: 1 + (r % 3),
                };
                drops.push(d);
            }
            state.drops = drops;
            state.width = area.width;
        }

        // update the frame counter (we wrap at 16 frames)
        state.frame += 1;
        state.frame %= 16;

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

                // calculate raindrop gradient colour...
                //
                let r = ((p - state.tail as i32) as f32)..(p as f32);

                if let Some(v) = r.sample(y as f32) {
                    cell.fg = state.grad.sample(v).into();
                } else {
                    cell.fg = tui::style::Color::Black;
                }
            }
        }
    }
}
