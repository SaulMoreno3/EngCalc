use crate::core::env::Environment;
use crate::core::formatter;
use crate::core::parser;

#[allow(dead_code)]
fn eval(input: &str) -> Result<String, String> {
    let ast = parser::parse(input).map_err(|e| e.to_string())?;
    let mut env = Environment::new();

    // Pre-populate with constants
    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }

    match ast.eval(&env) {
        Ok(value) => Ok(formatter::format_value(&value)),
        Err(e) => Err(e.to_string()),
    }
}

#[allow(dead_code)]
fn eval_with_env(input: &str) -> Result<(String, Environment), String> {
    let ast = parser::parse(input).map_err(|e| e.to_string())?;
    let mut env = Environment::new();

    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }

    match ast.eval(&env) {
        Ok(value) => {
            if let Some((name, val_expr)) = ast.as_assignment() {
                let val = val_expr.eval(&env).map_err(|e| e.to_string())?;
                env.set(name.to_string(), val.clone());
            }
            Ok((formatter::format_value(&value), env))
        }
        Err(e) => Err(e.to_string()),
    }
}

#[allow(dead_code)]
fn assert_result(input: &str, expected: &str) {
    let result = eval(input).expect(&format!("Expected success for '{}', but got error", input));
    assert_eq!(
        result, expected,
        "For '{}': expected '{}', got '{}'",
        input, expected, result
    );
}

#[allow(dead_code)]
fn assert_approx(input: &str, expected: f64, tolerance: f64) {
    let result = eval(input).expect(&format!("Expected success for '{}', but got error", input));
    assert_formatted_approx(&result, expected, tolerance);
}

#[allow(dead_code)]
fn assert_formatted_approx(result: &str, expected: f64, tolerance: f64) {
    let numeric_part = result.split_whitespace().next().unwrap_or(&result);
    let num: f64 = numeric_part
        .parse()
        .unwrap_or_else(|_| panic!("Failed to parse result '{}' as f64", result));
    assert!(
        (num - expected).abs() < tolerance,
        "expected ~{}, got {}",
        expected,
        result
    );
}

#[allow(dead_code)]
fn assert_error(input: &str) {
    let result = eval(input);
    assert!(
        result.is_err(),
        "Expected error for '{}', but got: {:?}",
        input,
        result
    );
}

#[test]
fn test_basic_addition() {
    assert_result("2 + 2", "4");
}

#[test]
fn test_basic_subtraction() {
    assert_result("10 - 3", "7");
}

#[test]
fn test_basic_multiplication() {
    assert_result("3 * 4", "12");
}

#[test]
fn test_basic_division() {
    assert_result("15 / 3", "5");
}

#[test]
fn test_precedence_mul_before_add() {
    assert_result("2 + 3 * 4", "14");
}

#[test]
fn test_parentheses_override_precedence() {
    assert_result("(2 + 3) * 4", "20");
}

#[test]
fn test_power() {
    assert_result("2^3", "8");
}

#[test]
fn test_power_right_associative() {
    assert_result("2^3^2", "512");
}

#[test]
fn test_unary_minus_power() {
    assert_result("-2^2", "-4");
}

#[test]
fn test_unary_minus_parentheses() {
    assert_result("(-2)^2", "4");
}

#[test]
fn test_modulo() {
    assert_result("10 % 3", "1");
}

#[test]
fn test_implicit_multiplication_number_variable() {
    let (result, env) = eval_with_env("x = 5").unwrap();
    assert_eq!(result, "5");

    let ast = parser::parse("2x").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "10");
}

#[test]
fn test_implicit_multiplication_number_parentheses() {
    assert_result("2(3 + 4)", "14");
}

#[test]
fn test_implicit_multiplication_parentheses_parentheses() {
    assert_result("(2 + 3)(4 + 5)", "45");
}

#[test]
fn test_sin_pi_over_2() {
    assert_approx("sin(pi / 2)", 1.0, 1e-10);
}

#[test]
fn test_cos_zero() {
    assert_approx("cos(0)", 1.0, 1e-10);
}

#[test]
fn test_sqrt() {
    assert_result("sqrt(16)", "4");
}

