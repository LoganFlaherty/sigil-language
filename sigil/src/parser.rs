use crate::{
    Sigil, Source, Structure,
    errors::{SynErr, SyntaxError},
};

use evalexpr::{Context, ContextWithMutableVariables, DefaultNumericTypes, HashMapContext, Value};

use regex::Regex;
use std::collections::{HashMap, VecDeque};

pub fn construct_queue(
    line: &str,
    sigil: Option<&Sigil>,
    global_table: &mut HashMap<String, Structure>,
    queue: &mut VecDeque<String>,
) -> SynErr {
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
                    if let Some(curr_sigil) = sigil {
                        builtin.in_sigil = Some(curr_sigil.name.clone());
                        queue.push_back(builtin.name.clone());
                    } else {
                        return Err(SyntaxError::lnew(
                            "Cannot invoke builtins outside of a sigil.",
                        ));
                    }
                }
            }
        } else {
            return Err(SyntaxError::lnew(format!(
                "{} is not a valid invoke",
                cleaned_target
            )));
        }
    }

    Ok(())
}

fn construct_src(
    line: &str,
    global_table: &mut HashMap<String, Structure>,
    src_cache: &mut HashMapContext<DefaultNumericTypes>,
) -> SynErr {
    if let Some((mut name, mut val)) = line.split_once(":") {
        name = name.trim();
        val = val.trim();
        let src = Structure::Source(Source::new(name.to_string(), val.to_string()));
        global_table.insert(name.to_string(), src);
        if let Err(e) = src_cache.set_value(name.to_string(), parse_value(val)?) {
            return Err(SyntaxError::lnew(e.to_string()));
        }
    }

    return Ok(());
}
fn parse_value(val: &str) -> Result<Value, SyntaxError> {
    if val.contains('"') || val.contains("'") {
        Ok(Value::String(
            val.trim_matches('\'').trim_matches('"').to_string(),
        ))
    } else if let Ok(f) = val.parse::<f64>() {
        Ok(Value::Float(f))
    } else if let Ok(i) = val.parse::<i64>() {
        Ok(Value::Int(i))
    } else if let Ok(b) = val.parse::<bool>() {
        Ok(Value::Boolean(b))
    } else {
        Err(SyntaxError::lnew(format!(
            "'{}' is not a valid value.",
            val
        )))
    }
}

fn construct_sigil(
    lines: &Vec<&str>,
    line: &str,
    global_table: &mut HashMap<String, Structure>,
    src_cache: &mut HashMapContext<DefaultNumericTypes>,
    li: &mut usize,
) -> SynErr {
    // Extract name
    let sigil_header: Vec<&str> = line.split(&[':', '?']).collect();
    let name: &str = sigil_header[0].trim();

    // Catch builtin overrides
    if name == "Whisper" || name == "Pulse" {
        return Err(SyntaxError::new(
            *li,
            format!("Cannot override built-in {}.", name),
            None,
        ));
    }

    // Extract and transform cond_expr
    let mut cond_expr: String = String::new();
    let mut src_rels: Vec<String> = Vec::new();
    if sigil_header.len() > 1 {
        cond_expr = sigil_header[1]
            .trim()
            .replace(" = ", " == ")
            .replace(" and ", " && ")
            .replace(" or ", " || ");

        let re: Regex = Regex::new(r#""[^"]*"|\S+"#).expect("THIS SHOUDLD ALWAYS BE A VALID REGEX");

        // Fix source bool truthiness and extract relationships
        let mut fixed_expr: String = String::new();
        let expr_split: Vec<&str> = re.find_iter(&cond_expr).map(|m| m.as_str()).collect();
        let mut e = 0;
        let expr_len = expr_split.len();
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
            if src_cache.get_value(token).is_some()
                && (expr_len < 2 || (e + 2 < expr_len && expr_split[e + 1] == "&&"))
            {
                fixed_expr += "!=0";
            }
            e += 1;
        }

        cond_expr = fixed_expr;
    }

    // Collect body
    let mut body: Vec<String> = Vec::new();
    let mut i = *li + 1;
    let lines_len = lines.len();

    while i < lines_len && lines[i].starts_with(char::is_whitespace) {
        let line = lines[i];
        let significant_content = line.split("#").next().expect("THIS MUST EXIST").trim();
        body.push(significant_content.to_string());
        i += 1;
    }

    // Finish
    let sigil: Structure =
        Structure::Sigil(Sigil::new(name.to_string(), cond_expr, body, src_rels));
    global_table.insert(name.to_string(), sigil);
    *li = i;
    Ok(())
}

pub fn parse(
    program: String,
    global_table: &mut HashMap<String, Structure>,
    src_cache: &mut HashMapContext<DefaultNumericTypes>,
    queue: &mut VecDeque<String>,
) -> SynErr {
    let lines: Vec<&str> = program.lines().collect();
    let lines_len = lines.len();
    let mut li = 0;
    while li < lines_len {
        let mut line: &str = lines[li];

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

        // Separate keyword and map
        if let Some((key, rest)) = line.split_once(" ") {
            line = rest;
            match key {
                "src" => construct_src(line, global_table, src_cache)?,
                "sigil" => construct_sigil(&lines, line, global_table, src_cache, &mut li)?,
                "invoke" => construct_queue(line, None, global_table, queue)?,
                _ => {
                    return Err(match key.trim().is_empty() {
                        true => {
                            SyntaxError::new(li, "Invalid leading whitespace.".to_owned(), None)
                        }
                        false => SyntaxError::new(li, format!("Unknown keyword '{}'.", key), None),
                    });
                }
            }
        }
        li += 1;
    }

    Ok(())
}
