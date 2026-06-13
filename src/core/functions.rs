#[derive(Debug, Clone, thiserror::Error)]
pub enum FuncError {
    #[error("unknown function")]
    Unknown,
    #[error("invalid argument count: expected {expected}, got {got}")]
    ArgCount { expected: usize, got: usize },
    #[error("invalid argument: {0}")]
    InvalidArg(String),
}

pub struct ParamInfo {
    pub name: &'static str,
    pub description: &'static str,
}

pub struct FunctionInfo {
    pub name: &'static str,
    pub params: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub category: &'static str,
    pub params_detail: Vec<ParamInfo>,
}

pub fn list_functions() -> Vec<FunctionInfo> {
    vec![
        // Trigonometric functions
        FunctionInfo {
            name: "sin",
            params: "x",
            description: "Sine function (x in radians)",
            example: "sin(pi/2) = 1",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Angle in radians" }],
        },
        FunctionInfo {
            name: "cos",
            params: "x",
            description: "Cosine function (x in radians)",
            example: "cos(0) = 1",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Angle in radians" }],
        },
        FunctionInfo {
            name: "tan",
            params: "x",
            description: "Tangent function (x in radians)",
            example: "tan(pi/4) = 1",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Angle in radians" }],
        },
        FunctionInfo {
            name: "asin",
            params: "x",
            description: "Arc sine (inverse sine), returns radians",
            example: "asin(1) = 1.5708... (pi/2)",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Sine value in range [-1, 1]" }],
        },
        FunctionInfo {
            name: "acos",
            params: "x",
            description: "Arc cosine (inverse cosine), returns radians",
            example: "acos(1) = 0",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Cosine value in range [-1, 1]" }],
        },
        FunctionInfo {
            name: "atan",
            params: "x",
            description: "Arc tangent (inverse tangent), returns radians",
            example: "atan(1) = 0.7854... (pi/4)",
            category: "Trigonometry",
            params_detail: vec![ParamInfo { name: "x", description: "Tangent value" }],
        },
        // Math functions
        FunctionInfo {
            name: "sqrt",
            params: "x",
            description: "Square root of x",
            example: "sqrt(16) = 4",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Non-negative number" }],
        },
        FunctionInfo {
            name: "ln",
            params: "x",
            description: "Natural logarithm (base e)",
            example: "ln(e) = 1",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Positive number" }],
        },
        FunctionInfo {
            name: "log",
            params: "x",
            description: "Base-10 logarithm",
            example: "log(100) = 2",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Positive number" }],
        },
        FunctionInfo {
            name: "log10",
            params: "x",
            description: "Base-10 logarithm (same as log)",
            example: "log10(1000) = 3",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Positive number" }],
        },
        FunctionInfo {
            name: "exp",
            params: "x",
            description: "Exponential function e^x",
            example: "exp(1) = 2.718... (e)",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Exponent value" }],
        },
        FunctionInfo {
            name: "abs",
            params: "x",
            description: "Absolute value (magnitude without sign)",
            example: "abs(-5) = 5",
            category: "Math",
            params_detail: vec![ParamInfo { name: "x", description: "Any number" }],
        },
        // Rounding functions
        FunctionInfo {
            name: "floor",
            params: "x",
            description: "Round down to nearest integer",
            example: "floor(3.7) = 3",
            category: "Rounding",
            params_detail: vec![ParamInfo { name: "x", description: "Number to round down" }],
        },
        FunctionInfo {
            name: "ceil",
            params: "x",
            description: "Round up to nearest integer",
            example: "ceil(3.2) = 4",
            category: "Rounding",
            params_detail: vec![ParamInfo { name: "x", description: "Number to round up" }],
        },
        FunctionInfo {
            name: "round",
            params: "x",
            description: "Round to nearest integer",
            example: "round(3.5) = 4",
            category: "Rounding",
            params_detail: vec![ParamInfo { name: "x", description: "Number to round" }],
        },
        // Comparison functions
        FunctionInfo {
            name: "min",
            params: "a, b",
            description: "Minimum of two values",
            example: "min(5, 3) = 3",
            category: "Comparison",
            params_detail: vec![
                ParamInfo { name: "a", description: "First value" },
                ParamInfo { name: "b", description: "Second value" },
            ],
        },
        FunctionInfo {
            name: "max",
            params: "a, b",
            description: "Maximum of two values",
            example: "max(5, 3) = 5",
            category: "Comparison",
            params_detail: vec![
                ParamInfo { name: "a", description: "First value" },
                ParamInfo { name: "b", description: "Second value" },
            ],
        },
        // Power function
        FunctionInfo {
            name: "pow",
            params: "base, exp",
            description: "Raise base to the power of exp",
            example: "pow(2, 3) = 8 (same as 2^3)",
            category: "Math",
            params_detail: vec![
                ParamInfo { name: "base", description: "Base number" },
                ParamInfo { name: "exp", description: "Exponent" },
            ],
        },
        // Root function
        FunctionInfo {
            name: "root",
            params: "x, n",
            description: "N-th root of x (root(x, n) = x^(1/n))",
            example: "root(27, 3) = 3 (cube root of 27)",
            category: "Math",
            params_detail: vec![
                ParamInfo { name: "x", description: "The value to take the root of" },
                ParamInfo { name: "n", description: "The root exponent (e.g., 2 for square root, 3 for cube root)" },
            ],
        },
        // Quadratic equation solver
        FunctionInfo {
            name: "quadratic",
            params: "a, b, c",
            description: "Solve quadratic equation ax² + bx + c = 0. Returns (x1, x2) or (real, imaginary) parts",
            example: "quadratic(1, -5, 6) = (3, 2)",
            category: "Algebra",
            params_detail: vec![
                ParamInfo { name: "a", description: "Coefficient of x²" },
                ParamInfo { name: "b", description: "Coefficient of x" },
                ParamInfo { name: "c", description: "Constant term" },
            ],
        },
        // Integration functions
        FunctionInfo {
            name: "trapz",
            params: "f, a, b, n",
            description: "Numerical integration using trapezoidal rule",
            example: "trapz(x^2, 0, 1, 100) approximates integral of x² from 0 to 1",
            category: "Integration",
            params_detail: vec![
                ParamInfo { name: "f", description: "Function to integrate (expression or function name)" },
                ParamInfo { name: "a", description: "Lower bound of integration" },
                ParamInfo { name: "b", description: "Upper bound of integration" },
                ParamInfo { name: "n", description: "Number of intervals (higher = more accurate)" },
            ],
        },
        FunctionInfo {
            name: "simpson",
            params: "f, a, b, n",
            description: "Numerical integration using Simpson's rule (n must be even)",
            example: "simpson(x^2, 0, 1, 100) = 0.333... (exact for quadratics)",
            category: "Integration",
            params_detail: vec![
                ParamInfo { name: "f", description: "Function to integrate" },
                ParamInfo { name: "a", description: "Lower bound" },
                ParamInfo { name: "b", description: "Upper bound" },
                ParamInfo { name: "n", description: "Number of intervals (must be even)" },
            ],
        },
        FunctionInfo {
            name: "rkf45",
            params: "f, a, b, [tol], [max_steps]",
            description: "Adaptive integration using Runge-Kutta-Fehlberg method",
            example: "rkf45(sin(x), 0, pi) = 2.0 (high precision)",
            category: "Integration",
            params_detail: vec![
                ParamInfo { name: "f", description: "Function to integrate" },
                ParamInfo { name: "a", description: "Lower bound" },
                ParamInfo { name: "b", description: "Upper bound" },
                ParamInfo { name: "tol", description: "Tolerance (optional, default 1e-6)" },
                ParamInfo { name: "max_steps", description: "Maximum iterations (optional)" },
            ],
        },
    ]
}

