use crate::types::{AppError, AppResult, RootEntry};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Var(char),
    Plus,
    Minus,
    Mul,
    Div,
    Caret,
    LParen,
    RParen,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Num(f64),
    Var(char),
    UnaryMinus(Box<Expr>),
    Binary(Box<Expr>, char, Box<Expr>),
}

pub struct Calculator;

impl Calculator {
    // =========================================================================
    // 1. ARITHMETIC EVALUATION (พร้อมแยก '=' vs '≈' และเศษส่วนแท้ p/q)
    // =========================================================================
    pub fn eval_expression(expr_str: &str) -> AppResult<(f64, bool, String)> {
        let ast = Self::parse(expr_str)?;
        let vars = Self::collect_vars(&ast);
        if !vars.is_empty() {
            return Err(AppError::InvalidInput(
                "Expression contains variables; use '=' to solve or 'diff:'/'int:'".to_string(),
            ));
        }
        let env = HashMap::new();
        let raw = Self::eval_ast(&ast, &env)?;
        if !raw.is_finite() {
            return Err(AppError::CalculationError("Result is not finite".to_string()));
        }
        let cleaned = Self::clean_float(raw);
        let is_term = Self::is_terminating_decimal(cleaned);
        let formatted = if is_term {
            format!("{} = {}", expr_str, Self::format_number(cleaned))
        } else if let Some((p, q)) = Self::to_rational(raw, 10_000) {
            format!(
                "{} = {}/{} (≈ {})",
                expr_str,
                p,
                q,
                Self::format_number(cleaned)
            )
        } else {
            format!("{} ≈ {}", expr_str, Self::format_number(cleaned))
        };
        Ok((cleaned, is_term, formatted))
    }

