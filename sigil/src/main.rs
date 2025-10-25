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

use std::env;
use std::collections::{HashMap, VecDeque};
use std::fs;
use std::time::Instant;
use evalexpr::*;
use regex::Regex;

#[derive(Clone)]
enum Structure {
    Source(Source),
    Sigil(Sigil),
    BuiltInSigil(BuiltInSigil),
}

#[derive(Clone)]
struct Source {
    name: String,
    value: String,
    sig_rels: Vec<String>, // All sigil relationships
}

#[derive(Clone)]
struct Sigil {
    name: String,
    cond_expr: String,
    body: Vec<String>,
    src_rels: Vec<String>, // All source relationships
}

#[derive(Clone)]
struct BuiltInSigil {
    name: String,
    func: fn(&BuiltInSigil, Vec<String>, &HashMap<String, Structure>, &HashMapContext<DefaultNumericTypes>, &mut VecDeque<String>),
    in_sigil: Option<String>,
}

impl Source {
    fn new(name: String, value: String ) -> Self {
        Source {
            name: name,
            value : value,
            sig_rels : Vec::new(),
        }
    }
}

impl Sigil {
    fn new(name: String, cond_expr: String, body: Vec<String>, src_deps: Vec<String> ) -> Self {
        Sigil {
            name: name,
            cond_expr: cond_expr,
            body: body,
            src_rels: src_deps,
        }
    }
}

impl BuiltInSigil {
    fn new(name: String, func: fn(&BuiltInSigil, Vec<String>, &HashMap<String, Structure>, &HashMapContext<DefaultNumericTypes>, &mut VecDeque<String>)) -> Self {
        BuiltInSigil {
            name: name,
            func: func,
            in_sigil: None,
        }
    }

    fn whisper(&self, args: Vec<String>, _: &HashMap<String, Structure>, _: &HashMapContext<DefaultNumericTypes>, _: &mut VecDeque<String>) {
        let mut arg_str = String::new();
        for arg in args {
            arg_str += &arg.trim_matches('"');
        }
        print!("{}", arg_str);
    }

    fn pulse(&self, _: Vec<String>, global_table: &HashMap<String, Structure>, src_cache: &HashMapContext<DefaultNumericTypes>, queue: &mut VecDeque<String>) {
        match &self.in_sigil {
            Some(name) => {
                if let Some(Structure::Sigil(sigil)) = global_table.get(name) {
                    match eval_boolean_with_context(&sigil.cond_expr, src_cache) {
                        Ok(true) => {
                            queue[0] = name.to_string();
                        }
                        Ok(false) => { }
                        Err(e) => { panic!("{}", e) }
                    }
                }
            }
            None => { panic!("Cannot invoke Pulse outside a sigil.") }
        };
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
        panic!("{} is not a valid value.", val);
    }
}

fn construct_src(line: &str, global_table: &mut HashMap<String, Structure>, src_cache: &mut HashMapContext<DefaultNumericTypes>) {
    if let Some((mut name, mut val)) = line.split_once(":") {
        name = name.trim();
        val = val.trim();
        let src = Structure::Source(Source::new(name.to_string(), val.to_string()));
        global_table.insert(name.to_string(), src);
        src_cache.set_value(name.to_string(), parse_value(val)).unwrap();
    }
}

