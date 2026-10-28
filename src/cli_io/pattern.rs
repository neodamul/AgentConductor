//! The regular-expression subset the acio schemas use, so no regex dependency is needed: `^` and `$` at the
//! ends, literals, `.`, classes (`[a-z0-9_-]`, `[^…]`), groups, `|`, and `?`, `*`, `+`, `{n}`, `{n,}`, `{n,m}`.

enum Node {
    Char(char),
    Any,
    Class(Vec<(char, char)>, bool),
    Group(Vec<Vec<Node>>),
    Repeat(Box<Node>, usize, usize),
}

pub(super) struct Pattern {
    alts: Vec<Vec<Node>>,
    start: bool,
    end: bool,
}

struct Parser {
    chars: Vec<char>,
    at: usize,
}

impl Parser {
    fn alternatives(&mut self) -> Result<Vec<Vec<Node>>, String> {
        let mut alts = vec![self.sequence()?];
        while self.chars.get(self.at) == Some(&'|') {
            self.at += 1;
            alts.push(self.sequence()?);
        }
        Ok(alts)
    }
    fn sequence(&mut self) -> Result<Vec<Node>, String> {
        let mut seq = vec![];
        while let Some(&c) = self.chars.get(self.at) {
            if c == '|' || c == ')' {
                break;
            }
            self.at += 1;
            let atom = match c {
                '(' => {
                    if self.chars[self.at..].starts_with(&['?', ':']) {
                        self.at += 2;
                    }
                    let alts = self.alternatives()?;
                    if self.chars.get(self.at) != Some(&')') {
                        return Err("unclosed group".into());
                    }
                    self.at += 1;
                    Node::Group(alts)
                }
                '[' => self.class()?,
                '.' => Node::Any,
                '\\' => {
                    let escaped = *self.chars.get(self.at).ok_or("dangling escape")?;
                    self.at += 1;
                    Node::Char(escaped)
                }
                '*' | '+' | '?' | '{' => return Err(format!("nothing to repeat before {c}")),
                c => Node::Char(c),
            };
            seq.push(self.repeat(atom)?);
        }
        Ok(seq)
    }
    fn class(&mut self) -> Result<Node, String> {
        let negate = self.chars.get(self.at) == Some(&'^');
        self.at += usize::from(negate);
        let mut set = vec![];
        loop {
            let c = *self.chars.get(self.at).ok_or("unclosed class")?;
            self.at += 1;
            if c == ']' && !set.is_empty() {
                return Ok(Node::Class(set, negate));
            }
            let c = if c == '\\' {
                self.at += 1;
                *self.chars.get(self.at - 1).ok_or("dangling escape")?
            } else {
                c
            };
            if self.chars.get(self.at) == Some(&'-')
                && self.chars.get(self.at + 1).is_some_and(|n| *n != ']')
            {
                let hi = self.chars[self.at + 1];
                self.at += 2;
                set.push((c, hi));
            } else {
                set.push((c, c));
            }
        }
    }
    fn repeat(&mut self, atom: Node) -> Result<Node, String> {
        let (min, max) = match self.chars.get(self.at) {
            Some('?') => (0, 1),
            Some('*') => (0, usize::MAX),
            Some('+') => (1, usize::MAX),
            Some('{') => {
                let close = self.chars[self.at..]
                    .iter()
                    .position(|c| *c == '}')
                    .ok_or("unclosed {")?
                    + self.at;
                let body: String = self.chars[self.at + 1..close].iter().collect();
                let bound = |s: &str| {
                    s.parse::<usize>()
                        .map_err(|_| format!("bad repeat {{{body}}}"))
                };
                let range = match body.split_once(',') {
                    None => (bound(&body)?, bound(&body)?),
                    Some((lo, "")) => (bound(lo)?, usize::MAX),
                    Some((lo, hi)) => (bound(lo)?, bound(hi)?),
                };
                self.at = close;
                range
            }
            _ => return Ok(atom),
        };
        self.at += 1;
        Ok(Node::Repeat(Box::new(atom), min, max))
    }
}

fn ends(seq: &[Node], text: &[char], from: usize) -> Vec<usize> {
    let mut cur = vec![from];
    for node in seq {
        let mut next: Vec<usize> = cur.iter().flat_map(|&p| node_ends(node, text, p)).collect();
        next.sort_unstable();
        next.dedup();
        if next.is_empty() {
            return next;
        }
        cur = next;
    }
    cur
}

fn node_ends(node: &Node, text: &[char], at: usize) -> Vec<usize> {
    let one = |ok: bool| if ok { vec![at + 1] } else { vec![] };
    match node {
        Node::Char(c) => one(text.get(at) == Some(c)),
        Node::Any => one(at < text.len()),
        Node::Class(set, negate) => one(text
            .get(at)
            .is_some_and(|c| set.iter().any(|(lo, hi)| lo <= c && c <= hi) != *negate)),
        Node::Group(alts) => {
            let mut all: Vec<usize> = alts.iter().flat_map(|a| ends(a, text, at)).collect();
            all.sort_unstable();
            all.dedup();
            all
        }
        Node::Repeat(inner, min, max) => {
            let mut all = if *min == 0 { vec![at] } else { vec![] };
            let mut cur = vec![at];
            for count in 1..=(*max).min(text.len() + 1) {
                let mut next: Vec<usize> = cur
                    .iter()
                    .flat_map(|&p| node_ends(inner, text, p))
                    .collect();
                next.sort_unstable();
                next.dedup();
                if next.is_empty() {
                    break;
                }
                if count >= *min {
                    all.extend(&next);
                }
                cur = next;
            }
            all.sort_unstable();
            all.dedup();
            all
        }
    }
}

impl Pattern {
    pub(super) fn new(source: &str) -> Result<Self, String> {
        let start = source.starts_with('^');
        let end = source.ends_with('$') && !source.ends_with("\\$");
        let body = &source[usize::from(start)..source.len() - usize::from(end)];
        let mut parser = Parser {
            chars: body.chars().collect(),
            at: 0,
        };
        let alts = parser.alternatives()?;
        if parser.at != parser.chars.len() {
            return Err(format!("unexpected character at {} in {source}", parser.at));
        }
        Ok(Self { alts, start, end })
    }
    pub(super) fn is_match(&self, text: &str) -> bool {
        let chars: Vec<char> = text.chars().collect();
        let starts: Vec<usize> = if self.start {
            vec![0]
        } else {
            (0..=chars.len()).collect()
        };
        starts.into_iter().any(|from| {
            let found: Vec<usize> = self
                .alts
                .iter()
                .flat_map(|a| ends(a, &chars, from))
                .collect();
            if self.end {
                found.contains(&chars.len())
            } else {
                !found.is_empty()
            }
        })
    }
}
