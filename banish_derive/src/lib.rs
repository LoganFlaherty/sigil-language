//! # Banish
//!
//! An easy to use DSL for creating state machines and rules-base logic.
//! It allows you to define "States" and "Rules" that execute until they reach a fixed point or transition.
//! This is the macro implementation for the `banish` crate, which provides the public API and user-facing documentation.

use proc_macro;
use proc_macro2;
use quote::quote;
use syn::{
    Expr, Ident, Result, Stmt, Token, braced,
    parse::{Parse, ParseStream}, parse_macro_input,
};
use core::panic;
use std::collections::HashSet;


//// AST

struct Context {
    states: Vec<State>,
}

struct State {
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
    StateTransition(Ident),
    Return(Expr),
}


//// Parsing

impl Parse for Context {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut states: Vec<State> = Vec::with_capacity(2);
        while !input.is_empty() {
            states.push(input.parse()?);
        }

        Ok(Context { states })
    }
}

impl Parse for State {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![@]>()?;
        let name: Ident = input.parse()?;

        let mut rules: Vec<Rule> = Vec::with_capacity(1);
        while !input.is_empty() && !input.peek(Token![@]) {
            rules.push(input.parse()?);
        }

        Ok(State { name, rules })
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
                let state: Ident = content.parse()?;
                content.parse::<Token![;]>()?;
                body.push(BanishStmt::StateTransition(state));
            }
            else if content.peek(Token![return]) {
                content.parse::<Token![return]>()?;
                if content.peek(Token![;]) {
                    content.parse::<Token![;]>()?;
                    body.push(BanishStmt::Return(syn::parse_quote! { () }));
                }
                else {
                    let expr: Expr = content.parse()?;
                    content.parse::<Token![;]>()?;
                    body.push(BanishStmt::Return(expr));
                    }
            }
            else {
                let stmt: Stmt = content.parse()?;
                body.push(BanishStmt::Rust(stmt));
            }
        }

        Ok(Rule { name, condition, body })
    }
}


//// Code Generation

#[proc_macro]
pub fn banish(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: Context = parse_macro_input!(input as Context);
    if let Err(err) = validate_state_and_rule_names(&input) {
        return err.to_compile_error().into();
    }

    let state_blocks = input.states.iter().enumerate().map(|(index, state)| {
        let rules = state.rules.iter().map(|func| {
            let body = func.body.iter().map(|stmt| {
                match stmt {
                    BanishStmt::Rust(stmt) => quote! { #stmt },
                    BanishStmt::StateTransition(transition) => {
                        let target: usize = input.states
                            .iter()
                            .position(|state| &state.name == transition)
                            .unwrap_or_else(|| { panic!("Invalid state transition target {}", transition); });
                        
                        let target: syn::Index = syn::Index::from(target);
                        quote! {
                            __current_state = #target;
                            continue 'banish_main;
                        }
                    },
                    BanishStmt::Return(expr) => quote! {
                        __banish_return = Some(#expr);
                        break 'banish_main;
                    }
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
            // If a rule is conditionless, we want to run it only once per state.
            else {
                quote! {
                    if __first_iteration {
                        __interaction = true;
                        #(#body)*
                    }
                }
            }
        });

        // State loop
        // If no interactions occur in a full pass, exit state
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

                __current_state += 1;
            }
        }
    });

    let expanded: proc_macro2::TokenStream = quote! {{
        let mut __banish_return: Option<_> = None;
        let mut __current_state: usize = 0;
        
        'banish_main: loop {
            match __current_state {
                #(#state_blocks)*
                _ => break,
            }
        }

        __banish_return
    }};
    proc_macro::TokenStream::from(expanded)
}

fn validate_state_and_rule_names(input: &Context) -> syn::Result<()> {
    let mut state_names: HashSet<String> = HashSet::new();
    for state in &input.states {
        let name: String = state.name.to_string();
        if !state_names.insert(name.clone()) {
            return Err(syn::Error::new(
                state.name.span(),
                format!("Duplicate state name '{}'", name),
            ));
        }

        let mut rule_names: HashSet<String> = HashSet::new();
        for rule in &state.rules {
            let name: String = rule.name.to_string();

            if !rule_names.insert(name.clone()) {
                return Err(syn::Error::new(
                    rule.name.span(),
                    format!(
                        "Duplicate rule '{}' in state '{}'",
                        name, state.name
                    ),
                ));
            }
        }
    }

    Ok(())
}