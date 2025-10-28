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

mod errors;
mod parser;

use evalexpr::*;
use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::time::Instant;

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
    func: fn(
        &BuiltInSigil,
        Vec<String>,
        &HashMap<String, Structure>,
        &HashMapContext<DefaultNumericTypes>,
        &mut VecDeque<String>,
    ),
    in_sigil: Option<String>,
}

impl Source {
    fn new(name: String, value: String) -> Self {
        Source {
            name: name,
            value: value,
            sig_rels: Vec::new(),
        }
    }
}

impl Sigil {
    fn new(name: String, cond_expr: String, body: Vec<String>, src_deps: Vec<String>) -> Self {
        Sigil {
            name: name,
            cond_expr: cond_expr,
            body: body,
            src_rels: src_deps,
        }
    }
}

impl BuiltInSigil {
    fn new(
        name: String,
        func: fn(
            &BuiltInSigil,
            Vec<String>,
            &HashMap<String, Structure>,
            &HashMapContext<DefaultNumericTypes>,
            &mut VecDeque<String>,
        ),
    ) -> Self {
        BuiltInSigil {
            name: name,
            func: func,
            in_sigil: None,
        }
    }

    fn whisper(
        &self,
        args: Vec<String>,
        _: &HashMap<String, Structure>,
        _: &HashMapContext<DefaultNumericTypes>,
        _: &mut VecDeque<String>,
    ) {
        let mut arg_str = String::new();
        for arg in args {
            arg_str += &arg.trim_matches('"');
        }
        print!("{}", arg_str);
    }

    fn pulse(
        &self,
        _: Vec<String>,
        global_table: &HashMap<String, Structure>,
        src_cache: &HashMapContext<DefaultNumericTypes>,
        queue: &mut VecDeque<String>,
    ) {
        match &self.in_sigil {
            Some(name) => {
                if let Some(Structure::Sigil(sigil)) = global_table.get(name) {
                    match eval_boolean_with_context(&sigil.cond_expr, src_cache) {
                        Ok(true) => {
                            queue[0] = name.to_string();
                        }
                        Ok(false) => {}
                        Err(e) => {
                            panic!("{}", e)
                        }
                    }
                }
            }
            None => {
                panic!("Cannot invoke Pulse outside a sigil.")
            }
        };
    }
}

fn invoke_sigil(
    sigil: &Sigil,
    global_table: &mut HashMap<String, Structure>,
    src_cache: &mut HashMapContext<DefaultNumericTypes>,
    queue: &mut VecDeque<String>,
) {
    for line in &sigil.body {
        let line_split: Vec<&str> = line.split_whitespace().collect();
        let key = line_split[0].trim();
        if key == "invoke" {
            let err = parser::construct_queue(
                line_split[1..].join(" ").trim(),
                Some(sigil),
                global_table,
                queue,
            );

            if let Err(err) = err {
                err.print();
                return;
            }
        } else if line_split[1].trim() == ":" {
            // Check if src exists before assigning
            let val = line_split[2..].join(" ").trim().to_string();
            if let Structure::Source(source) = global_table.get_mut(key).unwrap() {
                let evaled_val: Result<Value, EvalexprError> = eval_with_context(&val, src_cache);
                match evaled_val {
                    Ok(value) => {
                        source.value = value.to_string();
                        src_cache.set_value(source.name.clone(), value).unwrap();
                    }
                    Err(e) => {
                        panic!("{}", e)
                    }
                }
            } else {
                panic!("Attempted to assign to invalid source '{}'.", key);
            }
        } else {
            panic!("Invalid statement in sigil '{}': {}", sigil.name, line);
        }
    }
}

fn main() {
    // Collect command-line args
    let args: Vec<String> = env::args().collect();

    // Read file
    let file_path = match args.get(1) {
        Some(p) => p,
        None => {
            println!("No file specified.");
            return;
        }
    };
    let file_err = format!("Problem opening {}", file_path);
    let program = fs::read_to_string(file_path).expect(&file_err);

    // Setup
    let start: Instant = Instant::now(); // Start benchmark
    let mut global_table: HashMap<String, Structure> = HashMap::new();
    let mut src_cache: HashMapContext<DefaultNumericTypes> = HashMapContext::new();
    let mut queue: VecDeque<String> = VecDeque::new();
    let mut runtime_chain: Vec<String> = Vec::new();

    let whisper_sig: BuiltInSigil = BuiltInSigil::new("Whisper".to_string(), BuiltInSigil::whisper);
    global_table.insert("Whisper".to_string(), Structure::BuiltInSigil(whisper_sig));
    let pulse_sig: BuiltInSigil = BuiltInSigil::new("Pulse".to_string(), BuiltInSigil::pulse);
    global_table.insert("Pulse".to_string(), Structure::BuiltInSigil(pulse_sig));

    // Parse program
    let err = parser::parse(program, &mut global_table, &mut src_cache, &mut queue);
    if let Err(err) = err {
        err.print();
        return;
    }

    // Queue starts
    while let Some(target) = queue.pop_front() {
        if let Some(node) = global_table.get(&target).cloned() {
            match node {
                Structure::Source(src) => {
                    // Invoke all dependent sigils
                    for sig_dep in src.sig_rels.clone() {
                        if let Some(Structure::Sigil(sigil)) = global_table.get(&sig_dep).cloned() {
                            match eval_boolean_with_context(&sigil.cond_expr, &src_cache) {
                                Ok(true) => {
                                    runtime_chain.push(sigil.name.clone());
                                    invoke_sigil(
                                        &sigil,
                                        &mut global_table,
                                        &mut src_cache,
                                        &mut queue,
                                    );
                                }
                                Ok(false) => {}
                                Err(e) => {
                                    panic!("{}", e)
                                }
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
        runtime_chain.push(target);
    }
    let _elapsed_t: std::time::Duration = start.elapsed(); // Stop benchmark

    // Handle optional cl args
    for a in args {
        if a == "-c" {
            print!("\n{:?}", runtime_chain);
        } else if a == "-t" {
            print!("\n{:?}", _elapsed_t);
        }
    }
}
