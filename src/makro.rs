//! Makros: Datenmodell, Dateiformat, Beschreibung und Ablaufsteuerung.
//! Ohne Windows-Abhaengigkeiten, damit es sich ueberall testen laesst.

/// Ein Schritt eines Makros. Sprungziele sind 1-basierte Schrittnummern,
/// 0 bedeutet "mit dem naechsten Schritt weitermachen".
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    /// Klick mit Maustaste 0 (Mouse 1) oder 1 (Mouse 2), optional an fester Position.
    Click { button: u32, pos: Option<(i32, i32)> },
    /// Pause in Millisekunden.
    Wait(u64),
    /// Prueft einmal die Farbe; passt sie nicht, wird zu `else_goto` gesprungen.
    IfColor { pos: (i32, i32), rgb: u32, tol: u8, else_goto: usize },
    /// Wartet, bis die Farbe erscheint (timeout 0 = unbegrenzt); bei Zeitablauf Sprung zu `else_goto`.
    WaitColor { pos: (i32, i32), rgb: u32, tol: u8, timeout: u64, else_goto: usize },
    /// Springt zu Schritt `n`.
    Goto(usize),
}

/// Namen der Schritt-Typen in der Reihenfolge von `Step::kind()`.
pub const KIND_NAMES: [&str; 5] = ["Klick", "Warten", "Wenn Farbe, sonst springen", "Auf Farbe warten", "Zu Schritt springen"];

/// Was nach einem Schritt passiert.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Flow {
    Next,
    Jump(usize),
}

impl Step {
    pub fn kind(self) -> usize {
        match self {
            Step::Click { .. } => 0,
            Step::Wait(_) => 1,
            Step::IfColor { .. } => 2,
            Step::WaitColor { .. } => 3,
            Step::Goto(_) => 4,
        }
    }

    /// Eine Zeile im Dateiformat.
    pub fn to_line(self) -> String {
        match self {
            Step::Click { button, pos: None } => format!("click {}", button + 1),
            Step::Click { button, pos: Some((x, y)) } => format!("click {} {} {}", button + 1, x, y),
            Step::Wait(ms) => format!("wait {}", ms),
            Step::IfColor { pos: (x, y), rgb, tol, else_goto } => {
                format!("ifcolor {} {} {:06X} {} {}", x, y, rgb, tol, else_goto)
            }
            Step::WaitColor { pos: (x, y), rgb, tol, timeout, else_goto } => {
                format!("waitcolor {} {} {:06X} {} {} {}", x, y, rgb, tol, timeout, else_goto)
            }
            Step::Goto(n) => format!("goto {}", n),
        }
    }

    /// Liest eine Zeile im Dateiformat (Gross-/Kleinschreibung egal).
    pub fn parse_line(line: &str) -> Option<Step> {
        let lower = line.trim().to_ascii_lowercase();
        let mut it = lower.split_whitespace();
        let cmd = it.next()?;
        let args: Vec<&str> = it.collect();
        let int = |i: usize| args.get(i).and_then(|v| v.parse::<i32>().ok());
        let uint = |i: usize| args.get(i).and_then(|v| v.parse::<u64>().ok());
        let color = |i: usize| args.get(i).and_then(|v| parse_hex(v));
        let tol = |i: usize| args.get(i).and_then(|v| v.parse::<u8>().ok());
        match (cmd, args.len()) {
            ("click", 1) | ("click", 3) => {
                let button = match args[0] {
                    "1" => 0,
                    "2" => 1,
                    _ => return None,
                };
                let pos = if args.len() == 3 { Some((int(1)?, int(2)?)) } else { None };
                Some(Step::Click { button, pos })
            }
            ("wait", 1) => Some(Step::Wait(uint(0)?)),
            ("ifcolor", 5) => Some(Step::IfColor {
                pos: (int(0)?, int(1)?),
                rgb: color(2)?,
                tol: tol(3)?,
                else_goto: uint(4)? as usize,
            }),
            ("waitcolor", 6) => Some(Step::WaitColor {
                pos: (int(0)?, int(1)?),
                rgb: color(2)?,
                tol: tol(3)?,
                timeout: uint(4)?,
                else_goto: uint(5)? as usize,
            }),
            ("goto", 1) => Some(Step::Goto(uint(0)? as usize)),
            _ => None,
        }
    }

