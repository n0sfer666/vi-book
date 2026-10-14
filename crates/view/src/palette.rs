use crate::ansi::Ansi;

type Rgb = (u8, u8, u8);

const LEVELS: [u8; 6] = [0, 95, 135, 175, 215, 255];

pub(crate) fn index_rgb(index: u8) -> Rgb {
    if index < 16 {
        return Ansi::ALL[usize::from(index)].rgb();
    }
    if index >= 232 {
        let v = 8 + 10 * (index - 232);
        return (v, v, v);
    }
    let n = usize::from(index - 16);
    (LEVELS[n / 36], LEVELS[n / 6 % 6], LEVELS[n % 6])
}

pub(crate) fn nearest_256(rgb: Rgb) -> u8 {
    (16..=255)
        .min_by_key(|&index| distance(index_rgb(index), rgb))
        .unwrap_or(16)
}

pub(crate) fn nearest_16(rgb: Rgb) -> Ansi {
    Ansi::ALL
        .into_iter()
        .min_by_key(|ansi| distance(ansi.rgb(), rgb))
        .unwrap_or(Ansi::Black)
}

fn distance(a: Rgb, b: Rgb) -> i32 {
    let d = |p: u8, q: u8| (i32::from(p) - i32::from(q)).pow(2);
    d(a.0, b.0) + d(a.1, b.1) + d(a.2, b.2)
}
