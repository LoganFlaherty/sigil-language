/*
    Copyright 2025 Logan Flaherty

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
*/

use core::panic;
use std::env;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io;
use std::time::Instant;
use evalexpr::*;
use regex::Regex;

#[derive(Clone)]
enum Object {
    Source(Source),
    Sigil(Sigil),
    BuiltInSigil(BuiltInSigil),
}

#[derive(Clone)]
struct Source {
    name: String,
    value : String,
    sig_deps : Vec<String>, // All sigils dependent on this source
}

#[derive(Clone)]
struct Sigil {
    name: String,
    cond_expr: String,
    body: Vec<String>,
    src_deps: Vec<String>, // All sources this sigil is dependent on
}

#[derive(Clone)]
struct BuiltInSigil {
    name: String,
    func: fn(&BuiltInSigil, &mut Interpreter, Vec<String>),
    in_sigil: Option<String>,
}

struct Interpreter {
    keyword_table: HashMap<String, fn(&mut Interpreter, &Vec<&str>, &str, Option<&Sigil>)>,
    global_table: HashMap<String, Object>,
    src_cache: HashMapContext<DefaultNumericTypes>,
    invoke_queue: VecDeque<String>,
    runtime_chain: Vec<String>,
    li: u32, // Line iter
}

impl Source {
    fn new(name: String, value: String ) -> Self {
        Source {
            name: name,
            value : value,
            sig_deps : Vec::new(),
        }
    }
}

impl Sigil {
    fn new(name: String, cond_expr: String, body: Vec<String>, src_deps: Vec<String> ) -> Self {
        Sigil {
            name: name,
            cond_expr: cond_expr,
            body: body,
            src_deps: src_deps,
        }
    }
}

impl BuiltInSigil {
    fn new(name: String, func: fn(&BuiltInSigil, &mut Interpreter, Vec<String>)) -> Self {
        BuiltInSigil {
            name: name,
            func: func,
            in_sigil: None,
        }
    }

    fn whisper(&self, _: &mut Interpreter, args: Vec<String>) {
        let mut arg_str = String::new();
        for arg in args {
            arg_str += &arg.trim_matches('"');
        }
        print!("{}", arg_str);
    }

    fn pulse(&self, inter: &mut Interpreter, _: Vec<String>) {
        match &self.in_sigil {
            Some(name) => {
                if let Some(Object::Sigil(sigil)) = inter.global_table.get(name) {
                    match eval_expr(&sigil.cond_expr, &inter.src_cache) {
                        Ok(true) => {
                            inter.invoke_queue[0] = name.to_string();
                        }
                        Ok(false) => { }
                        Err(e) => { eprint!("{}", e) }
                    }
                }
            }
            None => eprint!("Cannot invoke Pulse outside a sigil."),
        };
    }
}

impl Interpreter {
    fn new() -> Self {
        let mut inter: Interpreter = Interpreter {
            keyword_table: HashMap::new(),
            global_table: HashMap::new(),
            src_cache: HashMapContext::new(),
            invoke_queue: VecDeque::new(),
            runtime_chain: Vec::new(),
            li: 0,
        };

        inter.keyword_table.insert("src".to_string(), Interpreter::construct_src);
        inter.keyword_table.insert("sigil".to_string(), Interpreter::construct_sigil);
        inter.keyword_table.insert("invoke".to_string(), Interpreter::queue_invokes);

        let whisper_sig:BuiltInSigil = BuiltInSigil::new("Whisper".to_string(), BuiltInSigil::whisper);
        inter.global_table.insert("Whisper".to_string(), Object::BuiltInSigil(whisper_sig));
        let pulse_sig:BuiltInSigil = BuiltInSigil::new("Pulse".to_string(), BuiltInSigil::pulse);
        inter.global_table.insert("Pulse".to_string(), Object::BuiltInSigil(pulse_sig));

        inter
    }

    fn construct_src(&mut self, _: &Vec<&str>, line: &str, _: Option<&Sigil>) {
        if let Some((mut name, mut val)) = line.split_once(":") {
            name = name.trim();
            val = val.trim();
            let src = Object::Source(Source::new(name.to_string(), val.to_string()));
            self.global_table.insert(name.to_string(), src);
            self.src_cache.set_value(name.to_string(), parse_value(val)).unwrap();
        }
    }

