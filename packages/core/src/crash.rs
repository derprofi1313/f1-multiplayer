use std::collections::BTreeMap;
use std::path::Path;

use crate::{GAME_EXE, IMAGE_BASE, Result, io_err};

const IMAGE_END: u64 = 0x1_5a00_0000;

pub const FATAL: &[(u32, &str)] = &[
    (0xc000_0005, "access violation"),
    (0xc000_001d, "illegal instruction"),
    (0xc000_0094, "integer divide by zero"),
    (0xc000_00fd, "stack overflow"),
    (0xc000_0409, "stack buffer overrun"),
    (0xc000_0374, "heap corruption"),
    (0xc000_0096, "privileged instruction"),
    (0xc000_0025, "noncontinuable exception"),
    (0xc000_0420, "assertion failure"),
    (0x8000_0003, "breakpoint"),
];

pub fn fatal_name(code: u32) -> Option<&'static str> {
    FATAL.iter().find(|(c, _)| *c == code).map(|(_, n)| *n)
}

const NOISE: &[&str] = &[
    "xinput", "dinput", "hid", "winediag", "ntoskrnl", "setupapi", "wbemprox", "rawinput",
];

#[derive(Debug, Clone, Default)]
pub struct Symbols {
    names: BTreeMap<u64, String>,
    bounds: Vec<(u64, u64)>,
}

impl Symbols {
    pub fn parse_kb(tsv: &str) -> Self {
        let names = tsv
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut cols = line.split('\t');
                let addr = u64::from_str_radix(cols.next()?.strip_prefix("0x")?, 16).ok()?;
                Some((addr, cols.next()?.to_owned()))
            })
            .collect();
        Self {
            names,
            bounds: Vec::new(),
        }
    }

    pub fn load_kb(tsv: &Path) -> Result<Self> {
        Ok(Self::parse_kb(
            &std::fs::read_to_string(tsv).map_err(io_err(tsv))?,
        ))
    }

    pub fn load_pdata(&mut self, exe: &Path) -> Result<()> {
        use std::io::{Read, Seek, SeekFrom};
        let mut f = std::fs::File::open(exe).map_err(io_err(exe))?;
        let mut header = vec![0; 4096];
        f.read_exact(&mut header).map_err(io_err(exe))?;
        let missing = || crate::Error::Other(format!("{}: no exception directory", exe.display()));
        let (offset, size) = pdata_range(&header).ok_or_else(missing)?;
        let mut table = vec![0; size];
        f.seek(SeekFrom::Start(offset as u64)).map_err(io_err(exe))?;
        f.read_exact(&mut table).map_err(io_err(exe))?;
        self.bounds = parse_pdata(&table);
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
    }

    pub fn describe_exe(&self, va: u64) -> String {
        let rva = va - IMAGE_BASE;
        if !self.bounds.is_empty() {
            let i = self.bounds.partition_point(|(s, _)| *s <= va);
            return match i.checked_sub(1).map(|i| self.bounds[i]) {
                Some((start, end)) if va < end => {
                    let name = self
                        .names
                        .get(&start)
                        .cloned()
                        .unwrap_or_else(|| format!("FUN_{start:x}"));
                    format!("{GAME_EXE}+{rva:#x} in {name}+{:#x}", va - start)
                }
                _ => format!("{GAME_EXE}+{rva:#x} (no function bounds, likely Denuvo .data1)"),
            };
        }
        match self.names.range(..=va).next_back() {
            Some((start, name)) if va - start < 0x2_0000 => {
                format!("{GAME_EXE}+{rva:#x} near {name}+{:#x}", va - start)
            }
            _ => format!("{GAME_EXE}+{rva:#x}"),
        }
    }
}

fn pdata_range(pe: &[u8]) -> Option<(usize, usize)> {
    let u16_at = |o: usize| Some(u16::from_le_bytes(pe.get(o..o + 2)?.try_into().ok()?));
    let u32_at = |o: usize| Some(u32::from_le_bytes(pe.get(o..o + 4)?.try_into().ok()?));
    let nt = u32_at(0x3c)? as usize;
    let sections = u16_at(nt + 6)? as usize;
    let opt = nt + 24;
    let (rva, size) = (u32_at(opt + 136)?, u32_at(opt + 140)?);
    let table = opt + u16_at(nt + 20)? as usize;
    (0..sections).find_map(|i| {
        let s = table + 40 * i;
        let (vsize, va, raw) = (u32_at(s + 8)?, u32_at(s + 12)?, u32_at(s + 20)?);
        (va <= rva && rva < va + vsize && size > 0).then(|| ((rva - va + raw) as usize, size as usize))
    })
}