    /// Kurzbeschreibung fuer die Liste.
    pub fn describe(self) -> String {
        let target = |n: usize| if n == 0 { "weiter".to_string() } else { format!("→ {}", n) };
        match self {
            Step::Click { button, pos: None } => format!("Klick  Mouse {}", button + 1),
            Step::Click { button, pos: Some((x, y)) } => format!("Klick  Mouse {}  ·  {} / {}", button + 1, x, y),
            Step::Wait(ms) => format!("Warten  {}", fmt_dur(ms)),
            Step::IfColor { pos: (x, y), rgb, else_goto, .. } => {
                format!("Wenn #{:06X} bei {} / {}  ·  sonst {}", rgb, x, y, target(else_goto))
            }
            Step::WaitColor { pos: (x, y), rgb, timeout, else_goto, .. } => {
                if timeout == 0 {
                    format!("Warte auf #{:06X} bei {} / {}", rgb, x, y)
                } else {
                    format!("Warte auf #{:06X} bei {} / {}  ·  max {} {}", rgb, x, y, fmt_dur(timeout), target(else_goto))
                }
            }
            Step::Goto(n) => format!("Springe zu Schritt {}", n),
        }
    }

    /// Farbe des Schritts (fuer das Farbfeld in der Liste).
    pub fn color(self) -> Option<u32> {
        match self {
            Step::IfColor { rgb, .. } | Step::WaitColor { rgb, .. } => Some(rgb),
            _ => None,
        }
    }
}

/// Naechster Schrittindex (0-basiert). Ein Index >= Anzahl beendet den Durchlauf.
pub fn advance(i: usize, flow: Flow) -> usize {
    match flow {
        Flow::Next | Flow::Jump(0) => i + 1,
        Flow::Jump(n) => n - 1,
    }
}

/// Vergleicht eine gelesene Farbe (0xRRGGBB) mit der Zielfarbe, Toleranz je Kanal.
pub fn color_matches(actual: u32, target: u32, tol: u8) -> bool {
    let tol = tol as i32;
    [16, 8, 0].iter().all(|&sh| {
        let a = ((actual >> sh) & 0xFF) as i32;
        let t = ((target >> sh) & 0xFF) as i32;
        (a - t).abs() <= tol
    })
}

/// "#34477A", "34477a", "Hex #34477A" -> 0x34477A
pub fn parse_hex(text: &str) -> Option<u32> {
    let t = text.trim();
    let t = t.strip_prefix("Hex").or_else(|| t.strip_prefix("hex")).unwrap_or(t);
    let t: String = t.chars().filter(|c| *c != '#' && !c.is_whitespace()).collect();
    if t.len() == 6 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        u32::from_str_radix(&t, 16).ok()
    } else {
        None
    }
}

pub fn fmt_dur(ms: u64) -> String {
    if ms >= 3_600_000 && ms.is_multiple_of(3_600_000) {
        format!("{} h", ms / 3_600_000)
    } else if ms >= 60_000 && ms.is_multiple_of(60_000) {
        format!("{} min", ms / 60_000)
    } else if ms >= 1000 {
        let mut t = format!("{:.3}", ms as f64 / 1000.0);
        while t.ends_with('0') {
            t.pop();
        }
        if t.ends_with('.') {
            t.pop();
        }
        format!("{} s", t.replace('.', ","))
    } else {
        format!("{} ms", ms)
    }
}

