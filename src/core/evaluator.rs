use crate::core::ast::*;
use crate::core::env::{Environment, UserFunction};
use crate::core::functions;
use crate::core::units::{self, CompoundUnit};
use crate::core::value::Value;

#[derive(Debug, Clone, thiserror::Error)]
pub enum EvalError {
    #[error("division by zero")]
    DivisionByZero,
    #[error("unknown function '{name}'")]
    UnknownFunction { name: String },
    #[error("unknown variable '{name}'")]
    UnknownVariable { name: String },
    #[error("invalid argument for {func}: {reason}")]
    InvalidArgument { func: String, reason: String },
    #[error("cannot convert '{from}' to '{to}': incompatible dimensions")]
    IncompatibleUnits { from: String, to: String },
    #[error("expected unit for conversion, got bare number")]
    NoUnitToConvert,
    #[error("dimensional mismatch: cannot {op} '{left}' and '{right}'")]
    DimensionalMismatch {
        op: String,
        left: String,
        right: String,
    },
    #[error("invalid number of arguments for {name}: expected {expected}, got {got}")]
    ArgCount {
        name: String,
        expected: usize,
        got: usize,
    },
    #[error("incomplete expression")]
    Incomplete,
    #[error("unit not recognized: '{unit}'")]
    UnknownUnit { unit: String },
}

pub fn evaluate(expr: &Expr, env: &Environment) -> Result<Value, EvalError> {
    match expr {
        Expr::Number(n) => Ok(Value::new(*n)),

        Expr::Identifier(name) => {
            if let Some(val) = env.get(name) {
                Ok(val)
            } else if let Some(val) = crate::core::constants::get(name) {
                Ok(val)
            } else {
                Err(EvalError::UnknownVariable { name: name.clone() })
            }
        }

        Expr::BinaryOp { op, left, right } => {
            let l = evaluate(left, env)?;
            let r = evaluate(right, env)?;
            apply_binary(op, l, r)
        }

        Expr::UnaryOp { op, operand } => {
            let val = evaluate(operand, env)?;
            match op {
                UnaryOperator::Neg => {
                    Ok(Value::new(-val.number()))
                }
            }
        }

        Expr::FunctionCall { name, args } => {
            // Special integration functions that take a function expression as first arg
            match name.as_str() {
                "trapz" => return eval_integration(args, env, IntegrationMethod::Trapezoidal),
                "simpson" => return eval_integration(args, env, IntegrationMethod::Simpson),
                "rkf45" => return eval_integration(args, env, IntegrationMethod::Rkf45),
                "quadratic" => return eval_quadratic(args, env),
                _ => {}
            }

            // First check for user-defined functions
            if let Some(func) = env.get_function(name).cloned() {
                return eval_user_function(&func, args, env);
            }

            // Built-in functions
            let values: Result<Vec<Value>, EvalError> =
                args.iter().map(|a| evaluate(a, env)).collect();
            let values = values?;

            // Functions operate on dimensionless numbers, so we strip units
            let nums: Vec<f64> = values.iter().map(|v| v.number()).collect();

            let result = functions::call(name, &nums).map_err(|e| match e {
                functions::FuncError::Unknown => EvalError::UnknownFunction { name: name.clone() },
                functions::FuncError::ArgCount { expected, got } => EvalError::ArgCount {
                    name: name.clone(),
                    expected,
                    got,
                },
                functions::FuncError::InvalidArg(reason) => EvalError::InvalidArgument {
                    func: name.clone(),
                    reason,
                },
            })?;

            Ok(Value::new(result))
        }

        Expr::Assignment { name: _, value } => {
            let val = evaluate(value, env)?;
            Ok(val.clone())
        }

        Expr::UnitConvert { value, target_unit } => {
            let val = evaluate(value, env)?;
            if let Some(ref src_unit) = val.unit {
                let src_str = src_unit.to_string();
                let result =
                    units::convert(val.number(), &src_str, target_unit).map_err(|e| match e {
                        units::UnitError::DimensionalMismatch(_, _)
                        | units::UnitError::Incompatible { .. } => EvalError::IncompatibleUnits {
                            from: src_str.clone(),
                            to: target_unit.clone(),
                        },
                        _ => EvalError::UnknownUnit {
                            unit: src_str.clone(),
                        },
                    })?;

                let mut new_unit = CompoundUnit::new();
                // Parse target_unit as compound
                if let Ok(compound) = units::parse_compound_unit(target_unit) {
                    Ok(Value::with_unit(result, compound))
                } else {
                    new_unit.add(target_unit, 1);
                    Ok(Value::with_unit(result, new_unit))
                }
            } else {
                Err(EvalError::NoUnitToConvert)
            }
        }

        Expr::UnitValue { value, unit } => {
            let val = evaluate(value, env)?;
            // Try to parse as compound unit
            if let Ok(compound) = units::parse_compound_unit(unit) {
                Ok(Value::with_unit(val.number(), compound))
            } else if units::is_valid_unit(unit) {
                let mut compound = CompoundUnit::new();
                compound.add(unit, 1);
                Ok(Value::with_unit(val.number(), compound))
            } else {
                Err(EvalError::UnknownUnit { unit: unit.clone() })
            }
        }

        Expr::FunctionDef { name, .. } => {
            // Function definitions are handled at a higher level (app.rs)
            // If we encounter one here, it's an error
            Err(EvalError::UnknownFunction { name: name.clone() })
        }
    }
}

