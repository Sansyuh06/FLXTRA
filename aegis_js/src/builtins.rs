//! JavaScript built-in functions

use crate::value::JsValue;

pub fn console_log(args: Vec<JsValue>) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    println!("{}", output.join(" "));
    JsValue::Undefined
}

pub fn console_error(args: Vec<JsValue>) -> JsValue {
    let output: Vec<String> = args.iter().map(|v| v.to_string()).collect();
    eprintln!("{}", output.join(" "));
    JsValue::Undefined
}

pub fn parse_int(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    let s = args[0].to_string();
    let radix = args.get(1).map(|v| v.to_number() as u32).unwrap_or(10);
    match i64::from_str_radix(s.trim(), radix) {
        Ok(n) => JsValue::Number(n as f64),
        Err(_) => JsValue::Number(f64::NAN),
    }
}

pub fn parse_float(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    match args[0].to_string().trim().parse::<f64>() {
        Ok(n) => JsValue::Number(n),
        Err(_) => JsValue::Number(f64::NAN),
    }
}

pub fn is_nan(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Boolean(true); }
    JsValue::Boolean(args[0].to_number().is_nan())
}

pub fn is_finite(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Boolean(false); }
    let n = args[0].to_number();
    JsValue::Boolean(n.is_finite())
}

pub fn type_of(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::String("undefined".to_string()); }
    JsValue::String(args[0].type_of().to_string())
}

pub fn math_abs(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    JsValue::Number(args[0].to_number().abs())
}

pub fn math_floor(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    JsValue::Number(args[0].to_number().floor())
}

pub fn math_ceil(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    JsValue::Number(args[0].to_number().ceil())
}

pub fn math_round(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NAN); }
    JsValue::Number(args[0].to_number().round())
}

pub fn math_random(_args: Vec<JsValue>) -> JsValue {
    // Add timing jitter for fingerprint resistance
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let random = ((seed % 1000000) as f64) / 1000000.0;
    JsValue::Number(random)
}

pub fn math_max(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::NEG_INFINITY); }
    let max = args.iter().map(|v| v.to_number()).fold(f64::NEG_INFINITY, f64::max);
    JsValue::Number(max)
}

pub fn math_min(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(f64::INFINITY); }
    let min = args.iter().map(|v| v.to_number()).fold(f64::INFINITY, f64::min);
    JsValue::Number(min)
}

pub fn array_push(args: Vec<JsValue>) -> JsValue {
    // Simplified - real impl would modify the array
    JsValue::Number(args.len() as f64)
}

pub fn string_length(args: Vec<JsValue>) -> JsValue {
    if args.is_empty() { return JsValue::Number(0.0); }
    JsValue::Number(args[0].to_string().len() as f64)
}
