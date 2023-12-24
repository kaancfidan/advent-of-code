use std::collections::{HashMap, VecDeque};
use std::fmt::{Display, Formatter};
use std::io;
use std::io::{BufRead, BufReader, Read};

pub struct System {
    messages: VecDeque<Message>,
    modules: HashMap<String, Module>,
}

pub struct Message {
    pub src: String,
    pub dst: String,
    pub pulse: bool,
}

#[derive(Clone)]
enum Module {
    Broadcaster(Broadcaster),
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
    Sink(String),
}

#[derive(Clone)]
struct FlipFlop {
    name: String,
    dst: Vec<String>,
    state: bool,
}

#[derive(Clone)]
struct Conjunction {
    name: String,
    dst: Vec<String>,
    inputs: HashMap<String, bool>,
}

#[derive(Clone)]
struct Broadcaster {
    dst: Vec<String>,
}

trait Communicate {
    fn name(&self) -> String;

    fn receive(&mut self, src: &str, pulse: bool) -> Vec<Message>;
    fn destinations(&self) -> Vec<String>;

    fn send(&mut self, pulse: bool) -> Vec<Message> {
        self.destinations()
            .into_iter()
            .map(|dst| Message {
                src: self.name(),
                dst,
                pulse,
            })
            .collect()
    }
}

impl System {
    pub fn push_button(&mut self) -> Vec<Message> {
        let msg = Message {
            src: "button".to_string(),
            dst: "broadcaster".to_string(),
            pulse: false,
        };

        self.messages.push_back(msg);
        self.run()
    }

    pub fn from_stream(r: impl Read) -> Result<Self, ParseError> {
        let reader = BufReader::new(r);
        let mut broadcaster_defined = false;

        let mut modules: HashMap<String, Module> = reader
            .lines()
            .map(|l| {
                let line = l.map_err(ParseError::IO)?;

                let split: Vec<_> = line.split("->").map(|s| s.trim().to_owned()).collect();
                let first = split[0].to_owned();
                let dst = split[1].split(',').map(|s| s.trim().to_owned()).collect();

                match first.as_str() {
                    "broadcaster" if !broadcaster_defined => {
                        broadcaster_defined = true;
                        Ok((first, Module::Broadcaster(Broadcaster { dst })))
                    }
                    s if s.starts_with('%') => {
                        let name = s[1..].to_owned();
                        Ok((
                            name.clone(),
                            Module::FlipFlop(FlipFlop {
                                name,
                                state: false,
                                dst,
                            }),
                        ))
                    }
                    s if s.starts_with('&') => {
                        let name = s[1..].to_owned();
                        Ok((
                            name.clone(),
                            Module::Conjunction(Conjunction {
                                name,
                                inputs: HashMap::new(),
                                dst,
                            }),
                        ))
                    }
                    "broadcaster" if broadcaster_defined => Err(ParseError::MultipleBroadcasters),
                    _ => Err(ParseError::UnknownModule(first.to_owned())),
                }
            })
            .collect::<Result<_, _>>()?;

        if modules.get("broadcaster").is_none() {
            return Err(ParseError::NoBroadcaster);
        }

        let ms: Vec<Module> = modules.values().cloned().collect();

        for m in ms {
            for d in m.destinations() {
                match modules.get_mut(&d) {
                    Some(Module::Conjunction(ref mut c)) => {
                        c.inputs.insert(m.name(), false);
                    }
                    Some(_) => {}
                    None => {
                        modules.insert(d.clone(), Module::Sink(d.clone()));
                    }
                }
            }
        }

        Ok(System {
            modules,
            messages: VecDeque::new(),
        })
    }

    pub fn reset(&mut self) {
        for m in self.modules.values_mut() {
            match m {
                Module::FlipFlop(f) => {
                    f.state = false;
                }
                Module::Conjunction(c) => {
                    let sources: Vec<_> = c.inputs.keys().cloned().collect();
                    for src in sources {
                        c.inputs.insert(src.clone(), false);
                    }
                }
                _ => {}
            }
        }
    }