fn eval_user_function(
    func: &UserFunction,
    args: &[Expr],
    env: &Environment,
) -> Result<Value, EvalError> {
    // Check argument count
    if args.len() != func.params.len() {
        return Err(EvalError::ArgCount {
            name: func.name.clone(),
            expected: func.params.len(),
            got: args.len(),
        });
    }

    // Evaluate arguments
    let arg_values: Result<Vec<Value>, EvalError> = args.iter().map(|a| evaluate(a, env)).collect();
    let arg_values = arg_values?;

    // Create local environment with parameters bound
    let mut local_env = Environment::new();

    // Copy existing variables (for closure-like behavior)
    for (name, value) in env.iter() {
        local_env.set(name.clone(), value.clone());
    }

    // Bind parameters
    for (param, value) in func.params.iter().zip(arg_values.iter()) {
        local_env.set(param.clone(), value.clone());
    }

    // Evaluate function body
    evaluate(&func.body, &local_env)
}

fn apply_binary(op: &BinaryOperator, l: Value, r: Value) -> Result<Value, EvalError> {
    match op {
        BinaryOperator::Add => {
            // Check dimensional compatibility
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "add".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            // If both have units, they must be the same (or convert)
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    // Try to convert r to l's units
                    let r_converted = units::convert(r.number(), &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number() + r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    // One has unit, one doesn't - just add numbers, keep unit
                    Ok(Value::with_unit(l.number() + r.number(), u.clone()))
                }
                (None, None) => Ok(Value::new(l.number() + r.number())),
            }
        }
        BinaryOperator::Sub => {
            // Same logic as addition
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "subtract".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    let r_converted = units::convert(r.number(), &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number() - r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    Ok(Value::with_unit(l.number() - r.number(), u.clone()))
                }
                (None, None) => Ok(Value::new(l.number() - r.number())),
            }
        }
        BinaryOperator::Mul => {
            let result_num = l.number() * r.number();
            match (l.unit, r.unit) {
                (Some(mut u1), Some(u2)) => {
                    // Multiply units: combine all parts
                    for part in u2.parts {
                        u1.add(&part.name, part.power);
                    }
                    // Simplify if possible
                    if let Some(simplified) = units::simplify_unit(&u1) {
                        let mut new_unit = CompoundUnit::new();
                        new_unit.add(&simplified, 1);
                        Ok(Value::with_unit(result_num, new_unit))
                    } else {
                        Ok(Value::with_unit(result_num, u1))
                    }
                }
                (Some(u), None) | (None, Some(u)) => Ok(Value::with_unit(result_num, u)),
                (None, None) => Ok(Value::new(result_num)),
            }
        }
        BinaryOperator::Div => {
            if r.number() == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            let result_num = l.number() / r.number();
            match (l.unit, r.unit) {
                (Some(u1), Some(mut u2)) => {
                    // Divide: l units divided by r units = l * r^-1
                    for part in &mut u2.parts {
                        part.power = -part.power;
                    }
                    let mut result_unit = u1;
                    for part in u2.parts {
                        result_unit.add(&part.name, part.power);
                    }
                    // Simplify if possible
                    if let Some(simplified) = units::simplify_unit(&result_unit) {
                        let mut new_unit = CompoundUnit::new();
                        new_unit.add(&simplified, 1);
                        Ok(Value::with_unit(result_num, new_unit))
                    } else {
                        Ok(Value::with_unit(result_num, result_unit))
                    }
                }
                (Some(u), None) => Ok(Value::with_unit(result_num, u)),
                (None, Some(mut u)) => {
                    // Dimensionless / unit = unit^-1
                    for part in &mut u.parts {
                        part.power = -part.power;
                    }
                    Ok(Value::with_unit(result_num, u))
                }
                (None, None) => Ok(Value::new(result_num)),
            }
        }
        BinaryOperator::Mod => {
            if r.number() == 0.0 {
                return Err(EvalError::DivisionByZero);
            }
            // Modulo requires same units
            if !l.dimensions_compatible(&r) {
                return Err(EvalError::DimensionalMismatch {
                    op: "modulo".to_string(),
                    left: l
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                    right: r
                        .get_unit_string()
                        .unwrap_or_else(|| "dimensionless".to_string()),
                });
            }
            match (&l.unit, &r.unit) {
                (Some(u1), Some(u2)) => {
                    let u1_str = u1.to_string();
                    let u2_str = u2.to_string();
                    let r_converted = units::convert(r.number(), &u2_str, &u1_str).map_err(|_| {
                        EvalError::IncompatibleUnits {
                            from: u2_str.clone(),
                            to: u1_str.clone(),
                        }
                    })?;
                    Ok(Value::with_unit(l.number() % r_converted, u1.clone()))
                }
                (Some(u), None) | (None, Some(u)) => {
                    Ok(Value::with_unit(l.number() % r.number(), u.clone()))
                }
                (None, None) => Ok(Value::new(l.number() % r.number())),
            }
        }
        BinaryOperator::Pow => {
            // Power must be dimensionless
            if r.unit.is_some() {
                return Err(EvalError::InvalidArgument {
                    func: "power".to_string(),
                    reason: "exponent must be dimensionless".to_string(),
                });
            }
            let exp = r.number();
            let result_num = if l.number() >= 0.0 || exp.fract() == 0.0 {
                l.number().powf(exp)
            } else {
                l.number().powf(exp) // Will produce NaN for invalid cases
            };
            match l.unit {
                Some(u) => {
                    // Unit^power: raise unit to power
                    let mut result_unit = CompoundUnit::new();
                    for part in u.parts {
                        result_unit.add(&part.name, part.power * exp as i8);
                    }
                    Ok(Value::with_unit(result_num, result_unit))
                }
                None => Ok(Value::new(result_num)),
            }
        }
    }
}

