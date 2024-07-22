use crate::{GradStops, RGBf32, SampleLinear};
use rand::seq::SliceRandom;
use rand::Rng;
use tui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    widgets::StatefulWidget,
};

impl From<RGBf32> for tui::style::Color {
    fn from(value: RGBf32) -> Self {
        tui::style::Color::Rgb(
            (value.r * 255.0) as u8,
            (value.g * 255.0) as u8,
            (value.b * 255.0) as u8,
        )
    }
}

#[derive(Copy, Clone)]
struct Raindrop {
    pub pos: i32,
    pub speed: u16,
}

pub struct MatrixWidget {}
pub struct MatrixWidgetState {
    frame: u16,
    drops: Vec<Raindrop>,
    area: Rect,
    tail: u16,
    grad: GradStops,
    chars: String,
    rng: rand::prelude::ThreadRng,
}

impl MatrixWidgetState {
    pub fn new(tail: u16, chars: impl ToString, grad: GradStops) -> Self {
        Self {
            frame: 0,
            area: Rect::new(0, 0, 0, 0),
            drops: Vec::new(),
            tail,
            grad,
            chars: chars.to_string(),
            rng: rand::thread_rng(),
        }
    }
}

impl MatrixWidget {
    fn init(&self, area: Rect, buf: &mut Buffer, state: &mut MatrixWidgetState) {
        // initialize raindrops with random starting location
        // above the visible area, and with a random movement
        // speed
        //
        let mut drops = Vec::new();

        for _ in 0..area.width {
            let d = Raindrop {
                pos: -(state.rng.gen_range(0..area.height + state.tail) as i32),
                speed: state.rng.gen_range(1..=4),
            };
            drops.push(d);
        }

        state.drops = drops;

        // initialize buffer with random characters; we do this
        // so that we avoid allocations by swapping charaters
        // that already exist on the screen using std::mem::swap
        //
        let cset = state
            .chars
            .chars()
            .into_iter()
            .map(|c| c.to_string())
            .collect::<Vec<String>>();

        for y in area.y..area.height {
            for x in area.x..area.width {
                let cell = buf.get_mut(x, y);
                cell.symbol = cset.choose(&mut state.rng).unwrap().clone();
                cell.fg = state.grad[0].1.into();
            }
        }

        // update our initialized area; if we detect this
        // changes, then we re-initialize!
        //
        state.area = area;
    }
}

impl StatefulWidget for MatrixWidget {
    type State = MatrixWidgetState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if state.area != area {
            self.init(area, buf, state);
        }

        // update the frame counter (we wrap at factorial(RAINDROP_MAX_SPEED) frame-count)
        //
        state.frame += 1;
        state.frame %= 3 * 2 * 1; // max speed factorial

        // update the raindrops
        //
        for d in state.drops.iter_mut() {
            if state.frame % d.speed == 0 {
                d.pos += 1;
                if d.pos > area.height as i32 + state.tail as i32 {
                    d.pos -= area.height as i32 * 2;
                }
            }
        }

        for y in area.y..area.height {
            for x in area.x..area.width {
                let drop = state.drops[x as usize];

                // update anything?
                //
                if state.frame % drop.speed == 0 {
                    // should we update the symbol?
                    //
                    let is_head = y as i32 == drop.pos;
                    let is_rand = state.rng.gen_bool(0.05);

                    if is_head || is_rand {
                        random_swap_cell_symbol(buf, x, y, state, area);
                    }

                    // calculate rain gradient colour
                    //
                    let cell = buf.get_mut(x, y);

                    let r = ((drop.pos - state.tail as i32) as f32)..(drop.pos as f32);

                    if let Some(v) = r.sample(y as f32) {
                        // get a blended colour from the gradient
                        //
                        cell.fg = state.grad.sample(v).into();
                    } else {
                        // use the first gradient stop colour
                        //
                        cell.fg = state.grad[0].1.into();
                    }
                }
            }
        }
    }
}

fn random_swap_cell_symbol(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    state: &mut MatrixWidgetState,
    area: Rect,
) {
    let rx = state.rng.gen_range(area.x..area.width);
    let ry = state.rng.gen_range(area.y..area.height);

    if rx != x || ry != y {
        let mut i0 = buf.index_of(x, y);
        let mut i1 = buf.index_of(rx, ry);

        // swap indexes so lower is used as split point
        //
        if i0 > i1 {
            (i0, i1) = (i1, i0);
        }

        let (s0, s1) = buf.content.as_mut_slice().split_at_mut(i0 + 1);

        let c0 = &mut s0[i0];
        let c1 = &mut s1[i1 - i0 - 1];

        std::mem::swap(&mut c0.symbol, &mut c1.symbol);
    }
}