fn construct_sigil(lines: &Vec<&str>, line: &str, global_table: &mut HashMap<String, Structure>, src_cache: &mut HashMapContext<DefaultNumericTypes>, li: &mut u32) {
    // Extract name
    let sigil_header: Vec<&str> = line.split(&[':', '?']).collect();
    let name: &str = sigil_header[0].trim();

    // Catch builtin overrides
    if name == "Whisper" || name == "Pulse" {
        panic!("Cannot override built-in {}.", name)
    }

    // Extract and transform cond_expr
    let mut cond_expr: String = String::new();
    let mut src_rels: Vec<String> = Vec::new();
    if sigil_header.len() > 1 {
        cond_expr = sigil_header[1].trim().replace(" = ", " == ").replace(" and ", " && ").replace(" or ", " || ");

        // Fix source bool truthiness and extract relationships
        let re: Regex = Regex::new(r#""[^"]*"|\S+"#).unwrap();
        let mut fixed_expr: String = String::new();
        let expr_split: Vec<&str> = re.find_iter(&cond_expr).map(|m| m.as_str()).collect();
        let mut e: usize = 0;
        let expr_len: usize = expr_split.len();
        while e < expr_len {
            let token = expr_split[e];
            fixed_expr += token;

            // Determine relationship
            if !src_rels.contains(&token.to_string()) {
                if let Some(node) = global_table.get_mut(token) {
                    match node {
                        Structure::Source(src) => {
                            src_rels.push(src.name.clone());
                            src.sig_rels.push(name.to_string());
                        }
                        _ => {}
                    }
                }
            }

            // If a source is by itself or chained for truthiness
            if src_cache.get_value(token).is_some() && (expr_len < 2 || (e + 2 < expr_len && expr_split[e + 1] == "&&")) {
                fixed_expr += "!=0";
            }
            e += 1;
        }

        cond_expr = fixed_expr;
    }

    // Collect body
    let mut body: Vec<String> = Vec::new();
    let mut i: u32 = *li + 1;
    let lines_len: usize = lines.len();
    while i < lines_len as u32 && lines[i as usize].starts_with(char::is_whitespace) {
        body.push(lines[i as usize].split("#").next().unwrap().trim().to_string());
        i += 1;
    }

    // Finish
    let sigil: Structure = Structure::Sigil(Sigil::new(name.to_string(), cond_expr, body, src_rels));
    global_table.insert(name.to_string(), sigil);
    *li = i;

}

fn construct_queue(line: &str, sigil: Option<&Sigil>, global_table: &mut HashMap<String, Structure>, queue: &mut VecDeque<String>) {
    let targets: Vec<&str> = line.split(",").collect();
    for target in targets {
        let cleaned_target = target.trim();
        if cleaned_target.is_empty() {
            continue;
        }

        if let Some(node) = global_table.get_mut(cleaned_target) {
            match node {
                Structure::Sigil(sigil) => {
                    queue.push_back(sigil.name.clone());
                }
                Structure::Source(src) => {
                    queue.push_back(src.name.clone());
                }
                Structure::BuiltInSigil(builtin) => {
                    if let Some(curr_sigil) =  sigil{
                        builtin.in_sigil = Some(curr_sigil.name.clone());
                        queue.push_back(builtin.name.clone());
                    }
                    else {
                        panic!("Cannot invoke builtins outside a sigil.");
                    }
                }
            }
        }
        else {
            panic!("{} is not a valid invoke. ", cleaned_target)
        }
    }
}

fn invoke_sigil(sigil: &Sigil, global_table: &mut HashMap<String, Structure>, src_cache: &mut HashMapContext<DefaultNumericTypes>, queue: &mut VecDeque<String>) {
    for line in &sigil.body {
        let line_split: Vec<&str> = line.split_whitespace().collect();
        let key = line_split[0].trim();
        if key == "invoke" {
            construct_queue(line_split[1..].join(" ").trim(), Some(sigil), global_table, queue);
        }
        else if line_split[1].trim() == ":" {
            // Check if src exists before assigning
            let val = line_split[2..].join(" ").trim().to_string();
            if let Structure::Source(source) = global_table.get_mut(key).unwrap() {
                let evaled_val: Result<Value, EvalexprError> = eval_with_context(&val, src_cache);
                match evaled_val {
                    Ok(value) => {
                        source.value = value.to_string();
                        src_cache.set_value(source.name.clone(), value).unwrap();
                    }
                    Err(e) => { panic!("{}", e) }
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

fn parse(program: String, global_table: &mut HashMap<String, Structure>, src_cache: &mut HashMapContext<DefaultNumericTypes>, queue: &mut VecDeque<String>) {
    let lines: Vec<&str> = program.lines().collect();
    let lines_len: usize = lines.len();
    let mut li: u32 = 0;
    while li < lines_len as u32 {
        let mut line: &str = lines[li as usize];
        
        // Skip empty and comments lines
        let is_line: &str = line.trim();
        if is_line.is_empty() || is_line.starts_with("#") {
            li += 1;
            continue;
        }

        // Remove inline comment
        if let Some((code, _comment)) = line.split_once("#") {
            line = code;
        }

        // Seperate keyword and map
        if let Some((key, rest)) = line.split_once(" ") {
            line = rest;
            match key {
                "src" => construct_src(line, global_table, src_cache),
                "sigil" => construct_sigil(&lines, line, global_table, src_cache, &mut li),
                "invoke" => construct_queue(line, None, global_table, queue),
                _ => panic!("{} is an unknown keyword.", key),
            }
        }
        li += 1;
    }
}

fn main() {
    // Collect command-line args
    let args: Vec<String> = env::args().collect();

    // Read file
    let file_path = &args[1];
    let file_err = format!("Problem opening {}", file_path);
    let program = fs::read_to_string(file_path).expect(&file_err);

    // Setup
    let start: Instant = Instant::now(); // Start benchmark
    let mut global_table: HashMap<String, Structure> = HashMap::new();
    let mut src_cache: HashMapContext<DefaultNumericTypes> = HashMapContext::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut runtime_chain: Vec<String> = Vec::new();

    let whisper_sig:BuiltInSigil = BuiltInSigil::new("Whisper".to_string(), BuiltInSigil::whisper);
    global_table.insert("Whisper".to_string(), Structure::BuiltInSigil(whisper_sig));
    let pulse_sig:BuiltInSigil = BuiltInSigil::new("Pulse".to_string(), BuiltInSigil::pulse);
    global_table.insert("Pulse".to_string(), Structure::BuiltInSigil(pulse_sig));

    // Parse program
    parse(program, &mut global_table, &mut src_cache, &mut queue);

    // Queue starts
    while let Some(target) = queue.pop_front() {
        runtime_chain.push(target.clone());
        if let Some(node) = global_table.get(&target).cloned() {
            match node {
                Structure::Source(src) => {
                    // Invoke all dependent sigils
                    for sig_dep in src.sig_rels.clone() {
                        if let Some(Structure::Sigil(sigil)) = global_table.get(&sig_dep).cloned() {
                            match eval_boolean_with_context(&sigil.cond_expr, &src_cache) {
                                Ok(true) => {
                                    runtime_chain.push(sigil.name.clone());
                                    invoke_sigil(&sigil, &mut global_table, &mut src_cache, &mut queue);
                                }
                                Ok(false) => {}
                                Err(e) => { panic!("{}", e) }
                            }
                        }
                    }
                }
                Structure::Sigil(sigil) => {
                    invoke_sigil(&sigil, &mut global_table, &mut src_cache, &mut queue);
                }
                Structure::BuiltInSigil(builtin) => {
                    let mut args: Vec<String> = Vec::new();
                    if let Some(in_sigil_name) = &builtin.in_sigil {
                        if let Some(Structure::Sigil(sig)) = global_table.get(in_sigil_name) {
                            // Clone source dependencies to avoid borrow issues
                            for dep in sig.src_rels.clone() {
                                if let Some(Structure::Source(src)) = global_table.get(&dep) {
                                    args.push(src.value.clone());
                                }
                            }
                        }
                    }

                    (builtin.func)(&builtin, args, &global_table, &src_cache, &mut queue);
                }
            }
        }
    }
    let _elasped_t: std::time::Duration = start.elapsed(); // Stop benchmark

    // Handle optional cl args
    for a in args {
        if a == "-c" {
            print!("\n{:?}", runtime_chain);
        }
        else if a == "-t" {
            print!("\n{:?}", _elasped_t);
        }
    }
}