pub fn call(name: &str, args: &[f64]) -> Result<f64, FuncError> {
    match name {
        "sin" => unary(args, f64::sin),
        "cos" => unary(args, f64::cos),
        "tan" => unary(args, f64::tan),
        "asin" => unary(args, f64::asin),
        "acos" => unary(args, f64::acos),
        "atan" => unary(args, f64::atan),
        "sqrt" => unary(args, |x| if x < 0.0 { f64::NAN } else { x.sqrt() }),
        "ln" => unary(args, f64::ln),
        "log" => unary(args, f64::log10),
        "log10" => unary(args, f64::log10),
        "exp" => unary(args, f64::exp),
        "abs" => unary(args, f64::abs),
        "floor" => unary(args, f64::floor),
        "ceil" => unary(args, f64::ceil),
        "round" => unary(args, f64::round),
        "min" => binary(args, f64::min),
        "max" => binary(args, f64::max),
        "pow" => binary(args, f64::powf),
        "root" => {
            if args.len() != 2 {
                return Err(FuncError::ArgCount {
                    expected: 2,
                    got: args.len(),
                });
            }
            let base = args[0];
            let n = args[1];
            if n == 0.0 {
                return Err(FuncError::InvalidArg("root exponent cannot be zero".to_string()));
            }
            if base < 0.0 && n.fract() != 0.0 {
                return Err(FuncError::InvalidArg("cannot take even root of negative number".to_string()));
            }
            Ok(base.powf(1.0 / n))
        }
        "quadratic" => {
            if args.len() != 3 {
                return Err(FuncError::ArgCount {
                    expected: 3,
                    got: args.len(),
                });
            }
            let a = args[0];
            let b = args[1];
            let c = args[2];
            if a == 0.0 {
                return Err(FuncError::InvalidArg("coefficient 'a' cannot be zero for quadratic equation".to_string()));
            }
            let discriminant = b * b - 4.0 * a * c;
            if discriminant < 0.0 {
                return Err(FuncError::InvalidArg("complex roots not supported, discriminant is negative".to_string()));
            }
            let sqrt_d = discriminant.sqrt();
            let x1 = (-b + sqrt_d) / (2.0 * a);
            Ok(x1)
        }
        _ => Err(FuncError::Unknown),
    }
}

