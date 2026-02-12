//! # Banish
//!
//! An easy to use DSL for creating state machines and fixed-point loops in Rust.
//! It allows you to define "Phases" and "Rules" that interact until they settle or transition.
//! This is the macro implementation for the `banish` crate, which provides the public API and user-facing documentation.

use proc_macro;
use proc_macro2;
use quote::quote;
use syn::{
    Expr, Ident, Result, Stmt, Token, braced, parse::{Parse, ParseStream}, parse_macro_input,
};
use core::panic;
use std::collections::HashSet;


//// AST

struct Context {
    phases: Vec<Phase>,
}

struct Phase {
    name: Ident,
    rules: Vec<Rule>,
}

struct Rule {
    name: Ident,
    condition: Option<Expr>,
    body: Vec<BanishStmt>,
}

enum BanishStmt {
    Rust(Stmt),
    PhaseJump(Ident),
}


//// Parsing

impl Parse for Context {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut phases: Vec<Phase> = Vec::with_capacity(2);
        while !input.is_empty() {
            phases.push(input.parse()?);
        }

        Ok(Context { phases })
    }
}

impl Parse for Phase {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![@]>()?;
        let name: Ident = input.parse()?;

        let mut rules: Vec<Rule> = Vec::with_capacity(1);
        while !input.is_empty() && !input.peek(Token![@]) {
            rules.push(input.parse()?);
        }

        Ok(Phase { name, rules })
    }
}

impl Parse for Rule {
    fn parse(input: ParseStream) -> Result<Self> {
        let name: Ident = input.parse()?;
        input.parse::<Token![?]>()?;
        let condition: Option<Expr> = if input.peek(syn::token::Brace) {
            None
        } else { Some(input.parse::<Expr>()?) };

        let content: syn::parse::ParseBuffer<'_>;
        braced!(content in input);
        let mut body: Vec<BanishStmt> = Vec::new();
        while !content.is_empty() {
            if content.peek(Token![=>]) {
                content.parse::<Token![=>]>()?;
                content.parse::<Token![@]>()?;
                let phase: Ident = content.parse()?;
                content.parse::<Token![;]>()?;
                body.push(BanishStmt::PhaseJump(phase));
            }
            else {
                let stmt: Stmt = content.parse()?;
                body.push(BanishStmt::Rust(stmt));
            }
        }

        Ok(Rule {
            name: name,
            condition,
            body,
        })
    }
}


//// Code Generation

#[proc_macro]
pub fn banish(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Context = parse_macro_input!(input as Context);
    if let Err(err) = validate_phase_and_rule_names(&input) {
        return err.to_compile_error().into();
    }

    let phase_blocks = input.phases.iter().enumerate().map(|(index, phase)| {
        let rules = phase.rules.iter().map(|func| {
            let body = func.body.iter().map(|stmt| {
                match stmt {
                    BanishStmt::Rust(stmt) => quote! { #stmt },
                    BanishStmt::PhaseJump(jump) => {
                        let target: usize = input.phases
                            .iter()
                            .position(|phase| &phase.name == jump)
                            .unwrap_or_else(|| { panic!("Invalid phase jump target {}", jump); });
                        
                        let target: syn::Index = syn::Index::from(target);
                        quote! {
                            __current_phase = #target;
                            continue 'banish_main;
                        }
                    },
                }
            });
            

            // If a rule has a condition, we want to run it every iteration until the condition is false.
            if let Some(condition) = &func.condition {
                quote! {
                    if #condition {
                        __interaction = true;
                        #(#body)*
                    }
                }
            }
            // If a rule is conditionless, we want to run it only once per phase.
            else {
                quote! {
                    if __first_iteration {
                        #(#body)*
                    }
                }
            }
        });

        // Phase loop
        // If no interactions occur in a full pass, exit phase
        let index: syn::Index = syn::Index::from(index);
        quote! {
            #index => {
                let mut __first_iteration = true;
                loop {
                    let mut __interaction = false;
                    #(#rules)*
                    __first_iteration = false;
                    if !__interaction {
                        break;
                    }
                }

                __current_phase += 1;
            }
        }
    });

    let expanded: proc_macro2::TokenStream = quote! {{
        let mut __current_phase: usize = 0;
        
        'banish_main: loop {
            match __current_phase {
                #(#phase_blocks)*
                _ => break,
            }
        }
    }};
    proc_macro::TokenStream::from(expanded)
}

fn validate_phase_and_rule_names(input: &Context) -> syn::Result<()> {
    let mut phase_names: HashSet<String> = HashSet::new();
    for phase in &input.phases {
        let name: String = phase.name.to_string();
        if !phase_names.insert(name.clone()) {
            return Err(syn::Error::new(
                phase.name.span(),
                format!("Duplicate phase name '{}'", name),
            ));
        }

        let mut rule_names: HashSet<String> = HashSet::new();
        for rule in &phase.rules {
            let name: String = rule.name.to_string();

            if !rule_names.insert(name.clone()) {
                return Err(syn::Error::new(
                    rule.name.span(),
                    format!(
                        "Duplicate rule '{}' in phase '{}'",
                        name, phase.name
                    ),
                ));
            }
        }
    }

    Ok(())
}