fn parse_pdata(table: &[u8]) -> Vec<(u64, u64)> {
    let mut out: Vec<(u64, u64)> = table
        .chunks_exact(12)
        .filter_map(|e| {
            let start = u32::from_le_bytes(e[0..4].try_into().ok()?);
            let end = u32::from_le_bytes(e[4..8].try_into().ok()?);
            (start != 0).then_some((IMAGE_BASE + start as u64, IMAGE_BASE + end as u64))
        })
        .collect();
    out.sort_unstable();
    out
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Module { base: u64, name: String },
    Fatal { code: u32, addr: u64, first: bool },
    Crash(String),
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartFailure {
    pub code: u32,
}

impl StartFailure {
    pub fn parse(line: &str) -> Option<Self> {
        let rest = &line[line.find("err:steam:run_process Failed to create process")?..];
        let code = rest.trim_end().rsplit_once(": ")?.1.parse().ok()?;
        Some(Self { code })
    }
}

impl std::fmt::Display for StartFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = self.code;
        write!(
            f,
            "Proton could not start the game process (CreateProcess error {code}). "
        )?;
        if (731..=733).contains(&code) {
            let status = code - 730;
            write!(
                f,
                "Wine reports the new process's own exit status ({status}) as ERROR_WAIT_{status}, \
                 meaning {GAME_EXE} exited during startup, before any DLL loaded. "
            )?;
        }
        write!(f, "Nothing from SwoftyMod ran.")
    }
}

#[derive(Debug, Clone, Default)]
pub struct LogScan {
    pub modules: Vec<(u64, String)>,
    pub exceptions: BTreeMap<(u32, u64), usize>,
    pub last_fatal: Option<(u32, u64)>,
    pub errors: BTreeMap<String, usize>,
    pub start_failure: Option<StartFailure>,
}

fn hex_after(line: &str, marker: &str) -> Option<u64> {
    let rest = &line[line.find(marker)? + marker.len()..];
    let end = rest.find(|c: char| !c.is_ascii_hexdigit()).unwrap_or(rest.len());
    u64::from_str_radix(&rest[..end], 16).ok()
}

fn strip_prefix(line: &str) -> &str {
    let s = line
        .trim()
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '.' || c == ':');
    let s = s.trim_start();
    match s.get(..10) {
        Some(p)
            if p.as_bytes()[4] == b':'
                && p.as_bytes()[9] == b':'
                && p.bytes().filter(|b| *b != b':').all(|b| b.is_ascii_hexdigit()) =>
        {
            &s[10..]
        }
        _ => s,
    }
}

impl LogScan {
    pub fn feed(&mut self, line: &str) -> Option<Event> {
        if let Some(i) = line.find("Loaded L\"") {
            let rest = &line[i + 9..];
            let path = rest.split('"').next()?;
            let base = hex_after(rest, "\" at ")?;
            let name = path
                .replace("\\\\", "\\")
                .rsplit('\\')
                .next()
                .unwrap_or(path)
                .to_owned();
            self.modules.push((base, name.clone()));
            self.modules.sort();
            return Some(Event::Module { base, name });
        }
        if line.contains("dispatch_exception code=") {
            let code = hex_after(line, "code=")? as u32;
            let addr = hex_after(line, "addr=")?;
            fatal_name(code)?;
            let n = self.exceptions.entry((code, addr)).or_default();
            *n += 1;
            self.last_fatal = Some((code, addr));
            return Some(Event::Fatal {
                code,
                addr,
                first: *n == 1,
            });
        }
        if line.contains("Unhandled") || line.contains("Backtrace") || line.contains("starting debugger") {
            return Some(Event::Crash(line.trim().to_owned()));
        }
        if self.start_failure.is_none() {
            self.start_failure = StartFailure::parse(line);
        }
        if line.contains("err:") {
            let text = strip_prefix(line).to_owned();
            if NOISE.iter().any(|n| text.contains(&format!("err:{n}"))) {
                return None;
            }
            let n = self.errors.entry(text.clone()).or_default();
            *n += 1;
            return (*n == 1 && self.errors.len() <= 200).then_some(Event::Error(text));
        }
        None
    }

    pub fn describe(&self, va: u64, symbols: &Symbols) -> String {
        if (IMAGE_BASE..IMAGE_END).contains(&va) {
            return symbols.describe_exe(va);
        }
        match self.modules.iter().rev().find(|(b, _)| *b <= va) {
            Some((base, name)) => format!("{name}+{:#x}", va - base),
            None => format!("{va:#x}"),
        }
    }