    fn run(&mut self) -> Vec<Message> {
        let mut messages: Vec<Message> = Vec::new();

        while let Some(msg) = self.messages.pop_front() {
            let dst = self.modules.get_mut(&msg.dst).unwrap();

            let new_messages = dst.receive(&msg.src, msg.pulse);

            self.messages.extend(new_messages.into_iter());
            messages.push(msg);
        }

        messages
    }
}

impl Communicate for Module {
    fn name(&self) -> String {
        match self {
            Module::Broadcaster(_) => "broadcaster".to_owned(),
            Module::FlipFlop(f) => f.name.clone(),
            Module::Conjunction(c) => c.name.clone(),
            Module::Sink(n) => n.clone(),
        }
    }

    fn receive(&mut self, src: &str, pulse: bool) -> Vec<Message> {
        let reaction = match self {
            Module::Broadcaster(_) => Some(pulse),
            Module::FlipFlop(f) => {
                if !pulse {
                    f.state = !f.state;
                    Some(f.state)
                } else {
                    None
                }
            }
            Module::Conjunction(c) => {
                c.inputs.insert(src.to_owned(), pulse);

                let all_high = c.inputs.values().all(|p| *p);
                Some(!all_high)
            }
            Module::Sink(_) => None,
        };

        if let Some(r) = reaction {
            self.send(r)
        } else {
            vec![]
        }
    }

    fn destinations(&self) -> Vec<String> {
        match self {
            Module::Broadcaster(b) => b.dst.clone(),
            Module::FlipFlop(f) => f.dst.clone(),
            Module::Conjunction(c) => c.dst.clone(),
            Module::Sink(_) => vec![],
        }
    }
}

#[derive(Debug)]
pub enum ParseError {
    IO(io::Error),
    NoBroadcaster,
    MultipleBroadcasters,
    UnknownModule(String),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::IO(e) => write!(f, "Could not read file: {e}"),
            ParseError::NoBroadcaster => write!(f, "No broadcaster definition given."),
            ParseError::MultipleBroadcasters => {
                write!(f, "Multiple broadcaster definitions given.")
            }
            ParseError::UnknownModule(s) => write!(f, "Unknown module type {s} was given."),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stringreader::StringReader;

    #[test]
    fn example1() {
        let input = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";

        let r = StringReader::new(input);
        let mut system = System::from_stream(r).unwrap();

        let messages = system.push_button();
        let len = messages.len();

        assert_eq!(len, 12);
    }

    #[test]
    fn example1_1000() {
        let input = "broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";

        let r = StringReader::new(input);
        let mut system = System::from_stream(r).unwrap();

        let messages: Vec<Message> = (0..1000).flat_map(|_| system.push_button()).collect();

        let low_count = messages.iter().filter(|m| !m.pulse).count();
        let high_count = messages.iter().filter(|m| m.pulse).count();

        assert_eq!(low_count, 8000);
        assert_eq!(high_count, 4000);
    }

    #[test]
    fn example2() {
        let input = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

        let r = StringReader::new(input);
        let mut system = System::from_stream(r).unwrap();

        let messages = system.push_button();

        assert_eq!(messages.len(), 8);

        let messages = system.push_button();

        assert_eq!(messages.len(), 6);

        let messages = system.push_button();

        assert_eq!(messages.len(), 8);

        let messages = system.push_button();

        assert_eq!(messages.len(), 6);
    }

    #[test]
    fn example2_1000() {
        let input = "broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

        let r = StringReader::new(input);
        let mut system = System::from_stream(r).unwrap();

        let messages: Vec<Message> = (0..1000).flat_map(|_| system.push_button()).collect();

        let low_count = messages.iter().filter(|m| !m.pulse).count();
        let high_count = messages.iter().filter(|m| m.pulse).count();

        assert_eq!(low_count, 4250);
        assert_eq!(high_count, 2750);
    }
}