    // =========================================================================
    // 2. SINGLE EQUATION SOLVER (Linear, Polynomial, Exponential, Underdetermined)
    // =========================================================================
    pub fn solve_single_equation(lhs_str: &str, rhs_str: &str) -> AppResult<(Vec<RootEntry>, String)> {
        let lhs = Self::parse(lhs_str)?;
        let rhs = Self::parse(rhs_str)?;

        let mut vars = Self::collect_vars(&lhs);
        vars.extend(Self::collect_vars(&rhs));

        if vars.is_empty() {
            let env = HashMap::new();
            let l = Self::eval_ast(&lhs, &env)?;
            let r = Self::eval_ast(&rhs, &env)?;
            let eq = (l - r).abs() < 1e-9;
            return Ok((
                vec![],
                format!("{} = {}  ==>  {}", lhs_str, rhs_str, if eq { "TRUE (Identity)" } else { "FALSE (Contradiction)" }),
            ));
        }

        if vars.len() > 1 {
            return Self::solve_underdetermined_equation(lhs_str, rhs_str, &lhs, &rhs, &vars);
        }

        let var_char = *vars.iter().next().unwrap();
        let var_name = var_char.to_string();

        let eval_diff = |x: f64| -> Result<(f64, f64, f64), AppError> {
            let mut env = HashMap::new();
            env.insert(var_char, x);
            let l = Self::eval_ast(&lhs, &env)?;
            let r = Self::eval_ast(&rhs, &env)?;
            if !l.is_finite() || !r.is_finite() {
                return Err(AppError::CalculationError("Non-finite evaluation".to_string()));
            }
            Ok((l, r, l - r))
        };

        // ตรวจสอบสมการเชิงเส้น (Linear Equation)
        if let (Ok((_, _, y0)), Ok((_, _, y1)), Ok((_, _, y2)), Ok((_, _, yn1))) =
            (eval_diff(0.0), eval_diff(1.0), eval_diff(2.0), eval_diff(-1.0))
        {
            let s1 = y1 - y0;
            let s2 = y2 - y1;
            let s3 = y0 - yn1;
            if (s1 - s2).abs() < 1e-9 && (s1 - s3).abs() < 1e-9 && s1.abs() > 1e-12 {
                let raw_root = -y0 / s1;
                let entry = Self::build_verified_root_entry(&var_name, raw_root, &eval_diff);
                let formatted = format!(
                    "{} = {}  ==>  {}",
                    lhs_str,
                    rhs_str,
                    Self::format_root_entry(&entry)
                );
                return Ok((vec![entry], formatted));
            }
        }

        let mut results: Vec<RootEntry> = Vec::new();
        let mut push_entry = |entry: RootEntry| {
            if let Some(existing) = results
                .iter_mut()
                .find(|r| (r.value - entry.value).abs() < 1e-5)
            {
                if entry.is_exact && !existing.is_exact {
                    *existing = entry;
                }
            } else {
                results.push(entry);
            }
        };

        // สแกนรากจำนวนเต็มแม่นยำ (-200..=200)
        for k in -200..=200 {
            let x = k as f64;
            if let Ok((l, r, diff)) = eval_diff(x) {
                let scale = l.abs().max(r.abs()).max(1.0);
                if diff.abs() / scale < 1e-12 {
                    push_entry(RootEntry {
                        variable: var_name.clone(),
                        value: x,
                        is_exact: true,
                        rational_repr: None,
                    });
                }
            }
        }

        // สแกนรากทั่วไปด้วย Log-Space + High-Precision Bisection
        let eval_scaled = |x: f64| -> Option<f64> {
            let (l, r, diff) = eval_diff(x).ok()?;
            if l > 1.0 && r > 1.0 && (l > 1e6 || r > 1e6) {
                Some(l.ln() - r.ln())
            } else {
                let scale = l.abs().max(r.abs()).max(1.0);
                Some(diff / scale)
            }
        };

        let mut grid = Vec::with_capacity(4500);
        for i in -1000..=1000 {
            grid.push(i as f64 * 0.1);
        }
        for i in -500..=500 {
            grid.push(i as f64 * 0.01);
        }
        grid.sort_by(|a, b| a.partial_cmp(b).unwrap());
        grid.dedup_by(|a, b| (*a - *b).abs() < 1e-7);

        for window in grid.windows(2) {
            let (a, b) = (window[0], window[1]);
            if let (Some(fa), Some(fb)) = (eval_scaled(a), eval_scaled(b)) {
                if fa.abs() < 1e-12 {
                    push_entry(Self::build_verified_root_entry(&var_name, a, &eval_diff));
                } else if fa * fb < 0.0 {
                    let (mut lo, mut hi) = (a, b);
                    let mut flo = fa;
                    let mut valid = true;
                    for _ in 0..75 {
                        let mid = 0.5 * (lo + hi);
                        if let Some(fmid) = eval_scaled(mid) {
                            if fmid.abs() < 1e-15 {
                                lo = mid;
                                hi = mid;
                                break;
                            }
                            if flo * fmid < 0.0 {
                                hi = mid;
                            } else {
                                lo = mid;
                                flo = fmid;
                            }
                        } else {
                            valid = false;
                            break;
                        }
                    }
                    if valid {
                        let candidate = 0.5 * (lo + hi);
                        if let Some(fc) = eval_scaled(candidate) {
                            if fc.abs() < 1e-5 {
                                push_entry(Self::build_verified_root_entry(
                                    &var_name, candidate, &eval_diff,
                                ));
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());

        if results.is_empty() {
            return Err(AppError::CalculationError(
                "No real roots found in search domain".to_string(),
            ));
        }

        let formatted_list: Vec<String> = results.iter().map(Self::format_root_entry).collect();
        let summary = format!("{} = {}  ==>  {}", lhs_str, rhs_str, formatted_list.join(", "));
        Ok((results, summary))
    }

    // =========================================================================
    // 3. MULTIVARIATE SYSTEM SOLVER (Gaussian Elimination + Jacobian Newton-Raphson)
    // =========================================================================
    pub fn solve_system(equations: &[(String, String)]) -> AppResult<(Vec<RootEntry>, String)> {
        if equations.is_empty() {
            return Err(AppError::InvalidInput("No equations provided".to_string()));
        }

        let mut parsed_eqs = Vec::new();
        let mut var_set = BTreeSet::new();

        for (l_str, r_str) in equations {
            let l_ast = Self::parse(l_str)?;
            let r_ast = Self::parse(r_str)?;
            var_set.extend(Self::collect_vars(&l_ast));
            var_set.extend(Self::collect_vars(&r_ast));
            parsed_eqs.push((l_ast, r_ast));
        }

        let vars: Vec<char> = var_set.into_iter().collect();
        let n = vars.len();
        let m = parsed_eqs.len();

        if n == 0 {
            return Err(AppError::InvalidInput(
                "No variables found in system of equations".to_string(),
            ));
        }

        let zero_env: HashMap<char, f64> = vars.iter().map(|&v| (v, 0.0)).collect();
        let mut matrix = vec![vec![0.0_f64; n + 1]; m];
        let mut is_linear = true;

        // ตรวจสอบความเป็นเชิงเส้นทั้งในแต่ละตัวแปรและพจน์คูณไขว้ (Cross-terms เช่น x*y)
        for (i, (l_ast, r_ast)) in parsed_eqs.iter().enumerate() {
            let c0 = match (Self::eval_ast(l_ast, &zero_env), Self::eval_ast(r_ast, &zero_env)) {
                (Ok(lv), Ok(rv)) if lv.is_finite() && rv.is_finite() => lv - rv,
                _ => {
                    is_linear = false;
                    break;
                }
            };
            matrix[i][n] = -c0;

            for (j, &v) in vars.iter().enumerate() {
                let mut unit_env = zero_env.clone();
                unit_env.insert(v, 1.0);
                let f1 = Self::eval_ast(l_ast, &unit_env)? - Self::eval_ast(r_ast, &unit_env)?;
                let coeff = f1 - c0;

                let mut two_env = zero_env.clone();
                two_env.insert(v, 2.0);
                let f2 = Self::eval_ast(l_ast, &two_env)? - Self::eval_ast(r_ast, &two_env)?;
                if ((f2 - f1) - coeff).abs() > 1e-7 {
                    is_linear = false;
                    break;
                }
                matrix[i][j] = coeff;
            }

            if !is_linear {
                break;
            }

            // ตรวจจับพจน์คูณไขว้ เช่น x*y โดยแทนทุกตัวแปรเป็น 1.0 พร้อมกัน
            let all_ones_env: HashMap<char, f64> = vars.iter().map(|&v| (v, 1.0)).collect();
            let f_all = Self::eval_ast(l_ast, &all_ones_env)? - Self::eval_ast(r_ast, &all_ones_env)?;
            let expected_linear_sum: f64 = c0 + matrix[i][0..n].iter().sum::<f64>();
            if (f_all - expected_linear_sum).abs() > 1e-7 {
                is_linear = false;
                break;
            }
        }

        if !is_linear {
            return Self::solve_nonlinear_multivariate_newton(equations, &parsed_eqs, &vars);
        }

        if m < n {
            return Err(AppError::CalculationError(format!(
                "Underdetermined linear system: {} equations for {} variables ({:?})",
                m, n, vars
            )));
        }

        // Gaussian Elimination with Partial Pivoting
        let mut pivot_row = 0;
        for col in 0..n {
            if pivot_row >= m {
                break;
            }
            let mut max_row = pivot_row;
            let mut max_val = matrix[pivot_row][col].abs();
            for row in (pivot_row + 1)..m {
                if matrix[row][col].abs() > max_val {
                    max_val = matrix[row][col].abs();
                    max_row = row;
                }
            }

            if max_val < 1e-11 {
                return Err(AppError::CalculationError(
                    "System has no unique solution (singular or dependent equations)".to_string(),
                ));
            }

            matrix.swap(pivot_row, max_row);
            let pivot_val = matrix[pivot_row][col];
            for c in col..=n {
                matrix[pivot_row][c] /= pivot_val;
            }

            for row in 0..m {
                if row != pivot_row {
                    let factor = matrix[row][col];
                    if factor.abs() > 1e-14 {
                        for c in col..=n {
                            matrix[row][c] -= factor * matrix[pivot_row][c];
                        }
                    }
                }
            }
            pivot_row += 1;
        }

        for row in n..m {
            if matrix[row][n].abs() > 1e-7 {
                return Err(AppError::CalculationError(
                    "Inconsistent system of equations (no solution exists)".to_string(),
                ));
            }
        }

        let mut entries = Vec::with_capacity(n);
        for (idx, &v) in vars.iter().enumerate() {
            let raw_val = matrix[idx][n];
            let cleaned = Self::clean_float(raw_val);
            let is_term = Self::is_terminating_decimal(cleaned);
            let rational_repr = if !is_term {
                Self::to_rational(raw_val, 10_000).map(|(p, q)| format!("{}/{}", p, q))
            } else {
                None
            };
            entries.push(RootEntry {
                variable: v.to_string(),
                value: cleaned,
                is_exact: is_term || rational_repr.is_some(),
                rational_repr,
            });
        }

        let eq_title: Vec<String> = equations
            .iter()
            .map(|(l, r)| format!("{} = {}", l, r))
            .collect();
        let sol_text: Vec<String> = entries.iter().map(Self::format_root_entry).collect();
        let summary = format!("{{{}}}  ==>  {}", eq_title.join(", "), sol_text.join(", "));
        Ok((entries, summary))
    }

    /// Multidimensional Newton-Raphson with Numerical Jacobian Matrix สำหรับระบบสมการไม่เชิงเส้นหลายตัวแปร
    fn solve_nonlinear_multivariate_newton(
        equations: &[(String, String)],
        parsed_eqs: &[(Expr, Expr)],
        vars: &[char],
    ) -> AppResult<(Vec<RootEntry>, String)> {
        let n = vars.len();
        let m = parsed_eqs.len();
        if m < n {
            return Err(AppError::CalculationError(
                "Non-linear system requires at least as many equations as variables".to_string(),
            ));
        }

        let eval_vec = |x_vec: &[f64]| -> Option<Vec<f64>> {
            let env: HashMap<char, f64> = vars.iter().copied().zip(x_vec.iter().copied()).collect();
            let mut res = Vec::with_capacity(m);
            for (l_ast, r_ast) in parsed_eqs {
                let lv = Self::eval_ast(l_ast, &env).ok()?;
                let rv = Self::eval_ast(r_ast, &env).ok()?;
                if !lv.is_finite() || !rv.is_finite() {
                    return None;
                }
                res.push(lv - rv);
            }
            Some(res)
        };

        // สร้าง Multi-Start Seeds กระจายทั่วปริภูมิเพื่อหาคำตอบหลายชุด (เช่น จุดตัดวงกลมกับเส้นตรง 2 จุด)
        let mut seeds: Vec<Vec<f64>> = Vec::new();
        if n == 2 {
            let coords = [-10.0, -3.5, -1.5, -0.5, 0.5, 1.5, 3.5, 10.0];
            for &sx in &coords {
                for &sy in &coords {
                    seeds.push(vec![sx, sy]);
                }
            }
        } else if n == 3 {
            let coords = [-5.0, -1.5, 0.5, 2.5, 6.0];
            for &sx in &coords {
                for &sy in &coords {
                    for &sz in &coords {
                        seeds.push(vec![sx, sy, sz]);
                    }
                }
            }
        } else {
            seeds.push(vec![1.0; n]);
            seeds.push(vec![-1.0; n]);
            seeds.push(vec![3.0; n]);
        }

        let mut solution_sets: Vec<Vec<f64>> = Vec::new();

        for seed in seeds {
            let mut x = seed;
            let mut converged = false;

            for _iter in 0..60 {
                let Some(f_val) = eval_vec(&x) else { break };
                let max_err = f_val.iter().map(|v| v.abs()).fold(0.0_f64, f64::max);
                if max_err < 1e-12 {
                    converged = true;
                    break;
                }

                // สร้างเมทริกซ์จาโคเบียน (Jacobian Matrix J_ij = df_i / dx_j)
                let h = 1e-7;
                let mut aug = vec![vec![0.0_f64; n + 1]; n];
                let mut jac_ok = true;

                for i in 0..n {
                    aug[i][n] = -f_val[i];
                }

                for j in 0..n {
                    let mut x_plus = x.clone();
                    x_plus[j] += h;
                    let Some(f_plus) = eval_vec(&x_plus) else {
                        jac_ok = false;
                        break;
                    };
                    for i in 0..n {
                        aug[i][j] = (f_plus[i] - f_val[i]) / h;
                    }
                }

                if !jac_ok {
                    break;
                }

                // แก้ J * delta = -F
                if let Some(delta) = Self::solve_small_linear_system(aug, n) {
                    for j in 0..n {
                        x[j] += delta[j];
                    }
                } else {
                    break;
                }
            }

            if converged {
                let cleaned_vec: Vec<f64> = x.iter().map(|&v| Self::clean_float(v)).collect();
                if let Some(check_f) = eval_vec(&cleaned_vec) {
                    if check_f.iter().all(|v| v.abs() < 1e-7) {
                        let is_dup = solution_sets.iter().any(|existing| {
                            existing
                                .iter()
                                .zip(cleaned_vec.iter())
                                .all(|(a, b)| (a - b).abs() < 1e-4)
                        });
                        if !is_dup {
                            solution_sets.push(cleaned_vec);
                        }
                    }
                }
            }
        }

        if solution_sets.is_empty() {
            return Err(AppError::CalculationError(
                "No real solutions found for non-linear multivariate system".to_string(),
            ));
        }

        solution_sets.sort_by(|a, b| a[0].partial_cmp(&b[0]).unwrap());

        let mut first_entries = Vec::new();
        let mut tuple_strings = Vec::new();

        for sol in &solution_sets {
            let mut parts = Vec::new();
            for (idx, &val) in sol.iter().enumerate() {
                let vname = vars[idx].to_string();
                let is_term = Self::is_terminating_decimal(val);
                let rational_repr = if !is_term {
                    Self::to_rational(val, 10_000).map(|(p, q)| format!("{}/{}", p, q))
                } else {
                    None
                };
                // ตรวจสอบว่าค่าที่ปัดแล้วทำให้สมการเท่ากับศูนย์เป๊ะหรือไม่
                let is_exact = if is_term {
                    let env: HashMap<char, f64> =
                        vars.iter().copied().zip(sol.iter().copied()).collect();
                    parsed_eqs.iter().all(|(l, r)| {
                        if let (Ok(lv), Ok(rv)) = (Self::eval_ast(l, &env), Self::eval_ast(r, &env))
                        {
                            (lv - rv).abs() < 1e-11
                        } else {
                            false
                        }
                    })
                } else {
                    rational_repr.is_some()
                };

                let entry = RootEntry {
                    variable: vname,
                    value: val,
                    is_exact,
                    rational_repr,
                };
                parts.push(Self::format_root_entry(&entry));
                if first_entries.len() < n {
                    first_entries.push(entry);
                }
            }
            tuple_strings.push(format!("({})", parts.join(", ")));
        }

        let eq_title: Vec<String> = equations
            .iter()
            .map(|(l, r)| format!("{} = {}", l, r))
            .collect();
        let summary = format!(
            "{{{}}}  ==>  {}",
            eq_title.join(", "),
            tuple_strings.join(" OR ")
        );
        Ok((first_entries, summary))
    }

    fn solve_small_linear_system(mut aug: Vec<Vec<f64>>, n: usize) -> Option<Vec<f64>> {
        for col in 0..n {
            let mut max_row = col;
            let mut max_val = aug[col][col].abs();
            for row in (col + 1)..n {
                if aug[row][col].abs() > max_val {
                    max_val = aug[row][col].abs();
                    max_row = row;
                }
            }
            if max_val < 1e-12 {
                return None;
            }
            aug.swap(col, max_row);
            let piv = aug[col][col];
            for c in col..=n {
                aug[col][c] /= piv;
            }
            for row in 0..n {
                if row != col {
                    let fac = aug[row][col];
                    for c in col..=n {
                        aug[row][c] -= fac * aug[col][c];
                    }
                }
            }
        }
        Some((0..n).map(|i| aug[i][n]).collect())
    }

    // =========================================================================
    // 4. SYMBOLIC DIFFERENTIATION (d/dx) & DEFINITE INTEGRATION (Simpson's Rule)
    // =========================================================================
    pub fn symbolic_derivative(expr_str: &str, var: char) -> AppResult<String> {
        let ast = Self::parse(expr_str)?;
        // สกัดสัมประสิทธิ์พหุนามถ้าเป็นพหุนามเพื่อให้จัดรูปสวยงาม หรือใช้กฎลูกโซ่ AST
        if let Some(poly_str) = Self::try_polynomial_derivative(&ast, var) {
            return Ok(format!("d/d{} [{}] = {}", var, expr_str, poly_str));
        }
        let d_ast = Self::diff_ast(&ast, var);
        let simplified = Self::format_ast(&d_ast);
        Ok(format!("d/d{} [{}] = {}", var, expr_str, simplified))
    }

    pub fn definite_integral(
        expr_str: &str,
        var: char,
        lower: f64,
        upper: f64,
    ) -> AppResult<(f64, String)> {
        let ast = Self::parse(expr_str)?;
        let steps = 1000_usize; // Composite Simpson's Rule 1000 intervals
        let h = (upper - lower) / (steps as f64);

        let eval_at = |x: f64| -> AppResult<f64> {
            let mut env = HashMap::new();
            env.insert(var, x);
            Self::eval_ast(&ast, &env)
        };

        let mut sum = eval_at(lower)? + eval_at(upper)?;
        for i in 1..steps {
            let x = lower + (i as f64) * h;
            let weight = if i % 2 == 1 { 4.0 } else { 2.0 };
            sum += weight * eval_at(x)?;
        }
        let raw_area = (h / 3.0) * sum;
        let cleaned = Self::clean_float(raw_area);
        let is_term = Self::is_terminating_decimal(cleaned);
        let sym = if is_term { "=" } else { "≈" };
        let summary = format!(
            "∫({}, {}={}..{}) {} {}",
            expr_str,
            var,
            Self::format_number(lower),
            Self::format_number(upper),
            sym,
            Self::format_number(cleaned)
        );
        Ok((cleaned, summary))
    }

    // =========================================================================
    // 5. BOOLEAN SAT & PROPOSITIONAL LOGIC ENGINE (sat: ...)
    // =========================================================================
    pub fn solve_boolean_sat(formula: &str) -> AppResult<String> {
        let normalized = formula
            .replace("&&", "&")
            .replace("||", "|")
            .replace("<->", "=")
            .replace("->", ">")
            .replace(" AND ", " & ")
            .replace(" and ", " & ")
            .replace(" OR ", " | ")
            .replace(" or ", " | ")
            .replace(" NOT ", " ! ")
            .replace(" not ", " ! ")
            .replace(" XOR ", " ^ ")
            .replace(" xor ", " ^ ");

        let mut vars_set = BTreeSet::new();
        for c in normalized.chars() {
            if c.is_ascii_alphabetic() {
                vars_set.insert(c);
            }
        }
        let vars: Vec<char> = vars_set.into_iter().collect();
        if vars.is_empty() {
            return Err(AppError::InvalidInput(
                "No propositional variables found in logic formula".to_string(),
            ));
        }
        if vars.len() > 12 {
            return Err(AppError::InvalidInput(
                "Maximum 12 variables supported for exhaustive SAT truth-table verification".to_string(),
            ));
        }

        let total_rows = 1_usize << vars.len();
        let mut true_count = 0_usize;
        let mut first_sat_model: Option<Vec<(char, bool)>> = None;
        let mut first_counterexample: Option<Vec<(char, bool)>> = None;

        for mask in 0..total_rows {
            let mut env = HashMap::new();
            let mut assignment = Vec::with_capacity(vars.len());
            for (bit, &v) in vars.iter().enumerate() {
                let val = ((mask >> (vars.len() - 1 - bit)) & 1) == 1;
                env.insert(v, val);
                assignment.push((v, val));
            }
            let truth = Self::eval_bool_expr(&normalized, &env)?;
            if truth {
                true_count += 1;
                if first_sat_model.is_none() {
                    first_sat_model = Some(assignment);
                }
            } else if first_counterexample.is_none() {
                first_counterexample = Some(assignment);
            }
        }

        let format_model = |m: &[(char, bool)]| -> String {
            m.iter()
                .map(|(k, v)| format!("{}={}", k, if *v { "T" } else { "F" }))
                .collect::<Vec<_>>()
                .join(", ")
        };

        if true_count == total_rows {
            Ok(format!(
                "Formula [{}]  ==>  VALID / TAUTOLOGY (True for all {}/{} assignments)",
                formula.trim(),
                true_count,
                total_rows
            ))
        } else if true_count == 0 {
            Ok(format!(
                "Formula [{}]  ==>  UNSATISFIABLE / CONTRADICTION (0/{} assignments)",
                formula.trim(),
                total_rows
            ))
        } else {
            let sat_str = format_model(first_sat_model.as_ref().unwrap());
            let ce_str = format_model(first_counterexample.as_ref().unwrap());
            Ok(format!(
                "Formula [{}]  ==>  SATISFIABLE ({}/{} true) | Witness: ({}) | Counterexample: ({})",
                formula.trim(),
                true_count,
                total_rows,
                sat_str,
                ce_str
            ))
        }
    }

    // =========================================================================
    // HELPER FUNCTIONS (Rational Reconstruction, Formatting, Polynomial Diff)
    // =========================================================================
    fn build_verified_root_entry<F>(var_name: &str, raw_val: f64, eval_diff: &F) -> RootEntry
    where
        F: Fn(f64) -> Result<(f64, f64, f64), AppError>,
    {
        let cleaned = Self::clean_float(raw_val);
        if Self::is_terminating_decimal(cleaned) {
            if let Ok((l, r, diff)) = eval_diff(cleaned) {
                let scale = l.abs().max(r.abs()).max(1.0);
                if diff.abs() / scale < 1e-12 {
                    return RootEntry {
                        variable: var_name.to_string(),
                        value: cleaned,
                        is_exact: true,
                        rational_repr: None,
                    };
                }
            }
        }

        // ลองกู้คืนเศษส่วนแท้ p/q แล้วแทนค่าจริงลงใน AST
        if let Some((p, q)) = Self::to_rational(raw_val, 10_000) {
            let exact_frac = (p as f64) / (q as f64);
            if let Ok((l, r, diff)) = eval_diff(exact_frac) {
                let scale = l.abs().max(r.abs()).max(1.0);
                if diff.abs() / scale < 1e-12 {
                    return RootEntry {
                        variable: var_name.to_string(),
                        value: cleaned,
                        is_exact: true,
                        rational_repr: Some(format!("{}/{}", p, q)),
                    };
                }
            }
        }

        RootEntry {
            variable: var_name.to_string(),
            value: cleaned,
            is_exact: false,
            rational_repr: None,
        }
    }

    pub fn format_root_entry(entry: &RootEntry) -> String {
        if let Some(ref frac) = entry.rational_repr {
            format!(
                "{} = {} (≈ {})",
                entry.variable,
                frac,
                Self::format_number(entry.value)
            )
        } else {
            format!(
                "{} {} {}",
                entry.variable,
                entry.symbol(),
                Self::format_number(entry.value)
            )
        }
    }

    /// Continued Fractions Algorithm เพื่อแปลงทศนิยมซ้ำเป็นเศษส่วนอย่างต่ำ p/q
    pub fn to_rational(val: f64, max_denom: i64) -> Option<(i64, i64)> {
        if !val.is_finite() {
            return None;
        }
        let sign = if val < 0.0 { -1 } else { 1 };
        let mut x = val.abs();
        let (mut h1, mut h2) = (1_i64, 0_i64);
        let (mut k1, mut k2) = (0_i64, 1_i64);

        for _ in 0..24 {
            let a = x.floor() as i64;
            let h = a.checked_mul(h1)?.checked_add(h2)?;
            let k = a.checked_mul(k1)?.checked_add(k2)?;
            if k > max_denom {
                break;
            }
            h2 = h1;
            h1 = h;
            k2 = k1;
            k1 = k;

            let approx = (h1 as f64) / (k1 as f64);
            if (val.abs() - approx).abs() < 1e-11 {
                if k1 > 1 {
                    return Some((sign * h1, k1));
                } else {
                    return None;
                }
            }
            let rem = x - (a as f64);
            if rem.abs() < 1e-12 {
                break;
            }
            x = 1.0 / rem;
        }
        None
    }

    fn try_polynomial_derivative(ast: &Expr, var: char) -> Option<String> {
        let coeffs = Self::extract_poly_coeffs(ast, var)?;
        let mut terms = Vec::new();
        for (&deg, &coeff) in coeffs.iter().rev() {
            if deg == 0 {
                continue;
            }
            let d_coeff = Self::clean_float(coeff * (deg as f64));
            if d_coeff.abs() < 1e-12 {
                continue;
            }
            let new_deg = deg - 1;
            terms.push((new_deg, d_coeff));
        }
        if terms.is_empty() {
            return Some("0".to_string());
        }
        let mut out = String::new();
        for (idx, (deg, c)) in terms.into_iter().enumerate() {
            let is_neg = c < 0.0;
            let abs_c = c.abs();
            if idx == 0 {
                if is_neg {
                    out.push('-');
                }
            } else if is_neg {
                out.push_str(" - ");
            } else {
                out.push_str(" + ");
            }

            if deg == 0 {
                out.push_str(&Self::format_number(abs_c));
            } else {
                if (abs_c - 1.0).abs() > 1e-9 {
                    out.push_str(&Self::format_number(abs_c));
                }
                out.push(var);
                if deg > 1 {
                    out.push('^');
                    out.push_str(&deg.to_string());
                }
            }
        }
        Some(out)
    }

    fn extract_poly_coeffs(expr: &Expr, var: char) -> Option<BTreeMap<i32, f64>> {
        match expr {
            Expr::Num(n) => {
                let mut m = BTreeMap::new();
                m.insert(0, *n);
                Some(m)
            }
            Expr::Var(v) => {
                if *v != var {
                    return None;
                }
                let mut m = BTreeMap::new();
                m.insert(1, 1.0);
                Some(m)
            }
            Expr::UnaryMinus(inner) => {
                let mut m = Self::extract_poly_coeffs(inner, var)?;
                for val in m.values_mut() {
                    *val = -*val;
                }
                Some(m)
            }
            Expr::Binary(l, op, r) => match *op {
                '+' | '-' => {
                    let mut ml = Self::extract_poly_coeffs(l, var)?;
                    let mr = Self::extract_poly_coeffs(r, var)?;
                    let sign = if *op == '+' { 1.0 } else { -1.0 };
                    for (deg, c) in mr {
                        *ml.entry(deg).or_insert(0.0) += sign * c;
                    }
                    Some(ml)
                }
                '*' => {
                    let ml = Self::extract_poly_coeffs(l, var)?;
                    let mr = Self::extract_poly_coeffs(r, var)?;
                    let mut res = BTreeMap::new();
                    for (&d1, &c1) in &ml {
                        for (&d2, &c2) in &mr {
                            *res.entry(d1 + d2).or_insert(0.0) += c1 * c2;
                        }
                    }
                    Some(res)
                }
                '^' => {
                    if let Expr::Num(exp) = **r {
                        let exp_int = exp.round() as i32;
                        if (exp - (exp_int as f64)).abs() < 1e-9 && (0..=12).contains(&exp_int) {
                            let base = Self::extract_poly_coeffs(l, var)?;
                            let mut acc = BTreeMap::new();
                            acc.insert(0, 1.0);
                            for _ in 0..exp_int {
                                let mut next = BTreeMap::new();
                                for (&d1, &c1) in &acc {
                                    for (&d2, &c2) in &base {
                                        *next.entry(d1 + d2).or_insert(0.0) += c1 * c2;
                                    }
                                }
                                acc = next;
                            }
                            return Some(acc);
                        }
                    }
                    None
                }
                _ => None,
            },
        }
    }

    fn diff_ast(expr: &Expr, var: char) -> Expr {
        match expr {
            Expr::Num(_) => Expr::Num(0.0),
            Expr::Var(v) => {
                if *v == var {
                    Expr::Num(1.0)
                } else {
                    Expr::Num(0.0)
                }
            }
            Expr::UnaryMinus(inner) => Expr::UnaryMinus(Box::new(Self::diff_ast(inner, var))),
            Expr::Binary(l, op, r) => match *op {
                '+' | '-' => Expr::Binary(
                    Box::new(Self::diff_ast(l, var)),
                    *op,
                    Box::new(Self::diff_ast(r, var)),
                ),
                '*' => Expr::Binary(
                    Box::new(Expr::Binary(
                        Box::new(Self::diff_ast(l, var)),
                        '*',
                        r.clone(),
                    )),
                    '+',
                    Box::new(Expr::Binary(
                        l.clone(),
                        '*',
                        Box::new(Self::diff_ast(r, var)),
                    )),
                ),
                '^' => {
                    if let Expr::Num(n) = **r {
                        let coeff = Expr::Binary(
                            Box::new(Expr::Num(n)),
                            '*',
                            Box::new(Expr::Binary(
                                l.clone(),
                                '^',
                                Box::new(Expr::Num(n - 1.0)),
                            )),
                        );
                        Expr::Binary(Box::new(coeff), '*', Box::new(Self::diff_ast(l, var)))
                    } else {
                        Expr::Num(0.0)
                    }
                }
                _ => Expr::Num(0.0),
            },
        }
    }

    fn format_ast(expr: &Expr) -> String {
        match expr {
            Expr::Num(n) => Self::format_number(*n),
            Expr::Var(c) => c.to_string(),
            Expr::UnaryMinus(i) => format!("-({})", Self::format_ast(i)),
            Expr::Binary(l, op, r) => {
                format!("({} {} {})", Self::format_ast(l), op, Self::format_ast(r))
            }
        }
    }

    fn solve_underdetermined_equation(
        lhs_str: &str,
        rhs_str: &str,
        lhs: &Expr,
        rhs: &Expr,
        vars: &BTreeSet<char>,
    ) -> AppResult<(Vec<RootEntry>, String)> {
        let var_list: Vec<char> = vars.iter().copied().collect();
        if var_list.len() == 2 {
            let (vx, vy) = (var_list[0], var_list[1]);
            let mut int_pairs = Vec::new();
            let mut first_entries = Vec::new();

            for x in -20..=20 {
                for y in -20..=20 {
                    let mut env = HashMap::new();
                    env.insert(vx, x as f64);
                    env.insert(vy, y as f64);
                    if let (Ok(l), Ok(r)) = (Self::eval_ast(lhs, &env), Self::eval_ast(rhs, &env)) {
                        if (l - r).abs() < 1e-9 {
                            if first_entries.is_empty() && x >= 0 && y >= 0 {
                                first_entries.push(RootEntry {
                                    variable: vx.to_string(),
                                    value: x as f64,
                                    is_exact: true,
                                    rational_repr: None,
                                });
                                first_entries.push(RootEntry {
                                    variable: vy.to_string(),
                                    value: y as f64,
                                    is_exact: true,
                                    rational_repr: None,
                                });
                            }
                            if int_pairs.len() < 4 {
                                int_pairs.push(format!("({} = {}, {} = {})", vx, x, vy, y));
                            }
                        }
                    }
                }
            }

            let sample_msg = if int_pairs.is_empty() {
                "Infinite real solutions (specify 1 more equation for a unique point)".to_string()
            } else {
                format!(
                    "Infinite solutions; integer lattice points: {}",
                    int_pairs.join(", ")
                )
            };

            let summary = format!("{} = {}  ==>  {}", lhs_str, rhs_str, sample_msg);
            return Ok((first_entries, summary));
        }

        Ok((
            vec![],
            format!(
                "{} = {}  ==>  Underdetermined equation with {} variables ({:?})",
                lhs_str,
                rhs_str,
                var_list.len(),
                var_list
            ),
        ))
    }

    pub fn is_terminating_decimal(val: f64) -> bool {
        let scaled = val * 1_000_000.0;
        (scaled - scaled.round()).abs() < 1e-6
    }

    pub fn clean_float(val: f64) -> f64 {
        if (val - val.round()).abs() < 1e-9 {
            let r = val.round();
            return if r == -0.0 { 0.0 } else { r };
        }
        let factor = 1e10_f64;
        let cleaned = (val * factor).round() / factor;
        if cleaned == -0.0 { 0.0 } else { cleaned }
    }

    pub fn format_number(val: f64) -> String {
        let cleaned = Self::clean_float(val);
        if (cleaned - cleaned.round()).abs() < 1e-9 {
            format!("{:.0}", cleaned.round())
        } else {
            format!("{}", cleaned)
        }
    }

    fn collect_vars(expr: &Expr) -> BTreeSet<char> {
        let mut set = BTreeSet::new();
        match expr {
            Expr::Num(_) => {}
            Expr::Var(c) => {
                set.insert(*c);
            }
            Expr::UnaryMinus(inner) => {
                set.extend(Self::collect_vars(inner));
            }
            Expr::Binary(l, _, r) => {
                set.extend(Self::collect_vars(l));
                set.extend(Self::collect_vars(r));
            }
        }
        set
    }

    fn tokenize(input: &str) -> AppResult<Vec<Token>> {
        let mut raw_tokens = Vec::new();
        let chars: Vec<char> = input.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            let c = chars[i];
            if c.is_whitespace() {
                i += 1;
                continue;
            }
            if c.is_ascii_digit() || c == '.' {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num = num_str.parse::<f64>().map_err(|_| {
                    AppError::InvalidInput(format!("Invalid number: '{}'", num_str))
                })?;
                raw_tokens.push(Token::Num(num));
                continue;
            }
            if c.is_ascii_alphabetic() {
                raw_tokens.push(Token::Var(c.to_ascii_lowercase()));
                i += 1;
                continue;
            }
            match c {
                '+' => raw_tokens.push(Token::Plus),
                '-' | '−' => raw_tokens.push(Token::Minus),
                '*' | '×' => raw_tokens.push(Token::Mul),
                '/' | '÷' => raw_tokens.push(Token::Div),
                '^' => raw_tokens.push(Token::Caret),
                '(' => raw_tokens.push(Token::LParen),
                ')' => raw_tokens.push(Token::RParen),
                _ => {
                    return Err(AppError::InvalidInput(format!(
                        "Unsupported character in math expression: '{}'",
                        c
                    )))
                }
            }
            i += 1;
        }

        let mut tokens = Vec::new();
        for idx in 0..raw_tokens.len() {
            if idx > 0 {
                let prev = &raw_tokens[idx - 1];
                let curr = &raw_tokens[idx];
                let need_mul = matches!(
                    (prev, curr),
                    (Token::Num(_), Token::Var(_))
                        | (Token::Num(_), Token::LParen)
                        | (Token::Var(_), Token::Num(_))
                        | (Token::Var(_), Token::Var(_))
                        | (Token::Var(_), Token::LParen)
                        | (Token::RParen, Token::Num(_))
                        | (Token::RParen, Token::Var(_))
                        | (Token::RParen, Token::LParen)
                );
                if need_mul {
                    tokens.push(Token::Mul);
                }
            }
            tokens.push(raw_tokens[idx].clone());
        }

        Ok(tokens)
    }

    pub fn parse(input: &str) -> AppResult<Expr> {
        let tokens = Self::tokenize(input)?;
        if tokens.is_empty() {
            return Err(AppError::InvalidInput("Empty expression".to_string()));
        }
        let mut pos = 0;
        let expr = Self::parse_add_sub(&tokens, &mut pos)?;
        if pos < tokens.len() {
            return Err(AppError::InvalidInput(
                "Unexpected trailing tokens in expression".to_string(),
            ));
        }
        Ok(expr)
    }

    fn parse_add_sub(tokens: &[Token], pos: &mut usize) -> AppResult<Expr> {
        let mut node = Self::parse_mul_div(tokens, pos)?;
        while *pos < tokens.len() {
            let op = match tokens[*pos] {
                Token::Plus => '+',
                Token::Minus => '-',
                _ => break,
            };
            *pos += 1;
            let rhs = Self::parse_mul_div(tokens, pos)?;
            node = Expr::Binary(Box::new(node), op, Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_mul_div(tokens: &[Token], pos: &mut usize) -> AppResult<Expr> {
        let mut node = Self::parse_power(tokens, pos)?;
        while *pos < tokens.len() {
            let op = match tokens[*pos] {
                Token::Mul => '*',
                Token::Div => '/',
                _ => break,
            };
            *pos += 1;
            let rhs = Self::parse_power(tokens, pos)?;
            node = Expr::Binary(Box::new(node), op, Box::new(rhs));
        }
        Ok(node)
    }

    fn parse_power(tokens: &[Token], pos: &mut usize) -> AppResult<Expr> {
        let left = Self::parse_unary(tokens, pos)?;
        if *pos < tokens.len() && tokens[*pos] == Token::Caret {
            *pos += 1;
            let right = Self::parse_power(tokens, pos)?;
            return Ok(Expr::Binary(Box::new(left), '^', Box::new(right)));
        }
        Ok(left)
    }

    fn parse_unary(tokens: &[Token], pos: &mut usize) -> AppResult<Expr> {
        if *pos < tokens.len() {
            match tokens[*pos] {
                Token::Minus => {
                    *pos += 1;
                    let inner = Self::parse_unary(tokens, pos)?;
                    return Ok(Expr::UnaryMinus(Box::new(inner)));
                }
                Token::Plus => {
                    *pos += 1;
                    return Self::parse_unary(tokens, pos);
                }
                _ => {}
            }
        }
        Self::parse_primary(tokens, pos)
    }

    fn parse_primary(tokens: &[Token], pos: &mut usize) -> AppResult<Expr> {
        if *pos >= tokens.len() {
            return Err(AppError::InvalidInput(
                "Unexpected end of expression".to_string(),
            ));
        }
        match &tokens[*pos] {
            Token::Num(n) => {
                let val = *n;
                *pos += 1;
                Ok(Expr::Num(val))
            }
            Token::Var(c) => {
                let ch = *c;
                *pos += 1;
                Ok(Expr::Var(ch))
            }
            Token::LParen => {
                *pos += 1;
                let expr = Self::parse_add_sub(tokens, pos)?;
                if *pos >= tokens.len() || tokens[*pos] != Token::RParen {
                    return Err(AppError::InvalidInput("Missing closing parenthesis".to_string()));
                }
                *pos += 1;
                Ok(expr)
            }
            other => Err(AppError::InvalidInput(format!(
                "Unexpected token: {:?}",
                other
            ))),
        }
    }

    fn eval_ast(expr: &Expr, env: &HashMap<char, f64>) -> AppResult<f64> {
        match expr {
            Expr::Num(n) => Ok(*n),
            Expr::Var(c) => env.get(c).copied().ok_or_else(|| {
                AppError::InvalidInput(format!("Unbound variable '{}'", c))
            }),
            Expr::UnaryMinus(inner) => Ok(-Self::eval_ast(inner, env)?),
            Expr::Binary(lhs, op, rhs) => {
                let l = Self::eval_ast(lhs, env)?;
                let r = Self::eval_ast(rhs, env)?;
                match *op {
                    '+' => Ok(l + r),
                    '-' => Ok(l - r),
                    '*' => Ok(l * r),
                    '/' => {
                        if r.abs() < 1e-15 {
                            Err(AppError::CalculationError("Division by zero".to_string()))
                        } else {
                            Ok(l / r)
                        }
                    }
                    '^' => Ok(l.powf(r)),
                    _ => Err(AppError::CalculationError("Unknown operator".to_string())),
                }
            }
        }
    }

    // Boolean Recursive Descent Evaluator
    fn eval_bool_expr(s: &str, env: &HashMap<char, bool>) -> AppResult<bool> {
        let chars: Vec<char> = s.chars().filter(|c| !c.is_whitespace()).collect();
        let mut pos = 0;
        let val = Self::parse_bool_equiv(&chars, &mut pos, env)?;
        if pos < chars.len() {
            return Err(AppError::InvalidInput(
                "Unexpected trailing tokens in boolean formula".to_string(),
            ));
        }
        Ok(val)
    }

    fn parse_bool_equiv(
        chars: &[char],
        pos: &mut usize,
        env: &HashMap<char, bool>,
    ) -> AppResult<bool> {
        let mut left = Self::parse_bool_implies(chars, pos, env)?;
        while *pos < chars.len() && chars[*pos] == '=' {
            *pos += 1;
            let right = Self::parse_bool_implies(chars, pos, env)?;
            left = left == right;
        }
        Ok(left)
    }

    fn parse_bool_implies(
        chars: &[char],
        pos: &mut usize,
        env: &HashMap<char, bool>,
    ) -> AppResult<bool> {
        let mut left = Self::parse_bool_or(chars, pos, env)?;
        while *pos < chars.len() && chars[*pos] == '>' {
            *pos += 1;
            let right = Self::parse_bool_or(chars, pos, env)?;
            left = (!left) || right;
        }
        Ok(left)
    }

    fn parse_bool_or(
        chars: &[char],
        pos: &mut usize,
        env: &HashMap<char, bool>,
    ) -> AppResult<bool> {
        let mut left = Self::parse_bool_and(chars, pos, env)?;
        while *pos < chars.len() && (chars[*pos] == '|' || chars[*pos] == '^') {
            let op = chars[*pos];
            *pos += 1;
            let right = Self::parse_bool_and(chars, pos, env)?;
            left = if op == '|' { left || right } else { left ^ right };
        }
        Ok(left)
    }

    fn parse_bool_and(
        chars: &[char],
        pos: &mut usize,
        env: &HashMap<char, bool>,
    ) -> AppResult<bool> {
        let mut left = Self::parse_bool_unary(chars, pos, env)?;
        while *pos < chars.len() && chars[*pos] == '&' {
            *pos += 1;
            let right = Self::parse_bool_unary(chars, pos, env)?;
            left = left && right;
        }
        Ok(left)
    }

    fn parse_bool_unary(
        chars: &[char],
        pos: &mut usize,
        env: &HashMap<char, bool>,
    ) -> AppResult<bool> {
        if *pos < chars.len() && (chars[*pos] == '!' || chars[*pos] == '~') {
            *pos += 1;
            return Ok(!Self::parse_bool_unary(chars, pos, env)?);
        }
        if *pos >= chars.len() {
            return Err(AppError::InvalidInput("Unexpected end of boolean formula".to_string()));
        }
        let c = chars[*pos];
        if c == '(' {
            *pos += 1;
            let val = Self::parse_bool_equiv(chars, pos, env)?;
            if *pos >= chars.len() || chars[*pos] != ')' {
                return Err(AppError::InvalidInput("Missing ')' in boolean formula".to_string()));
            }
            *pos += 1;
            return Ok(val);
        }
        if c.is_ascii_alphabetic() {
            *pos += 1;
            return env
                .get(&c)
                .copied()
                .ok_or_else(|| AppError::InvalidInput(format!("Unknown variable '{}'", c)));
        }
        Err(AppError::InvalidInput(format!(
            "Invalid character '{}' in boolean formula",
            c
        )))
    }
}