    fn construct_sigil(&mut self, lines: &Vec<&str>, line: &str, _: Option<&Sigil>) {
        // Extract name
        let sigil_header: Vec<&str> = line.split(&[':', '?']).collect();
        let name: &str = sigil_header[0].trim();

        // Catch builtin overrides
        if name == "Whisper" || name == "Pulse" {
            eprint!("Cannot override built-in {}.", name)
        }

        // Extract cond_expr and deps
        let mut cond_expr: String = String::new();
        let mut src_deps: Vec<String> = Vec::new();
        if sigil_header.len() > 1 {
            cond_expr = sigil_header[1].trim().replace(" = ", " == ").replace(" and ", " && ").replace(" or ", " || ");
            let cond_expr_split: Vec<&str> = cond_expr.split_whitespace().collect();
            for token in cond_expr_split {
                if !src_deps.contains(&token.to_string()) {
                    if let Some(node) = self.global_table.get_mut(token) {
                        match node {
                            Object::Source(src) => {
                                src_deps.push(src.name.clone());
                                src.sig_deps.push(name.to_string());
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Collect body
        let mut body: Vec<String> = Vec::new();
        let mut i: usize = (self.li + 1) as usize; 
        let lines_len: usize = lines.len();
        while i < lines_len && lines[i].starts_with(char::is_whitespace) {
            body.push(lines[i].split("#").next().unwrap().trim().to_string());
            i += 1;
        }

        // Finish
        let sigil:Object = Object::Sigil(Sigil::new(name.to_string(), cond_expr, body, src_deps));
        self.global_table.insert(name.to_string(), sigil);
        self.li = i as u32;

    }

    fn queue_invokes(&mut self, _: &Vec<&str>, line: &str, sigil: Option<&Sigil>) {
        let targets: Vec<&str> = line.split(",").collect();
        for target in targets {
            let cleaned_target = target.trim();
            if cleaned_target.is_empty() {
                continue;
            }

            if let Some(node) = self.global_table.get_mut(cleaned_target) {
                match node {
                    Object::Sigil(sigil) => {
                        self.invoke_queue.push_back(sigil.name.clone());
                    }
                    Object::Source(src) => {
                        self.invoke_queue.push_back(src.name.clone());
                    }
                    Object::BuiltInSigil(builtin) => {
                        if let Some(curr_sigil) =  sigil{
                            builtin.in_sigil = Some(curr_sigil.name.clone());
                            self.invoke_queue.push_back(builtin.name.clone());
                        }
                        else {
                            eprint!("Cannot invoke builtins outside a sigil.");
                        }
                    }
                }
            }
            else {
                eprint!("{} is not a valid invoke. ", cleaned_target)
            }
        }
    }
}

fn main() {
    // Take in command-line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        panic!("Usage: sigil <file path> <options>");
    }

    let mut interpreter: Interpreter = Interpreter::new();

    // Check if path is valid
    let file_path: &str = &args[1];
    let file: Result<String, io::Error> = fs::read_to_string(file_path);
    let start: Instant = Instant::now(); // Start benchmark
    match file {
        Ok(program) => parse(&program, &mut interpreter),
        Err(e) => eprint!("Problem opening {}: {e:?}", file_path),
    }

    invoke(&mut interpreter);
    let _elasped_t: std::time::Duration = start.elapsed(); // Stop benchmark

    // Handle optional cl args
    for a in args {
        if a == "-c" {
            print!("\n{:?}", interpreter.runtime_chain);
        }
        else if a == "-t" {
            print!("\n{:?}", _elasped_t);
        }
    }
}

fn parse(program: &str, inter: &mut Interpreter) {
    let lines: Vec<&str> = program.lines().collect();
    let lines_len: u32 = lines.len() as u32;
    while inter.li < lines_len {
        let mut line: &str = lines[inter.li as usize];
        
        // Skip empty lines and comments
        let is_line: &str = &line.trim();
        if is_line.is_empty() || is_line.starts_with("#") {
            inter.li += 1;
            continue;
        }

        // Remove inline comment
        if let Some((code, _comment)) = line.split_once("#") {
            line = code;
        }

        // Seperate keyword
        let mut keyword: &str = "";
        if let Some((key, rest)) = line.split_once(" ") {
            keyword = key;
            line = rest
        }

        // Try to map keyword
        if let Some(func) = inter.keyword_table.get(keyword) {
            func(inter, &lines, line.trim(), None);
        }
        else {
            panic!("{} is an unknown keyword.", keyword)
        }

        inter.li += 1;
    }
}

fn invoke(inter: &mut Interpreter) {
    while let Some(target) = inter.invoke_queue.pop_front() {
        inter.runtime_chain.push(target.clone());
        if let Some(node) = inter.global_table.get(&target).cloned() {
            match node {
                Object::Source(src) => {
                    // Invoke all dependent sigils
                    for sig_dep in &src.sig_deps {
                        if let Some(Object::Sigil(sigil)) = inter.global_table.get(sig_dep).cloned() {
                            match eval_expr(&sigil.cond_expr, &inter.src_cache) {
                                Ok(true) => {
                                    inter.runtime_chain.push(sigil.name.clone());
                                    invoke_sigil(inter, &sigil);
                                }
                                Ok(false) => {}
                                Err(e) => { eprint!("{}", e) }
                            }
                        }
                    }
                }
                Object::Sigil(sigil) => {
                    invoke_sigil(inter, &sigil);
                }
                Object::BuiltInSigil(builtin) => {
                    let mut args: Vec<String> = Vec::new();
                    if let Some(in_sigil_name) = &builtin.in_sigil {
                        if let Some(Object::Sigil(sig)) = inter.global_table.get(in_sigil_name) {
                            // Clone source dependencies to avoid borrow issues
                            for dep in sig.src_deps.clone() {
                                if let Some(Object::Source(src)) = inter.global_table.get(&dep) {
                                    args.push(src.value.clone());
                                }
                            }
                        }
                    }

                    (builtin.func)(&builtin, inter, args);
                }
            }
        }
    }
}

fn invoke_sigil(inter: &mut Interpreter, sigil: &Sigil) {
    for line in &sigil.body {
        let line_split: Vec<&str> = line.split_whitespace().collect();
        let key = line_split[0].trim();
        if key == "invoke" {
            // line_split is a dummy pass
            inter.keyword_table[key](inter, &line_split, line_split[1..].join(" ").trim(), Some(sigil));
        }
        else if line_split[1].trim() == ":" {
            // Check if src exists before assigning
            let val = line_split[2..].join(" ").trim().to_string();
            if let Object::Source(source) = inter.global_table.get_mut(key).unwrap() {
                let evaled_val: Result<Value, EvalexprError> = eval_with_context(&val, &inter.src_cache);
                match evaled_val {
                    Ok(value) => {
                        source.value = value.to_string();
                        inter.src_cache.set_value(source.name.clone(), value).unwrap();
                    }
                    Err(e) => { eprint!("{}", e) }
                }
                
            }
            else {
                panic!("Attempted to assign to invalid source '{}'.", key);
            }
        }
        else {
            panic!("Invalid statement in sigil '{}': {}", sigil.name, line);
        }
    }
}

fn parse_value(val: &str) -> Value {
    if val.contains('"') || val.contains("'") {
        Value::String(val.trim_matches('\'').trim_matches('"').to_string())
    }
    else if let Ok(f) = val.parse::<f64>() {
        Value::Float(f)
    }
    else if let Ok(i) = val.parse::<i64>() {
        Value::Int(i)
    }
    else if let Ok(b) = val.parse::<bool>() {
        Value::Boolean(b)
    }
    else {
        eprint!("{} is not a valid value.", val);
        Value::Empty
    }
}

fn eval_expr(cond_expr: &str, src_cache: &HashMapContext<DefaultNumericTypes>) -> Result<bool, evalexpr::EvalexprError> {
    // Fix source bool truthiness
    let re: Regex = Regex::new(r#""[^"]*"|\S+"#).unwrap();
    let mut fixed_expr: String = String::new();
    let expr_split: Vec<&str> = re.find_iter(cond_expr).map(|m| m.as_str()).collect();
    let mut e: usize = 0;
    let expr_len: usize = expr_split.len();
    while e < expr_len {
        fixed_expr += expr_split[e];
        if src_cache.get_value(expr_split[e]).is_some() && (expr_len < 2 || (e + 2 != expr_len && expr_split[e + 1] == "&&")) {
            fixed_expr += "!=0";
        }
        e += 1;
    }
    
    eval_boolean_with_context(&fixed_expr, src_cache)
}
