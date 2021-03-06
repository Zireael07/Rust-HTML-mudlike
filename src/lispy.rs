//based on https://gist.github.com/stopachka/22b4b06b8263687d7178f61fb22e1bf2
//described in https://stopa.io/post/222
//custom: implementation of do; implementation of loop (based on https://github.com/Pebaz/LambdaCore/)

use std::collections::HashMap;
use std::fmt;
use std::num::ParseFloatError;

use super::log;

/*
  Types
*/

//TODO: expose structs to Rust
#[derive(Clone)]
pub enum RispExp {
  Bool(bool), 
  String(String),
  Symbol(String),
  Number(f64),
  List(Vec<RispExp>),
  Func(fn(&[RispExp]) -> Result<RispExp, RispErr>),
}

impl fmt::Display for RispExp {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let str = match self {
      RispExp::Bool(a) => a.to_string(),
      RispExp::String(a) => a.to_string(),
      RispExp::Symbol(s) => s.clone(),
      RispExp::Number(n) => n.to_string(),
      RispExp::List(list) => {
        let xs: Vec<String> = list
          .iter()
          .map(|x| x.to_string())
          .collect();
        format!("({})", xs.join(","))
      },
      RispExp::Func(_) => "Function {}".to_string(),
    };
    
    write!(f, "{}", str)
  }
}

#[derive(Debug)]
pub enum RispErr {
  Reason(String),
}

#[derive(Clone)]
pub struct RispEnv {
  pub data: HashMap<String, RispExp>,
}

/*
  Parse
  (manually to avoid pulling in a big dependency such as nom or combine)
*/

//this is very simple, based on Peter Norvig's Lispy
//sucks because only splits by whitespace (can't have multi words strings, for instance)
// better tokenizer in http://norvig.com/lispy2.html
fn tokenize(expr: String) -> Vec<String> {
  expr
    .replace("{", " ( ") //is trick! we use brackets not parens
    .replace("}", " ) ")
    .split_whitespace()
    .map(|x| x.to_string())
    .collect()
}

//TODO: implement comments
fn parse<'a>(tokens: &'a [String]) -> Result<(RispExp, &'a [String]), RispErr> {
  let (token, rest) = tokens.split_first()
    .ok_or(
      RispErr::Reason("could not get token".to_string())
    )?;
  match &token[..] {
    "(" => read_seq(rest),
    ")" => Err(RispErr::Reason("unexpected `)`".to_string())),
    _ => Ok((parse_atom(token), rest)),
  }
}

fn read_seq<'a>(tokens: &'a [String]) -> Result<(RispExp, &'a [String]), RispErr> {
  let mut res: Vec<RispExp> = vec![];
  let mut xs = tokens;
  loop {
    let (next_token, rest) = xs
      .split_first()
      .ok_or(RispErr::Reason("could not find closing `)`".to_string()))
      ?;
    if next_token == ")" {
      return Ok((RispExp::List(res), rest)) // skip `)`, head to the token after
    }
    let (exp, new_xs) = parse(&xs)?;
    res.push(exp);
    xs = new_xs;
  }
}

fn parse_atom(token: &str) -> RispExp {      
  match token.as_ref() {
    "true" => RispExp::Bool(true),
    "false" => RispExp::Bool(false),
    _ => {
      let potential_float: Result<f64, ParseFloatError> = token.parse();
      match potential_float {
        Ok(v) => RispExp::Number(v),
        Err(_) => {
          //log!("{}", format!("{} is a string or a symbol", token));
          if token.chars().nth(0) == Some('"') {
            RispExp::String(token.to_string().clone())
          }
          else {
            RispExp::Symbol(token.to_string().clone())
          }
        } 
      }
    }  
  }
}

/*
  Env
*/

macro_rules! ensure_tonicity {
  ($check_fn:expr) => {{
    |args: &[RispExp]| -> Result<RispExp, RispErr> {
      let floats = parse_list_of_floats(args)?;
      let first = floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
      let rest = &floats[1..];
      fn f (prev: &f64, xs: &[f64]) -> bool {
        match xs.first() {
          Some(x) => $check_fn(prev, x) && f(x, &xs[1..]),
          None => true,
        }
      };
      Ok(RispExp::Bool(f(first, rest)))
    }
  }};
}

//stub
impl RispEnv {
  pub fn new() -> RispEnv {
    RispEnv {
        data: HashMap::new(),
    }
  }
}