/// Inhalt einer Makro-Datei.
#[derive(Debug, Default, PartialEq)]
pub struct File {
    pub steps: Vec<Step>,
    pub looped: Option<bool>,
    pub rounds: Option<u32>,
    /// Anzahl uebersprungener, unbekannter Zeilen
    pub bad: usize,
}

pub fn parse_file(text: &str) -> File {
    let mut f = File::default();
    for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
        let lower = line.to_ascii_lowercase();
        if let Some(v) = lower.strip_prefix("loop=") {
            f.looped = Some(v.trim() == "1");
        } else if let Some(v) = lower.strip_prefix("rounds=") {
            f.rounds = v.trim().parse().ok();
        } else if let Some(st) = Step::parse_line(line) {
            f.steps.push(st);
        } else {
            f.bad += 1;
        }
    }
    f
}

pub fn write_file(steps: &[Step], looped: bool, rounds: u32) -> String {
    let mut out = String::from("# Maus-Utility Makro\r\n");
    out += "# click TASTE [X Y] | wait MS | ifcolor X Y RRGGBB TOL SONST | waitcolor X Y RRGGBB TOL MAX_MS SONST | goto N\r\n";
    out += &format!("loop={}\r\n", looped as u8);
    out += &format!("rounds={}\r\n", rounds);
    for st in steps {
        out += &st.to_line();
        out += "\r\n";
    }
    out
}

/// Passt Sprungziele an, wenn ein Schritt an `at` (0-basiert) eingefuegt wurde.
pub fn renumber_after_insert(steps: &mut [Step], at: usize) {
    let fix = |n: &mut usize| {
        if *n > at {
            *n += 1;
        }
    };
    for st in steps.iter_mut() {
        match st {
            Step::IfColor { else_goto, .. } | Step::WaitColor { else_goto, .. } => fix(else_goto),
            Step::Goto(n) => fix(n),
            _ => {}
        }
    }
}

/// Passt Sprungziele an, wenn der Schritt an `at` (0-basiert) entfernt wurde.
pub fn renumber_after_remove(steps: &mut [Step], at: usize) {
    let fix = |n: &mut usize| {
        if *n > at + 1 {
            *n -= 1;
        }
    };
    for st in steps.iter_mut() {
        match st {
            Step::IfColor { else_goto, .. } | Step::WaitColor { else_goto, .. } => fix(else_goto),
            Step::Goto(n) => fix(n),
            _ => {}
        }
    }
}