    pub fn summary(&self, symbols: &Symbols) -> Vec<String> {
        if self.exceptions.is_empty() {
            return vec!["no fatal-class exceptions in the wine log".into()];
        }
        let mut out = vec![format!(
            "{} distinct fatal-class exception(s):",
            self.exceptions.len()
        )];
        let mut rows: Vec<_> = self.exceptions.iter().collect();
        rows.sort_by_key(|(_, n)| std::cmp::Reverse(**n));
        for (&(code, addr), n) in rows.into_iter().take(15) {
            let mark = if self.last_fatal == Some((code, addr)) {
                "  <- last"
            } else {
                ""
            };
            let name = fatal_name(code).unwrap_or("?");
            out.push(format!(
                "  x{n:<5} {name:<24} {}{mark}",
                self.describe(addr, symbols)
            ));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TSV: &str = "addr\tname\tsig\tlevel\n0x14173df80\tFeTile_Build_List\t-\tL2\n";

    #[test]
    fn names_exe_addresses_from_the_kb() {
        let sym = Symbols::parse_kb(TSV);
        assert_eq!(sym.len(), 1);
        assert_eq!(
            sym.describe_exe(0x14173dfa0),
            "F1_25.exe+0x173dfa0 near FeTile_Build_List+0x20"
        );
    }

    #[test]
    fn uses_pdata_bounds_when_loaded() {
        let mut sym = Symbols::parse_kb(TSV);
        sym.bounds = vec![(0x14173df80, 0x14173e100)];
        assert_eq!(
            sym.describe_exe(0x14173dfa0),
            "F1_25.exe+0x173dfa0 in FeTile_Build_List+0x20"
        );
        assert!(sym.describe_exe(0x150000000).contains("no function bounds"));
    }

    #[test]
    fn scans_wine_log() {
        let mut scan = LogScan::default();
        let sym = Symbols::parse_kb(TSV);
        let ev = scan.feed(r#"1.234:0024:0028:trace:loaddll:build_module Loaded L"C:\\windows\\system32\\version.dll" at 00006FFFFB540000: native"#);
        assert_eq!(
            ev,
            Some(Event::Module {
                base: 0x6ffffb540000,
                name: "version.dll".into()
            })
        );
        let exc = "1.300:0024:0028:trace:seh:dispatch_exception code=c0000005 (EXCEPTION_ACCESS_VIOLATION) flags=0 addr=000000014173DFA0 ip=000000014173DFA0 tid=0028";
        assert!(matches!(scan.feed(exc), Some(Event::Fatal { first: true, .. })));
        assert!(matches!(scan.feed(exc), Some(Event::Fatal { first: false, .. })));
        assert_eq!(
            scan.feed(
                "1.3:0024:0028:trace:seh:dispatch_exception code=406d1388 flags=0 addr=0000000000001234"
            ),
            None
        );
        scan.feed("1.301:0024:0028:trace:seh:dispatch_exception code=c0000005 flags=0 addr=00006FFFFB541000 ip=0 tid=0028");
        assert!(matches!(
            scan.feed("1.4:0024:0028:err:module:import_dll Library foo.dll not found"),
            Some(Event::Error(_))
        ));
        assert_eq!(scan.feed("1.4:0024:0028:err:xinput:x"), None);
        let s = scan.summary(&sym);
        assert_eq!(s.len(), 3);
        assert!(
            s[1].contains("x2") && s[1].contains("F1_25.exe+0x173dfa0 near FeTile_Build_List+0x20"),
            "{}",
            s[1]
        );
        assert!(
            s[2].contains("version.dll+0x1000") && s[2].ends_with("<- last"),
            "{}",
            s[2]
        );
        assert_eq!(
            scan.errors.keys().next().unwrap(),
            "err:module:import_dll Library foo.dll not found"
        );
        assert_eq!(scan.start_failure, None);
    }

    #[test]
    fn finds_a_process_proton_could_not_start() {
        let mut scan = LogScan::default();
        for line in include_str!("../tests/fixtures/steam-run-process-731.txt").lines() {
            scan.feed(line);
        }
        let failure = scan.start_failure.expect("run_process failure");
        assert_eq!(failure, StartFailure { code: 731 });
        assert_eq!(
            failure.to_string(),
            "Proton could not start the game process (CreateProcess error 731). \
             Wine reports the new process's own exit status (1) as ERROR_WAIT_1, \
             meaning F1_25.exe exited during startup, before any DLL loaded. Nothing from SwoftyMod ran."
        );
        assert_eq!(
            StartFailure { code: 2 }.to_string(),
            "Proton could not start the game process (CreateProcess error 2). Nothing from SwoftyMod ran."
        );
        assert_eq!(
            StartFailure::parse(
                "0.1:0020:0024:err:steam:run_process Failed to create process L\"x.exe\": 2\r"
            ),
            Some(StartFailure { code: 2 })
        );
        assert_eq!(
            StartFailure::parse("err:steam:run_process Failed to create process"),
            None
        );
    }
}