enum IntegrationMethod {
    Trapezoidal,
    Simpson,
    Rkf45,
}

fn eval_integration(
    args: &[Expr],
    env: &Environment,
    method: IntegrationMethod,
) -> Result<Value, EvalError> {
    use crate::core::integration;

    let min_args = match method {
        IntegrationMethod::Rkf45 => 3, // func, a, b (uses adaptive tolerance)
        _ => 4,                        // func, a, b, n
    };

    if args.len() < min_args {
        return Err(EvalError::ArgCount {
            name: match method {
                IntegrationMethod::Trapezoidal => "trapz".to_string(),
                IntegrationMethod::Simpson => "simpson".to_string(),
                IntegrationMethod::Rkf45 => "rkf45".to_string(),
            },
            expected: min_args,
            got: args.len(),
        });
    }

    // First argument: function expression
    let func_expr = &args[0];

    // Get bounds
    let a = evaluate(&args[1], env)?.number();
    let b = evaluate(&args[2], env)?.number();

    // Extract function body and parameter name
    let (func_body, param_name) = match func_expr {
        Expr::Identifier(name) => {
            // Try to get user-defined function
            if let Some(func) = env.get_function(name) {
                // Check if function has exactly one parameter
                if func.params.len() != 1 {
                    return Err(EvalError::InvalidArgument {
                        func: match method {
                            IntegrationMethod::Trapezoidal => "trapz".to_string(),
                            IntegrationMethod::Simpson => "simpson".to_string(),
                            IntegrationMethod::Rkf45 => "rkf45".to_string(),
                        },
                        reason: format!("Function '{}' has {} parameters, but integration requires exactly 1. Define a single-parameter function or use an expression like '{}'", 
                            name, func.params.len(),
                            if func.params.len() > 1 {
                                format!("{}->{}({})", func.params[0], name, func.params[0])
                            } else {
                                format!("x->{}", name)
                            }
                        ),
                    });
                }
                (func.body.clone(), func.params[0].clone())
            } else {
                // Assume it's an expression with x
                (func_expr.clone(), "x".to_string())
            }
        }
        Expr::FunctionCall { name: _, args: call_args } if call_args.len() == 1 => {
            if let Expr::Identifier(param) = &call_args[0] {
                (func_expr.clone(), param.clone())
            } else {
                return Err(EvalError::InvalidArgument {
                    func: match method {
                        IntegrationMethod::Trapezoidal => "trapz".to_string(),
                        IntegrationMethod::Simpson => "simpson".to_string(),
                        IntegrationMethod::Rkf45 => "rkf45".to_string(),
                    },
                    reason: "Expected parameter name".to_string(),
                });
            }
        }
        _ => {
            // Direct expression like "x^2"
            (func_expr.clone(), "x".to_string())
        }
    };

    // Perform integration
    let result = match method {
        IntegrationMethod::Trapezoidal => {
            let n = evaluate(&args[3], env)?.number() as usize;
            integration::trapezoidal(&func_body, &param_name, a, b, n, env)
                .map_err(|e| EvalError::InvalidArgument {
                    func: "trapz".to_string(),
                    reason: e,
                })?
        }
        IntegrationMethod::Simpson => {
            let n = evaluate(&args[3], env)?.number() as usize;
            // Make sure n is even
            let n = if n % 2 == 0 { n } else { n + 1 };
            integration::simpson(&func_body, &param_name, a, b, n, env)
                .map_err(|e| EvalError::InvalidArgument {
                    func: "simpson".to_string(),
                    reason: e,
                })?
        }
        IntegrationMethod::Rkf45 => {
            let tolerance = if args.len() >= 4 {
                evaluate(&args[3], env)?.number()
            } else {
                1e-6
            };
            let max_steps = if args.len() >= 5 {
                evaluate(&args[4], env)?.number() as usize
            } else {
                10000
            };
            integration::rkf45(&func_body, &param_name, a, b, tolerance, max_steps, env)
                .map_err(|e| EvalError::InvalidArgument {
                    func: "rkf45".to_string(),
                    reason: e,
                })?
        }
    };

    Ok(Value::new(result))
}

fn eval_quadratic(args: &[Expr], env: &Environment) -> Result<Value, EvalError> {
    if args.len() != 3 {
        return Err(EvalError::ArgCount {
            name: "quadratic".to_string(),
            expected: 3,
            got: args.len(),
        });
    }

    let a = evaluate(&args[0], env)?.number();
    let b = evaluate(&args[1], env)?.number();
    let c = evaluate(&args[2], env)?.number();

    if a == 0.0 {
        return Err(EvalError::InvalidArgument {
            func: "quadratic".to_string(),
            reason: "coefficient 'a' cannot be zero".to_string(),
        });
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return Err(EvalError::InvalidArgument {
            func: "quadratic".to_string(),
            reason: "complex roots not supported".to_string(),
        });
    }

    let sqrt_d = discriminant.sqrt();
    let x1 = (-b + sqrt_d) / (2.0 * a);
    let x2 = (-b - sqrt_d) / (2.0 * a);

    let formatted = format!("({}, {})", x1, x2);
    Ok(Value::new(x1).with_display_str(formatted))
}