/// Passt Sprungziele an, wenn die Schritte `a` und `b` (0-basiert) getauscht wurden.
pub fn renumber_after_swap(steps: &mut [Step], a: usize, b: usize) {
    let fix = |n: &mut usize| {
        if *n == a + 1 {
            *n = b + 1;
        } else if *n == b + 1 {
            *n = a + 1;
        }
    };
    for st in steps.iter_mut() {
        match st {
            Step::IfColor { else_goto, .. } | Step::WaitColor { else_goto, .. } => fix(else_goto),
            Step::Goto(n) => fix(n),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Beispiel: klicken, bis an einer Stelle eine Farbe erscheint, dann zwei Buttons bestaetigen.
    const BEISPIEL: &str = "# Beispiel\r\nloop=1\r\nrounds=0\r\nclick 1 500 600\r\nwait 1000\r\n\
ifcolor 700 300 336699 10 1\r\nwait 500\r\nclick 1 900 200\r\nwait 1000\r\nclick 1 650 650\r\nwait 1000\r\n";

    #[test]
    fn roundtrip() {
        let steps = vec![
            Step::Click { button: 0, pos: Some((500, 600)) },
            Step::Click { button: 1, pos: None },
            Step::Wait(1500),
            Step::IfColor { pos: (700, 300), rgb: 0x336699, tol: 10, else_goto: 1 },
            Step::WaitColor { pos: (-5, 7), rgb: 0xFFFFFF, tol: 0, timeout: 10_000, else_goto: 0 },
            Step::Goto(2),
        ];
        let text = write_file(&steps, true, 0);
        let f = parse_file(&text);
        assert_eq!(f.steps, steps);
        assert_eq!(f.looped, Some(true));
        assert_eq!(f.rounds, Some(0));
        assert_eq!(f.bad, 0);
    }

    #[test]
    fn old_klickfolge_files_still_load() {
        let f = parse_file("loop=0\r\nrounds=3\r\nclick 1\r\nwait 1000\r\nclick 2 640 900\r\nquatsch\r\n");
        assert_eq!(
            f.steps,
            vec![
                Step::Click { button: 0, pos: None },
                Step::Wait(1000),
                Step::Click { button: 1, pos: Some((640, 900)) }
            ]
        );
        assert_eq!(f.bad, 1);
    }

    #[test]
    fn example_file_is_valid() {
        let f = parse_file(BEISPIEL);
        assert_eq!(f.bad, 0);
        assert_eq!(f.looped, Some(true));
        assert_eq!(f.steps.len(), 8);
        assert_eq!(f.steps[0], Step::Click { button: 0, pos: Some((500, 600)) });
        assert!(matches!(f.steps[2], Step::IfColor { pos: (700, 300), rgb: 0x336699, else_goto: 1, .. }));
    }

    /// Simuliert den Ablauf des Beispiel-Makros: erst wird geklickt, bis die Farbe da ist,
    /// dann werden die beiden Buttons gedrueckt.
    #[test]
    fn example_flow() {
        let steps = parse_file(BEISPIEL).steps;
        let mut clicks = Vec::new();
        let mut i = 0;
        let mut checks = 0;
        while i < steps.len() {
            let flow = match steps[i] {
                Step::Click { pos, .. } => {
                    clicks.push(pos.unwrap());
                    Flow::Next
                }
                Step::IfColor { rgb, tol, else_goto, .. } => {
                    checks += 1;
                    // erst bei der 4. Pruefung ist die Farbe da
                    let seen = if checks > 3 { 0x336699 } else { 0x101010 };
                    if color_matches(seen, rgb, tol) { Flow::Next } else { Flow::Jump(else_goto) }
                }
                _ => Flow::Next,
            };
            i = advance(i, flow);
        }
        assert_eq!(clicks, vec![(500, 600), (500, 600), (500, 600), (500, 600), (900, 200), (650, 650)]);
    }

    #[test]
    fn colors() {
        assert!(color_matches(0x34477A, 0x34477A, 0));
        assert!(color_matches(0x3A4070, 0x34477A, 10));
        assert!(!color_matches(0x34477A, 0x34477B + 0x0B, 10));
        assert_eq!(parse_hex("#34477a"), Some(0x34477A));
        assert_eq!(parse_hex("Hex #34477A"), Some(0x34477A));
        assert_eq!(parse_hex("34477"), None);
    }

    #[test]
    fn advance_rules() {
        assert_eq!(advance(2, Flow::Next), 3);
        assert_eq!(advance(2, Flow::Jump(0)), 3);
        assert_eq!(advance(5, Flow::Jump(1)), 0);
        assert_eq!(advance(0, Flow::Jump(99)), 98);
    }

    #[test]
    fn renumbering() {
        let mut s = vec![Step::Goto(1), Step::Goto(2), Step::Goto(3)];
        renumber_after_insert(&mut s, 1); // neuer Schritt wird zu Nr. 2
        assert_eq!(s, vec![Step::Goto(1), Step::Goto(3), Step::Goto(4)]);
        let mut s = vec![Step::Goto(1), Step::Goto(3), Step::Goto(4)];
        renumber_after_remove(&mut s, 1); // Nr. 2 entfernt
        assert_eq!(s, vec![Step::Goto(1), Step::Goto(2), Step::Goto(3)]);
        let mut s = vec![Step::Goto(1), Step::Goto(2)];
        renumber_after_swap(&mut s, 0, 1);
        assert_eq!(s, vec![Step::Goto(2), Step::Goto(1)]);
    }
}
