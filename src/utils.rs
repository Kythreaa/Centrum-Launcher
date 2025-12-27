pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let i = (h / 60.0).floor() as i32;
    let f = h / 60.0 - i as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
pub fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;
    let mut h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };
    if h < 0.0 { h += 360.0; }
    let s = if max == 0.0 { 0.0 } else { delta / max };
    (h, s, max)
}
pub fn detect_terminal() -> String {
    let terminals = ["kitty", "alacritty", "ghostty", "foot", "wezterm", "gnome-terminal", "konsole", "xterm"];
    for term in terminals {
        if std::process::Command::new("which").arg(term).output().map(|o| o.status.success()).unwrap_or(false) {
            return term.to_string();
        }
    }
    "xterm".to_string()
}