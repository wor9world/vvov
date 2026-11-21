use crate::types::{AppError, AppResult, IntentType};
use std::collections::BTreeSet;

pub struct IntentClassifier;

impl IntentClassifier {
    pub fn classify(input: &str) -> AppResult<IntentType> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(AppError::InvalidInput("Query cannot be empty".to_string()));
        }

        if trimmed.eq_ignore_ascii_case("ping") || trimmed.eq_ignore_ascii_case("health") {
            return Ok(IntentType::HealthCheck);
        }

        // 1. ตรวจจับคำสั่งอนุพันธ์เชิงสัญลักษณ์ (เช่น "diff: 4x^3 - 5x^2 + 2x - 9" หรือ "ดิฟ 4x^3 - 5x^2")
        if let Some(rest) = Self::strip_any_prefix(trimmed, &["diff:", "DIFF:", "d/dx:", "ดิฟ"]) {
            return Ok(IntentType::SymbolicDiff {
                expr: rest.to_string(),
                var: 'x',
            });
        }

        // 2. ตรวจจับคำสั่งอินทิเกรตจำกัดเขต (เช่น "int: x^2, 0, 3" หรือ "อินทิเกรต x^2, 0, 3")
        if let Some(rest) = Self::strip_any_prefix(trimmed, &["int:", "INT:", "integral:", "อินทิเกรต"]) {
            let parts: Vec<&str> = rest.split(',').map(str::trim).collect();
            if parts.len() == 3 {
                let lower = parts[1].parse::<f64>().map_err(|_| {
                    AppError::InvalidInput(format!("Invalid lower bound: '{}'", parts[1]))
                })?;
                let upper = parts[2].parse::<f64>().map_err(|_| {
                    AppError::InvalidInput(format!("Invalid upper bound: '{}'", parts[2]))
                })?;
                return Ok(IntentType::DefiniteIntegral {
                    expr: parts[0].to_string(),
                    var: 'x',
                    lower,
                    upper,
                });
            }
        }

        // 3. ตรวจจับคำสั่งพิสูจน์ตรรกศาสตร์ SAT (เช่น "sat: (A & B) | !A")
        if let Some(rest) = Self::strip_any_prefix(trimmed, &["sat:", "SAT:", "logic:", "ตรรกะ:"]) {
            return Ok(IntentType::BooleanSat {
                formula: rest.to_string(),
            });
        }

        let cleaned = Self::strip_natural_prefixes(trimmed);

        let candidates: Vec<String> = cleaned
            .split(|c| c == ';' || c == ',')
            .flat_map(|part| Self::split_multi_math_tokens(part))
            .filter(|s| !s.is_empty())
            .collect();

        if candidates.len() > 1 && candidates.iter().all(|c| Self::looks_like_math(c)) {
            if Self::is_coupled_multivariate_system(&candidates) {
                let mut equations = Vec::new();
                for item in &candidates {
                    if let Some((l, r)) = item.split_once('=') {
                        equations.push((l.trim().to_string(), r.trim().to_string()));
                    }
                }
                return Ok(IntentType::SolveSystem { equations });
            }
            return Ok(IntentType::BatchProcess { items: candidates });
        }

        let target = if candidates.len() == 1 {
            candidates[0].as_str()
        } else {
            cleaned.as_str()
        };

        if let Some((lhs, rhs)) = target.split_once('=') {
            let l = lhs.trim();
            let r = rhs.trim();
            if Self::looks_like_math(l) && Self::looks_like_math(r) {
                return Ok(IntentType::SolveSingleEquation {
                    lhs: l.to_string(),
                    rhs: r.to_string(),
                });
            }
        }

        if Self::looks_like_math(target) {
            return Ok(IntentType::EvaluateExpression {
                expr: target.to_string(),
            });
        }

        Ok(IntentType::Echo {
            message: trimmed.to_string(),
        })
    }

    fn strip_any_prefix<'a>(s: &'a str, prefixes: &[&str]) -> Option<&'a str> {
        for p in prefixes {
            if let Some(rest) = s.strip_prefix(p) {
                return Some(rest.trim());
            }
        }
        None
    }

    fn is_coupled_multivariate_system(items: &[String]) -> bool {
        if items.len() < 2 || !items.iter().all(|s| s.contains('=')) {
            return false;
        }
        let mut all_vars = BTreeSet::new();
        for item in items {
            for c in item.chars() {
                if c.is_ascii_alphabetic() {
                    all_vars.insert(c.to_ascii_lowercase());
                }
            }
        }
        all_vars.len() >= 2
    }

    fn strip_natural_prefixes(s: &str) -> String {
        let prefixes = [
            "ทดสอบ",
            "ช่วยคิด",
            "ช่วยคำนวณ",
            "คำนวณ",
            "แก้ระบบสมการ",
            "แก้สมการ",
            "หาค่า",
            "ถาม",
            "calc:",
            "CALC:",
            "solve:",
            "SOLVE:",
            "test",
            "TEST",
        ];
        let mut current = s.trim();
        let mut changed = true;
        while changed {
            changed = false;
            for p in prefixes {
                if let Some(rest) = current.strip_prefix(p) {
                    current = rest.trim();
                    changed = true;
                }
            }
        }
        current.to_string()
    }

    fn split_multi_math_tokens(s: &str) -> Vec<String> {
        let raw_parts: Vec<&str> = s.split_whitespace().collect();
        if raw_parts.len() <= 1 {
            return raw_parts.into_iter().map(|x| x.to_string()).collect();
        }
        let all_self_contained = raw_parts.iter().all(|part| {
            Self::looks_like_math(part)
                && !matches!(part.chars().next(), Some('+' | '*' | '/' | '^' | '='))
                && !matches!(part.chars().last(), Some('+' | '-' | '*' | '/' | '^' | '='))
                && (part.contains('+')
                    || part.contains('-')
                    || part.contains('*')
                    || part.contains('/')
                    || part.contains('^')
                    || part.contains('='))
        });

        if all_self_contained {
            raw_parts.into_iter().map(|x| x.to_string()).collect()
        } else {
            vec![s.trim().to_string()]
        }
    }

    pub fn looks_like_math(s: &str) -> bool {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return false;
        }
        let has_digit_or_var = trimmed
            .chars()
            .any(|c| c.is_ascii_digit() || c.is_ascii_alphabetic());
        let all_valid = trimmed.chars().all(|c| {
            c.is_ascii_digit()
                || c.is_ascii_alphabetic()
                || c.is_whitespace()
                || matches!(
                    c,
                    '.' | '+' | '-' | '−' | '*' | '×' | '/' | '÷' | '^' | '(' | ')' | '='
                )
        });
        has_digit_or_var && all_valid
    }
}