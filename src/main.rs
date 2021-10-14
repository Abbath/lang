use clap::{App, Arg};
use regex::Regex;
use std::collections::{HashMap, VecDeque};

macro_rules! collection {
    // map-like
    ($($k:expr => $v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$(($k, $v),)*]))
    }};
    // set-like
    ($($v:expr),* $(,)?) => {{
        use std::iter::{Iterator, IntoIterator};
        Iterator::collect(IntoIterator::into_iter([$($v,)*]))
    }};
}

#[derive(Debug)]
enum Token {
    Word(String),
    Number(i64),
}

#[derive(Clone, Copy, Debug)]
enum Intrinsic {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    LT,
    GT,
    LE,
    GE,
    EQ,
    NE,
    Dup,
    Drop,
}

#[derive(Clone, Copy, Debug)]
enum Op {
    Push(i64),
    Int(Intrinsic),
    Cond,
    BStart(usize, usize),
    BElse(usize, usize),
    BEnd(usize),
}

fn lex_token(tok: &str) -> Token {
    let re = Regex::new(r"^-?\d{1,10}$").unwrap();
    if re.is_match(&tok) {
        Token::Number(tok.parse::<i64>().unwrap())
    } else {
        Token::Word(tok.to_string())
    }
}

fn lex(input: &str) -> VecDeque<Token> {
    let mut res: VecDeque<Token> = VecDeque::new();
    let mut current_token = String::new();
    input.chars().for_each(|c| {
        if !c.is_whitespace() {
            current_token.push(c);
        } else {
            res.push_back(lex_token(&current_token));
            current_token = String::new();
        }
    });
    if !current_token.is_empty() {
        res.push_back(lex_token(&current_token));
    }
    res
}

fn parse(input: &VecDeque<Token>) -> VecDeque<Op> {
    let mut res = VecDeque::<Op>::new();
    let mut idx = 0usize;
    let ops: HashMap<String, Op> = collection! {
        "+".to_string() => Op::Int(Intrinsic::Add),
        "-".to_string() => Op::Int(Intrinsic::Sub),
        "*".to_string() => Op::Int(Intrinsic::Mult),
        "/".to_string() => Op::Int(Intrinsic::Div),
        "%".to_string() => Op::Int(Intrinsic::Mod),
        "<".to_string() => Op::Int(Intrinsic::LT),
        ">".to_string() => Op::Int(Intrinsic::GT),
        "<=".to_string() => Op::Int(Intrinsic::LE),
        ">=".to_string() => Op::Int(Intrinsic::GE),
        "==".to_string() => Op::Int(Intrinsic::EQ),
        "!=".to_string() => Op::Int(Intrinsic::NE),
        ":".to_string() => Op::Int(Intrinsic::Dup),
        ";".to_string() => Op::Int(Intrinsic::Drop),
        "?".to_string() => Op::Cond,
        "{".to_string() => Op::BStart(0, 0),
        "}{".to_string() => Op::BElse(0, 0),
        "}".to_string() => Op::BEnd(0)
    };
    let mut stack = VecDeque::<usize>::new();
    for tok in input.iter() {
        match tok {
            Token::Number(n) => {
                res.push_back(Op::Push(*n));
                idx += 1;
            }
            Token::Word(w) => {
                let op = ops.get(w).unwrap();
                match op {
                    Op::BStart(_, _) => {
                        stack.push_back(idx);
                        res.push_back(*op)
                    }
                    Op::BElse(_, _) => {
                        let bi = stack.pop_back().unwrap();
                        res[bi] = Op::BStart(idx, 0);
                        stack.push_back(idx);
                        res.push_back(Op::BElse(bi, 0))
                    }
                    Op::BEnd(_) => {
                        let bi = stack.pop_back().unwrap();
                        if let Op::BElse(o, _) = res[bi] {
                            res[bi] = Op::BElse(o, idx);
                            res[o] = Op::BStart(bi, idx);
                            res.push_back(Op::BEnd(bi))
                        }
                        if let Op::BStart(o, _) = res[bi] {
                            res[bi] = Op::BStart(bi, idx);
                            res.push_back(Op::BEnd(bi))
                        }
                    }
                    _ => res.push_back(*op),
                }
                idx += 1;
            }
        }
    }
    res
}

fn compute(tokens: VecDeque<Token>) -> Result<VecDeque<i64>, String> {
    let mut stack = VecDeque::<i64>::new();
    let math_ops = vec!["+", "-", "*", "/", "%", "<", ">", "<=", ">=", "==", "!="];
    let stack_ops = vec![":", ";"];
    for tok in tokens.iter() {
        match tok {
            Token::Number(n) => stack.push_back(*n),
            Token::Word(w) => {
                if math_ops.contains(&w.as_str()) {
                    if stack.len() < 2 {
                        return Err("We fucked up!".to_string());
                    }
                    let a = stack.pop_back().unwrap();
                    let b = stack.pop_back().unwrap();
                    stack.push_back(match w.as_str() {
                        "+" => a + b,
                        "-" => b - a,
                        "*" => a * b,
                        "/" => b / a,
                        "%" => b % a,
                        "<" => {
                            if b < a {
                                1
                            } else {
                                0
                            }
                        }
                        ">" => {
                            if b > a {
                                1
                            } else {
                                0
                            }
                        }
                        "<=" => {
                            if b <= a {
                                1
                            } else {
                                0
                            }
                        }
                        ">=" => {
                            if b >= a {
                                1
                            } else {
                                0
                            }
                        }
                        "!=" => {
                            if b != a {
                                1
                            } else {
                                0
                            }
                        }
                        "==" => {
                            if b == a {
                                1
                            } else {
                                0
                            }
                        }
                        _ => return Err("Unknown op".to_string()),
                    });
                }
                if stack_ops.contains(&w.as_str()) {
                    if stack.len() < 1 {
                        return Err("We fucked up!".to_string());
                    }
                    match w.as_str() {
                        ":" => stack.push_back(*stack.back().unwrap()),
                        ";" => {
                            stack.pop_back().unwrap();
                        }
                        _ => return Err("Unknown op".to_string()),
                    }
                }
            }
        }
    }
    Ok(stack)
}

fn main() {
    let matches = App::new("lang")
        .version("1.0")
        .author("a66ath <pitongogi@gmail.com>")
        .about("Simple programming language")
        .arg(
            Arg::new("INPUT")
                .about("Sets the input file to use")
                .required(true)
                .index(1),
        )
        .get_matches();
    if let Some(i) = matches.value_of("INPUT") {
        let content = std::fs::read_to_string(i).unwrap();
        let res = lex(&content);
        println!("{:?}", parse(&res));
        println!("{:?}", compute(res));
    }
}