pub fn default_env() -> RispEnv {
  let mut data: HashMap<String, RispExp> = HashMap::new();
  
  //FIXME: works poorly with multi-word strings
  data.insert(
    "println".to_string(),
    RispExp::Func(
      |args: &[RispExp]| -> Result<RispExp, RispErr> {
        for arg in args.iter() {
          log!("{}", format!("{}", arg));
          //eval(&arg.clone(), env)?;
        }

        Ok(RispExp::Bool(true))
      }
    )
  );

  //calculations
  data.insert(
    "+".to_string(), 
    RispExp::Func(
      |args: &[RispExp]| -> Result<RispExp, RispErr> {
        let sum = parse_list_of_floats(args)?.iter().fold(0.0, |sum, a| sum + a);
        
        Ok(RispExp::Number(sum))
      }
    )
  );
  data.insert(
    "-".to_string(), 
    RispExp::Func(
      |args: &[RispExp]| -> Result<RispExp, RispErr> {
        let floats = parse_list_of_floats(args)?;
        let first = *floats.first().ok_or(RispErr::Reason("expected at least one number".to_string()))?;
        let sum_of_rest = floats[1..].iter().fold(0.0, |sum, a| sum + a);
        
        Ok(RispExp::Number(first - sum_of_rest))
      }
    )
  );
  //comparison operators
  data.insert(
    "=".to_string(), 
    RispExp::Func(ensure_tonicity!(|a, b| a == b))
  );
  data.insert(
    ">".to_string(), 
    RispExp::Func(ensure_tonicity!(|a, b| a > b))
  );
  data.insert(
    ">=".to_string(), 
    RispExp::Func(ensure_tonicity!(|a, b| a >= b))
  );
  data.insert(
    "<".to_string(), 
    RispExp::Func(ensure_tonicity!(|a, b| a < b))
  );
  data.insert(
    "<=".to_string(), 
    RispExp::Func(ensure_tonicity!(|a, b| a <= b))
  );
  
  RispEnv {data}
}

pub fn parse_list_of_floats(args: &[RispExp]) -> Result<Vec<f64>, RispErr> {
  args
    .iter()
    .map(|x| parse_single_float(x))
    .collect::<Result<Vec<f64>, RispErr>>()
}

pub fn parse_single_float(exp: &RispExp) -> Result<f64, RispErr> {
  match exp {
    RispExp::Number(num) => Ok(*num),
    _ => Err(RispErr::Reason("expected a number".to_string())),
  }
}

/*
  Eval
*/

fn eval_if_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
  let test_form = arg_forms.first().ok_or(
    RispErr::Reason(
      "expected test form".to_string(),
    )
  )?;
  let test_eval = eval(test_form, env)?;
  match test_eval {
    RispExp::Bool(b) => {
      let form_idx = if b { 1 } else { 2 };
      let res_form = arg_forms.get(form_idx)
        .ok_or(RispErr::Reason(
          format!("expected form idx={}", form_idx)
        ))?;
      let res_eval = eval(res_form, env);
      
      res_eval
    },
    _ => Err(
      RispErr::Reason(format!("unexpected test form='{}'", test_form.to_string()))
    )
  }
}

// 'def' defines a variable
fn eval_def_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
  let first_form = arg_forms.first().ok_or(
    RispErr::Reason(
      "expected first form".to_string(),
    )
  )?;
  let first_str = match first_form {
    RispExp::Symbol(s) => Ok(s.clone()),
    _ => Err(RispErr::Reason(
      "expected first form to be a symbol".to_string(),
    ))
  }?;
  let second_form = arg_forms.get(1).ok_or(
    RispErr::Reason(
      "expected second form".to_string(),
    )
  )?;
  if arg_forms.len() > 2 {
    return Err(
      RispErr::Reason(
        "def can only have two forms ".to_string(),
      )
    )
  } 
  let second_eval = eval(second_form, env)?;
  env.data.insert(first_str, second_eval);
  
  Ok(first_form.clone())
}

fn eval_do_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
  if let Some((last, args)) = (&arg_forms[0..]).split_last() {
    //log!("Eval do");
    for arg in args.iter() {
        eval(&arg.clone(), env);
    }
    eval(&last.clone(), env)
  } else {
      return Err(RispErr::Reason("Empty do block".to_string(),))
  }
}  