fn unary(args: &[f64], f: fn(f64) -> f64) -> Result<f64, FuncError> {
    if args.len() != 1 {
        return Err(FuncError::ArgCount {
            expected: 1,
            got: args.len(),
        });
    }
    Ok(f(args[0]))
}

fn binary(args: &[f64], f: fn(f64, f64) -> f64) -> Result<f64, FuncError> {
    if args.len() != 2 {
        return Err(FuncError::ArgCount {
            expected: 2,
            got: args.len(),
        });
    }
    Ok(f(args[0], args[1]))
}

pub fn is_function(name: &str) -> bool {
    matches!(
        name,
        "sin"
            | "cos"
            | "tan"
            | "asin"
            | "acos"
            | "atan"
            | "sqrt"
            | "ln"
            | "log"
            | "log10"
            | "exp"
            | "abs"
            | "floor"
            | "ceil"
            | "round"
            | "min"
            | "max"
            | "pow"
            | "root"
            | "quadratic"
            | "trapz"
            | "simpson"
            | "rkf45"
    )
}

pub fn function_names() -> Vec<&'static str> {
    vec![
        "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "ln", "log", "log10", "exp", "abs",
        "floor", "ceil", "round", "min", "max", "pow", "root", "quadratic", "trapz", "simpson", "rkf45",
    ]
}

/// Get detailed info for a specific function
pub fn get_function_info(name: &str) -> Option<FunctionInfo> {
    list_functions().into_iter().find(|f| f.name == name)
}

/// Get functions by category
pub fn get_functions_by_category(category: &str) -> Vec<FunctionInfo> {
    list_functions()
        .into_iter()
        .filter(|f| f.category.eq_ignore_ascii_case(category))
        .collect()
}

/// Get all categories
pub fn get_categories() -> Vec<&'static str> {
    vec!["Trigonometry", "Math", "Rounding", "Comparison", "Integration"]
}
