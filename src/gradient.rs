use crate::RGBf32;

pub type GradStops = Vec<(f32, RGBf32)>;

pub trait SampleLinear {
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
            return RGBf32::BLACK;
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

                // if we are sampling from the same index,
                // then no need to lerp the colour values
                //
                if i == i0 {
                    return c;
                }

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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_grad() -> GradStops {
        vec![
            (0.0, RGBf32::RED),
            (0.5, RGBf32::GREEN),
            (1.0, RGBf32::BLUE),
        ]
    }

    #[test]
    fn test_grad() {
        let grad = create_grad();

        assert_eq!(grad.sample(0.0), RGBf32::RED);
        assert_eq!(grad.sample(0.5), RGBf32::GREEN);
        assert_eq!(grad.sample(1.0), RGBf32::BLUE);
    }
}