#[test]
fn test_ln_e() {
    assert_approx("ln(e)", 1.0, 1e-10);
}

#[test]
fn test_log10() {
    assert_result("log10(1000)", "3");
}

#[test]
fn test_exp() {
    assert_approx("exp(0)", 1.0, 1e-10);
}

#[test]
fn test_abs() {
    assert_result("abs(-5)", "5");
}

#[test]
fn test_floor() {
    assert_result("floor(3.7)", "3");
}

#[test]
fn test_ceil() {
    assert_result("ceil(3.2)", "4");
}

#[test]
fn test_round() {
    assert_result("round(3.6)", "4");
}

#[test]
fn test_min_max() {
    assert_result("min(3, 7)", "3");
    assert_result("max(3, 7)", "7");
}

#[test]
fn test_pow_function() {
    assert_result("pow(2, 3)", "8");
}

#[test]
fn test_constant_pi() {
    assert_approx("pi", 3.141592653589793, 1e-10);
}

#[test]
fn test_constant_e() {
    assert_approx("e", 2.718281828459045, 1e-10);
}

#[test]
fn test_constant_tau() {
    assert_approx("tau", 6.283185307179586, 1e-10);
}

#[test]
fn test_variable_assignment_and_use() {
    let (_, env) = eval_with_env("x = 5").unwrap();
    let ast = parser::parse("2x + 3").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "13");
}

#[test]
fn test_variable_chain() {
    let (_, env) = eval_with_env("r = 3").unwrap();
    let ast = parser::parse("pi * r^2").unwrap();
    let val = ast.eval(&env).unwrap();
    assert_formatted_approx(&formatter::format_value(&val), 28.2743338823, 1e-6);
}

#[test]
fn test_unit_km_to_m() {
    let ast = parser::parse("10 km in m").unwrap();
    let mut env = Environment::new();
    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "10000 m");
}

#[test]
fn test_unit_h_to_s() {
    let ast = parser::parse("1 h in s").unwrap();
    let mut env = Environment::new();
    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "3600 s");
}

#[test]
fn test_unit_bar_to_pa() {
    let ast = parser::parse("5 bar in Pa").unwrap();
    let mut env = Environment::new();
    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }
    let val = ast.eval(&env).unwrap();
    assert_eq!(formatter::format_value(&val), "500000 Pa");
}

#[test]
fn test_division_by_zero() {
    assert_error("1 / 0");
}

#[test]
fn test_unknown_function() {
    assert_error("foo(3)");
}

#[test]
fn test_unknown_variable() {
    assert_error("x");
}

#[test]
fn test_unmatched_parentheses() {
    assert_error("(2 + 3");
}

#[test]
fn test_leading_power_operator_errors() {
    assert_error("^3");
    assert_error("^-");
}

#[test]
fn test_formatter_integer_display() {
    let val = crate::core::value::Value::new(12.0);
    assert_eq!(formatter::format_value(&val), "12");
}

#[test]
fn test_formatter_decimal_display() {
    let val = crate::core::value::Value::new(3.14159);
    let result = formatter::format_value(&val);
    assert!(result.starts_with("3.14"));
    assert!(result.contains("(3.141590000000e0)"));
}

#[test]
fn test_formatter_short_decimal_display() {
    let val = crate::core::value::Value::new(3.7);
    assert_eq!(formatter::format_value(&val), "3.7");
}

#[test]
fn test_formatter_unit_decimal_dual_notation() {
    let unit = crate::core::units::parse_compound_unit("m/s").unwrap();
    let val = crate::core::value::Value::with_unit(0.000333333333, unit);
    let result = formatter::format_value(&val);

    assert_eq!(result, "0.000333333333 (3.333333330000e-4) m/s");
}

#[test]
fn test_formatter_nan() {
    let val = crate::core::value::Value::new(f64::NAN);
    assert_eq!(formatter::format_value(&val), "NaN");
}

#[test]
fn test_formatter_infinity() {
    let val = crate::core::value::Value::new(f64::INFINITY);
    assert_eq!(formatter::format_value(&val), "∞");

    let val = crate::core::value::Value::new(f64::NEG_INFINITY);
    assert_eq!(formatter::format_value(&val), "-∞");
}

