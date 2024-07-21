use crate::GradStops;
use crate::{gradient::SampleLinear, RGBf32};
use rand::Rng;
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
    area: Rect,
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
            area: Rect::new(0, 0, 0, 0),
            drops: Vec::new(),
            tail,
            grad,
            cset,
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
            let r = rand::RngCore::next_u32(&mut state.rng) as u16;
            let d = Drop {
                pos: -((r % (area.height * 2 + state.tail)) as i32),
                speed: 1 + (r % 3),
            };
            drops.push(d);
        }

        state.drops = drops;

        // initialize buffer with random characters; we do this
        // so that we avoid allocations when randomizing the
        // screen contents
        //
        for y in 0..area.height {
            for x in 0..area.width {
                let cell = buf.get_mut(x, y);
                cell.symbol = state.cset.choose(&mut state.rng).unwrap().clone();
            }
        }

        // indicate successful init for this area
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

        for y in 0..area.height {
            for x in 0..area.width {
                let p = state.drops.get(x as usize).unwrap().pos;

                // should we update the symbol?
                //
                let is_head = y as i32 == p;
                let is_rand = state.rng.gen_bool(0.05);

                if is_head || is_rand {
                    // swap with another character in the
                    // buffer to avoid allocations
                    //
                    let mut temp = String::new(); // (no alloc)

                    // x,y <==> temp
                    {
                        let cell = buf.get_mut(x, y);
                        std::mem::swap(&mut cell.symbol, &mut temp);
                    }

                    // temp <==> rx, ry
                    {
                        let rx = state.rng.gen_range(0..area.width);
                        let ry = state.rng.gen_range(0..area.height);

                        if rx != x || ry != y {
                            let cell = buf.get_mut(rx, ry);
                            std::mem::swap(&mut cell.symbol, &mut temp);
                        }
                    }

                    // x,y <==> temp
                    {
                        let cell = buf.get_mut(x, y);
                        std::mem::swap(&mut cell.symbol, &mut temp);
                    }
                }

                // calculate rain gradient colour
                //
                let cell = buf.get_mut(x, y);

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