fn eval_loop_args(arg_forms: &[RispExp], env: &mut RispEnv) -> Result<RispExp, RispErr> {
  let first_form = arg_forms.first().ok_or(
    RispErr::Reason(
      "expected first form".to_string(),
    )
  )?;
  let first_str = match first_form {
    RispExp::Symbol(s) => Ok(s.clone()),
    _ => Err(RispErr::Reason(
      "expected first form to be a symbol".to_string(),
    ))
  }?;
  let second_form = arg_forms.get(1).ok_or(
    RispErr::Reason(
      "expected second form".to_string(),
    )
  )?;
  let iters = match second_form {
    RispExp::Number(s) => Ok(s.clone()),
    RispExp::Symbol(s) => { 
      let exp = eval(second_form, env)?;
      match exp {
        RispExp::Number(n) => Ok(n.clone()),
        _ => Err(RispErr::Reason("unknown symbol".to_string()))
      }
    },
    _ => Err(RispErr::Reason(
      "expected second form to be a number".to_string(),
    ))
  }?;
  let third_form = arg_forms.get(2).ok_or(
    RispErr::Reason(
      "expected third form".to_string(),
    )
  )?;

  log!("Loop eval: {} {} {}", first_str, iters, third_form);

  //the meat of this function
  for i in 0..iters as i64 {
    //log!("Running the loop i {} ", i);
    //if we have a value that is our first form
    //if RispExp::Symbol(s) == first_str {

      //assign the *current* value of iters to first form
      env.data.insert(first_str.clone(), RispExp::Number(i as f64));
      //evaluate
      eval(third_form, env);
    //}

  }
  Ok(RispExp::Bool(true))
}


//
fn eval_built_in_form(
  exp: &RispExp, arg_forms: &[RispExp], env: &mut RispEnv
) -> Option<Result<RispExp, RispErr>> {
  match exp {
    RispExp::Symbol(s) => 
      match s.as_ref() {
        "if" => Some(eval_if_args(arg_forms, env)),
        "def" => Some(eval_def_args(arg_forms, env)),
        "do" => Some(eval_do_args(arg_forms, env)),
        "loop" => Some(eval_loop_args(arg_forms, env)),
        _ => None,
      }
    ,
    _ => None,
  }
}

//simple tree walking interpreter
fn eval(exp: &RispExp, env: &mut RispEnv) -> Result<RispExp, RispErr> {
  match exp {
    RispExp::Symbol(k) =>
        env.data.get(k)
        .ok_or(
          RispErr::Reason(
            format!("unexpected symbol k='{}'", k)
          )
        )
        .map(|x| x.clone())
    ,
    RispExp::Number(_a) => Ok(exp.clone()),
    RispExp::List(list) => {
      let first_form = list
        .first()
        .ok_or(RispErr::Reason("expected a non-empty list".to_string()))?;
      let arg_forms = &list[1..];
      match eval_built_in_form(first_form, arg_forms, env) {
        Some(res) => res,
        // not built-in, continue as normal
        None => {
          let first_eval = eval(first_form, env)?;
          //log!("f:{}", first_eval);
          match first_eval {
            RispExp::Func(f) => {
              let args_eval = arg_forms
                .iter()
                .map(|x| eval(x, env))
                .collect::<Result<Vec<RispExp>, RispErr>>();
              f(&args_eval?)
            },
            _ => Err(
              RispErr::Reason("first form must be a function".to_string())
            ),
          }
        }
      }    
    },
    RispExp::Func(_) => Err(
      RispErr::Reason("unexpected form".to_string())
    ),
    RispExp::Bool(_a) => Ok(exp.clone()),
    RispExp::String(_a) => Ok(exp.clone()),
  }
}

/*
  Repl
*/

fn parse_eval(expr: String, env: &mut RispEnv) -> Result<RispExp, RispErr> {
  let (parsed_exp, _) = parse(&tokenize(expr))?;
  let evaled_exp = eval(&parsed_exp, env)?;
  
  Ok(evaled_exp)
}

fn slurp_expr() -> String {
  let mut expr = String::new();
  
  expr = "{do {spawn {+ 2 3} \"patron\" }
  { spawn {+ 2 2} \"patron\" } }".to_string();

  //io::stdin().read_line(&mut expr)
  //  .expect("Failed to read line");
  
  expr
}

//split out for ease of use
pub fn slurp_eval(env: &mut RispEnv) {
    log!("lispy >");
    let expr = slurp_expr();
    match parse_eval(expr, env) {
      //use log! and format! instead of println! to display stuff in browser
      Ok(res) => log!("{}", format!("// ???? => {}", res)),
      Err(e) => match e {
        RispErr::Reason(msg) => log!("{}", format!("// ???? => {}", msg)),
      },
    }
}

//split out for ease of use
pub fn read_eval(data: String, env: &mut RispEnv) {
  log!("lispy >");
  let expr = data;
  match parse_eval(expr, env) {
    //use log! and format! instead of println! to display stuff in browser
    Ok(res) => log!("{}", format!("// ???? => {}", res)),
    Err(e) => match e {
      RispErr::Reason(msg) => log!("{}", format!("// ???? => {}", msg)),
    },
  }
}


fn main() {
  let env = &mut default_env();
  loop {
    slurp_eval(env);
  }
}

//public function to run from main module
pub fn run_lisp(){
    let env = &mut default_env();
    slurp_eval(env);
}