#[test]
fn test_engineering_constants() {
    assert_approx("R", 8.314462618, 1e-6);
    assert_approx("g", 9.80665, 1e-6);
    assert_approx("atm", 101325.0, 1e-3);
}

#[test]
fn test_complex_expression() {
    assert_approx("sin(pi / 4) * sqrt(2)", 1.0, 1e-10);
}

#[test]
fn test_nested_functions() {
    assert_approx("sin(cos(0))", 0.8414709848, 1e-10);
}

#[test]
fn test_scientific_notation_uppercase() {
    assert_approx("1E5", 100000.0, 1.0);
}

#[test]
fn test_scientific_notation_lowercase() {
    assert_approx("1.5e3", 1500.0, 1.0);
}

#[test]
fn test_scientific_notation_negative_exp() {
    assert_approx("1e-3", 0.001, 1e-10);
}

#[test]
fn test_scientific_notation_positive_exp() {
    assert_approx("2.5e+2", 250.0, 1.0);
}

#[test]
fn test_scientific_notation_math() {
    assert_approx("2e3 + 1e2", 2100.0, 1.0);
}

#[test]
fn test_compound_unit_velocity() {
    // 36 km/h = 10 m/s
    let result = eval("36 km/h in m/s").unwrap();
    assert!(
        result.contains("10") && result.contains("m/s"),
        "Expected '10 m/s', got '{}'",
        result
    );
}

#[test]
fn test_dimensional_mismatch() {
    // Cannot add m + s
    let result = eval("10 m + 5 s");
    assert!(result.is_err(), "Should error on dimensional mismatch");
}

#[test]
fn test_multiply_units() {
    // 10 m * 5 m = 50 m²
    let result = eval("10 m * 5 m").unwrap();
    assert!(
        result.contains("50") && result.contains("m"),
        "Expected '50 m²', got '{}'",
        result
    );
}

#[test]
fn test_force_units() {
    // 1 kg * 1 m/s² = 1 N
    let result = eval("1 kg * 1 m/s^2").unwrap();
    assert!(
        result.contains("1") && result.contains("N"),
        "Expected '1 N', got '{}'",
        result
    );
}

#[test]
fn test_newtons_equivalence() {
    // 1 N should equal 1 kg·m/s²
    let result = eval("1 N in kg*m/s^2").unwrap();
    assert!(
        result.contains("1") && (result.contains("kg") || result.contains("m")),
        "Expected conversion to kg·m/s², got '{}'",
        result
    );
}

#[test]
fn test_user_function_single_param() {
    // Define f(x) = x^2 + 1, then call f(5)
    let mut env = Environment::new();
    use crate::core::ast::{BinaryOperator, Expr};
    use crate::core::env::UserFunction;

    // f(x) = x^2 + 1
    let func = UserFunction {
        name: "f".to_string(),
        params: vec!["x".to_string()],
        body: Expr::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expr::BinaryOp {
                op: BinaryOperator::Pow,
                left: Box::new(Expr::Identifier("x".to_string())),
                right: Box::new(Expr::Number(2.0)),
            }),
            right: Box::new(Expr::Number(1.0)),
        },
    };
    env.set_function(func);

    // Call f(5)
    let call = Expr::FunctionCall {
        name: "f".to_string(),
        args: vec![Expr::Number(5.0)],
    };

    let result = call.eval(&env).unwrap();
    assert!(
        (result.number() - 26.0).abs() < 1e-10,
        "Expected 26, got {}",
        result.number()
    );
}

#[test]
fn test_user_function_two_params() {
    // Define add(a, b) = a + b
    let mut env = Environment::new();
    use crate::core::ast::{BinaryOperator, Expr};
    use crate::core::env::UserFunction;

    let func = UserFunction {
        name: "add".to_string(),
        params: vec!["a".to_string(), "b".to_string()],
        body: Expr::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expr::Identifier("a".to_string())),
            right: Box::new(Expr::Identifier("b".to_string())),
        },
    };
    env.set_function(func);

    // Call add(3, 4)
    let call = Expr::FunctionCall {
        name: "add".to_string(),
        args: vec![Expr::Number(3.0), Expr::Number(4.0)],
    };

    let result = call.eval(&env).unwrap();
    assert!(
        (result.number() - 7.0).abs() < 1e-10,
        "Expected 7, got {}",
        result.number()
    );
}

