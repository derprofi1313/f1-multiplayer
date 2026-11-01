use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum Vdf {
    Str(String),
    Map(BTreeMap<String, Vdf>),
}

impl Vdf {
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Map(m) => m
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Str(_) => None,
        }
    }

    pub fn path(&self, keys: &[&str]) -> Option<&Vdf> {
        keys.iter().try_fold(self, |v, k| v.get(k))
    }

    pub fn str(&self) -> Option<&str> {
        match self {
            Vdf::Str(s) => Some(s),
            Vdf::Map(_) => None,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = (&String, &Vdf)> {
        match self {
            Vdf::Map(m) => Some(m.iter()),
            Vdf::Str(_) => None,
        }
        .into_iter()
        .flatten()
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Str(String),
    Open,
    Close,
}

fn tokens(src: &str) -> Vec<Token> {
    let mut out = Vec::new();
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '{' => out.push(Token::Open),
            '}' => out.push(Token::Close),
            '/' if chars.peek() == Some(&'/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        break;
                    }
                }
            }
            '"' => {
                let mut s = String::new();
                while let Some(c) = chars.next() {
                    match c {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(o) => s.push(o),
                            None => {}
                        },
                        o => s.push(o),
                    }
                }
                out.push(Token::Str(s));
            }
            c if c.is_whitespace() => {}
            c => {
                let mut s = String::from(c);
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() || n == '{' || n == '}' || n == '"' {
                        break;
                    }
                    s.push(n);
                    chars.next();
                }
                out.push(Token::Str(s));
            }
        }
    }
    out
}

pub fn parse(src: &str) -> Option<Vdf> {
    let toks = tokens(src);
    let mut pos = 0;
    let root = map(&toks, &mut pos)?;
    Some(Vdf::Map(root))
}

fn map(toks: &[Token], pos: &mut usize) -> Option<BTreeMap<String, Vdf>> {
    let mut m = BTreeMap::new();
    while *pos < toks.len() {
        match &toks[*pos] {
            Token::Close => {
                *pos += 1;
                return Some(m);
            }
            Token::Str(k) => {
                *pos += 1;
                match toks.get(*pos)? {
                    Token::Open => {
                        *pos += 1;
                        m.insert(k.clone(), Vdf::Map(map(toks, pos)?));
                    }
                    Token::Str(v) => {
                        *pos += 1;
                        m.insert(k.clone(), Vdf::Str(v.clone()));
                    }
                    Token::Close => return None,
                }
            }
            Token::Open => return None,
        }
    }
    Some(m)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_libraryfolders() {
        let v = parse(
            r#""libraryfolders"
            {
                "0" { "path" "/home/u/.local/share/Steam" "apps" { "228980" "1" } }
                // comment
                "1" { "path" "/mnt/games/SteamLibrary" "apps" { "3059520" "1234" } }
            }"#,
        )
        .unwrap();
        let lf = v.get("libraryfolders").unwrap();
        let paths: Vec<&str> = lf.entries().filter_map(|(_, e)| e.get("path")?.str()).collect();
        assert_eq!(paths, ["/home/u/.local/share/Steam", "/mnt/games/SteamLibrary"]);
        assert_eq!(
            lf.path(&["1", "Apps", "3059520"]).and_then(Vdf::str),
            Some("1234")
        );
    }

    #[test]
    fn handles_escapes() {
        let v = parse(r#""a" { "LaunchOptions" "\"x\" %command%" }"#).unwrap();
        assert_eq!(
            v.path(&["a", "launchoptions"]).and_then(Vdf::str),
            Some("\"x\" %command%")
        );
    }
}
