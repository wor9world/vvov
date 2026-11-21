use server::core::Orchestrator;

#[test]
fn test_batch_9_8_minus_9_11_and_linear_and_exponential() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("ทดสอบ  9.8-9.11   23x+100=491   3^x=x^9")
        .expect("Should solve full batch");
    assert_eq!(res.intent, "BatchProcess");
    assert!(res.output.contains("9.8-9.11 = -0.69"));
    assert!(res.output.contains("x = 17"));
    assert!(res.output.contains("x ≈ 1.1508248246"));
    assert!(res.output.contains("x = 27"));
}

#[test]
fn test_exact_rational_fraction_output() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("3x+y=1, y=0")
        .expect("Should reconstruct exact rational 1/3");
    assert!(
        res.output.contains("x = 1/3 (≈ 0.3333333333)"),
        "Expected exact fraction with approx decimal, got: {}",
        res.output
    );
}

#[test]
fn test_3x3_linear_multivariate_system() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("x+y+z=6, 2x-y+z=3, x+2y-z=2")
        .expect("Should solve 3x3 system");
    assert!(res.output.contains("x = 1"));
    assert!(res.output.contains("y = 2"));
    assert!(res.output.contains("z = 3"));
}

#[test]
fn test_nonlinear_multivariate_circle_line_intersection() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("x^2+y^2=25, x-y=1")
        .expect("Should solve non-linear circle-line intersection via Jacobian Newton-Raphson");
    assert_eq!(res.intent, "SolveMultivariateSystem");
    // จุดตัดทั้ง 2 จุดคือ (-3, -4) และ (4, 3)
    assert!(
        res.output.contains("(x = -3, y = -4)") && res.output.contains("(x = 4, y = 3)"),
        "Expected both intersection tuples, got: {}",
        res.output
    );
}

#[test]
fn test_symbolic_differentiation() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("diff: 4x^3 - 5x^2 + 2x - 9")
        .expect("Should compute symbolic derivative");
    assert_eq!(res.intent, "SymbolicCalculus");
    assert_eq!(res.output, "d/dx [4x^3 - 5x^2 + 2x - 9] = 12x^2 - 10x + 2");
}

#[test]
fn test_definite_integral_simpson() {
    let orchestrator = Orchestrator::default();
    let res = orchestrator
        .ask("int: x^2, 0, 3")
        .expect("Should compute definite integral");
    assert_eq!(res.intent, "DefiniteIntegral");
    assert_eq!(res.numeric_result, Some(9.0));
    assert!(res.output.contains("= 9"));
}

#[test]
fn test_boolean_sat_tautology_and_satisfiable() {
    let orchestrator = Orchestrator::default();
    let taut = orchestrator
        .ask("sat: (A & B) -> (A | C)")
        .expect("Should verify tautology");
    assert!(taut.output.contains("VALID / TAUTOLOGY"));

    let sat = orchestrator
        .ask("sat: (A | B) & !A")
        .expect("Should find satisfying assignment");
    assert!(sat.output.contains("SATISFIABLE"));
    assert!(sat.output.contains("A=F, B=T"));
}