#[test]
fn test_user_function_uses_constants() {
    // f(x) = pi * x
    let mut env = Environment::new();
    use crate::core::ast::{BinaryOperator, Expr};
    use crate::core::env::UserFunction;

    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }

    let func = UserFunction {
        name: "circle_area".to_string(),
        params: vec!["r".to_string()],
        body: Expr::BinaryOp {
            op: BinaryOperator::Mul,
            left: Box::new(Expr::Identifier("pi".to_string())),
            right: Box::new(Expr::BinaryOp {
                op: BinaryOperator::Pow,
                left: Box::new(Expr::Identifier("r".to_string())),
                right: Box::new(Expr::Number(2.0)),
            }),
        },
    };
    env.set_function(func);

    // Call circle_area(1)
    let call = Expr::FunctionCall {
        name: "circle_area".to_string(),
        args: vec![Expr::Number(1.0)],
    };

    let result = call.eval(&env).unwrap();
    assert!(
        (result.number() - std::f64::consts::PI).abs() < 1e-10,
        "Expected pi, got {}",
        result.number()
    );
}

#[test]
fn test_integration_with_user_function() {
    // Define f(x) = x^2, then integrate it
    use crate::core::ast::{Expr, BinaryOperator};
    use crate::core::env::UserFunction;
    use crate::core::integration;

    let mut env = Environment::new();

    // f(x) = x^2
    let func = UserFunction {
        name: "square".to_string(),
        params: vec!["x".to_string()],
        body: Expr::BinaryOp {
            op: BinaryOperator::Pow,
            left: Box::new(Expr::Identifier("x".to_string())),
            right: Box::new(Expr::Number(2.0)),
        },
    };
    env.set_function(func);

    // Integrate square(x) from 0 to 1 using Simpson's rule
    let func_body = Expr::FunctionCall {
        name: "square".to_string(),
        args: vec![Expr::Identifier("x".to_string())],
    };

    let result = integration::simpson(&func_body, "x", 0.0, 1.0, 100, &env).unwrap();
    assert!(
        (result - 1.0 / 3.0).abs() < 1e-10,
        "Expected 1/3, got {}",
        result
    );
}

#[test]
fn test_integration_with_expression() {
    // Test that we can integrate expressions directly without defining a function
    use crate::core::ast::{Expr, BinaryOperator};
    use crate::core::integration;

    let env = Environment::new();

    // Integrate x^2 expression directly
    let func_body = Expr::BinaryOp {
        op: BinaryOperator::Pow,
        left: Box::new(Expr::Identifier("x".to_string())),
        right: Box::new(Expr::Number(2.0)),
    };

    let result = integration::simpson(&func_body, "x", 0.0, 1.0, 100, &env).unwrap();
    assert!(
        (result - 1.0 / 3.0).abs() < 1e-10,
        "Expected 1/3, got {}",
        result
    );
}

#[test]
fn test_quadratic_two_real_roots() {
    assert_result("quadratic(1, -5, 6)", "(3, 2)");
}

#[test]
fn test_quadratic_single_root() {
    assert_result("quadratic(1, -2, 1)", "(1, 1)");
}

#[test]
fn test_quadratic_negative_discriminant_error() {
    assert_error("quadratic(1, 1, 1)");
}

#[test]
fn test_quadratic_zero_a_error() {
    assert_error("quadratic(0, 1, 1)");
}

#[test]
fn test_quadratic_ans_takes_positive_root() {
    let mut env = Environment::new();
    for c in crate::core::constants::list() {
        env.set(c.name.to_string(), crate::core::value::Value::new(c.value));
    }
    let ast = parser::parse("quadratic(1, -5, 6)").unwrap();
    let value = ast.eval(&env).unwrap();
    env.set("ans".to_string(), value.clone());
    let ans_val = env.get("ans").expect("ans should be set");
    assert_eq!(ans_val.number(), 3.0);
    let formatted = formatter::format_value(&value);
    assert_eq!(formatted, "(3, 2)");
}
