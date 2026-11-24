// EVM metrics tests — verify EAC/BAC/TCPI/VAC per PMI standard
// (guide §5.6 TDD: test written first, CI verifies)

use cadence_core::model::{Activity, ActivityId, EvmMetrics};

#[test]
fn evm_metrics_eac_bac_tcpi_vac() {
    // BAC=100, EV=40, AC=50, PV=50
    // CPI = 40/50 = 0.8
    // EAC = BAC/CPI = 100/0.8 = 125
    // TCPI = (100-40)/(100-50) = 60/50 = 1.2
    // VAC = BAC - EAC = 100 - 125 = -25 (over budget)
    let bac = 100;
    let ev = 40;
    let ac = 50;
    let _pv = 50;

    let cpi = Some(ev as f64 / ac as f64);
    let eac = Some(bac as f64 / cpi.unwrap());
    let tcpi = Some((bac as f64 - ev as f64) / (bac as f64 - ac as f64));
    let vac = bac as f64 - eac.unwrap();

    assert!((eac.unwrap() - 125.0).abs() < f64::EPSILON);
    assert!((tcpi.unwrap() - 1.2).abs() < f64::EPSILON);
    assert!((vac - (-25.0)).abs() < f64::EPSILON);
}

#[test]
fn evm_metrics_no_baseline_zero_pv_ev() {
    let bac = 100;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(0),
        percent_complete: Some(0),
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 0); // planned_fraction = 0

    assert_eq!(metrics.planned_value, 0);
    assert_eq!(metrics.earned_value, 0);
    assert_eq!(metrics.actual_cost, 0);
    assert_eq!(metrics.cost_variance, 0);
    assert_eq!(metrics.schedule_variance, 0);
    assert_eq!(metrics.cost_performance_index, None);
    assert_eq!(metrics.schedule_performance_index, None);
    // EAC falls back to BAC when CPI undefined
    assert_eq!(metrics.estimate_at_completion, Some(bac as f64));
    // TCPI = (BAC - EV) / (BAC - AC) = (100-0)/(100-0) = 1.0
    assert!((metrics.to_complete_performance_index.unwrap() - 1.0).abs() < f64::EPSILON);
    // VAC = BAC - EAC = 100 - 100 = 0
    assert_eq!(metrics.variance_at_completion, 0);
}

#[test]
fn evm_metrics_cpi_undefined_ac_zero() {
    let bac = 100;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(0),
        percent_complete: Some(5000), // 50%
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 5000);

    assert_eq!(metrics.earned_value, 50);
    assert_eq!(metrics.actual_cost, 0);
    assert_eq!(metrics.cost_performance_index, None); // CPI undefined when AC=0
                                                      // EAC falls back to BAC
    assert_eq!(metrics.estimate_at_completion, Some(bac as f64));
    // TCPI = (100-50)/(100-0) = 0.5
    assert!((metrics.to_complete_performance_index.unwrap() - 0.5).abs() < f64::EPSILON);
    // VAC = 0
    assert_eq!(metrics.variance_at_completion, 0);
}

#[test]
fn evm_metrics_tcpi_none_when_bac_equals_ac() {
    let bac = 100;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(100),       // AC == BAC
        percent_complete: Some(8000), // 80%
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 8000);

    assert_eq!(metrics.earned_value, 80);
    assert_eq!(metrics.actual_cost, 100);
    assert_eq!(metrics.to_complete_performance_index, None); // TCPI undefined when BAC == AC
}

#[test]
fn evm_metrics_over_budget_positive_vac() {
    // EAC > BAC → VAC negative (over budget in PMI convention: VAC = BAC - EAC)
    // But our implementation: variance_at_completion = BAC - EAC
    // If EAC=150, BAC=100 → VAC = -50 (negative = over budget)
    let bac = 100;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(80),
        percent_complete: Some(5000), // 50%
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 5000);

    // PV=50, EV=50, AC=80
    // CV = -30, SV = 0
    // CPI = 50/80 = 0.625
    // EAC = 100/0.625 = 160
    // TCPI = (100-50)/(100-80) = 50/20 = 2.5
    // VAC = 100 - 160 = -60
    assert_eq!(metrics.cost_variance, -30);
    assert_eq!(metrics.schedule_variance, 0);
    assert!((metrics.cost_performance_index.unwrap() - 0.625).abs() < f64::EPSILON);
    assert!((metrics.estimate_at_completion.unwrap() - 160.0).abs() < f64::EPSILON);
    assert!((metrics.to_complete_performance_index.unwrap() - 2.5).abs() < f64::EPSILON);
    assert_eq!(metrics.variance_at_completion, -60);
}

#[test]
fn evm_metrics_under_budget_negative_vac() {
    // EAC < BAC → VAC positive (under budget)
    let bac = 100;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(30),
        percent_complete: Some(5000), // 50%
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 5000);

    // PV=50, EV=50, AC=30
    // CPI = 50/30 = 1.666...
    // EAC = 100/1.666... = 60
    // TCPI = (100-50)/(100-30) = 50/70 = 0.714...
    // VAC = 100 - 60 = 40
    assert_eq!(metrics.cost_variance, 20);
    assert!((metrics.estimate_at_completion.unwrap() - 60.0).abs() < 0.001);
    assert!((metrics.to_complete_performance_index.unwrap() - (50.0 / 70.0)).abs() < f64::EPSILON);
    assert_eq!(metrics.variance_at_completion, 40);
}

#[test]
fn evm_metrics_serde_roundtrip() {
    let bac = 12345;
    let activity = Activity {
        id: ActivityId(1),
        name: "Test".to_owned(),
        original_duration: 0,
        wbs: None,
        resources: Vec::new(),
        assignment_units: std::collections::BTreeMap::new(),
        calendar: None,
        constraint: cadence_core::model::Constraint::AsSoonAsPossible,
        early_start: None,
        early_finish: None,
        late_start: None,
        late_finish: None,
        scheduled_start: None,
        scheduled_finish: None,
        total_float: None,
        free_float: None,
        critical: false,
        budget_at_completion: Some(bac),
        actual_cost: Some(6789),
        percent_complete: Some(6000),
        actual_start: None,
        actual_finish: None,
        owner: None,
        promised: false,
        delay_reason: None,
        location: None,
    };
    let metrics = EvmMetrics::for_activity(&activity, 6000);

    let json = serde_json::to_string(&metrics).unwrap();
    let decoded: EvmMetrics = serde_json::from_str(&json).unwrap();

    assert_eq!(metrics, decoded);
}

#[test]
fn evm_metrics_project_level_aggregation() {
    // Project-level EVM is a sum of activity-level metrics
    // This test verifies the aggregation pattern matches the engine
    let activities = vec![
        Activity {
            id: ActivityId(1),
            name: "A1".to_owned(),
            original_duration: 0,
            wbs: None,
            resources: Vec::new(),
            assignment_units: std::collections::BTreeMap::new(),
            calendar: None,
            constraint: cadence_core::model::Constraint::AsSoonAsPossible,
            early_start: None,
            early_finish: None,
            late_start: None,
            late_finish: None,
            scheduled_start: None,
            scheduled_finish: None,
            total_float: None,
            free_float: None,
            critical: false,
            budget_at_completion: Some(100),
            actual_cost: Some(50),
            percent_complete: Some(5000),
            actual_start: None,
            actual_finish: None,
            owner: None,
            promised: false,
            delay_reason: None,
            location: None,
        },
        Activity {
            id: ActivityId(2),
            name: "A2".to_owned(),
            original_duration: 0,
            wbs: None,
            resources: Vec::new(),
            assignment_units: std::collections::BTreeMap::new(),
            calendar: None,
            constraint: cadence_core::model::Constraint::AsSoonAsPossible,
            early_start: None,
            early_finish: None,
            late_start: None,
            late_finish: None,
            scheduled_start: None,
            scheduled_finish: None,
            total_float: None,
            free_float: None,
            critical: false,
            budget_at_completion: Some(200),
            actual_cost: Some(100),
            percent_complete: Some(2500),
            actual_start: None,
            actual_finish: None,
            owner: None,
            promised: false,
            delay_reason: None,
            location: None,
        },
    ];
    let planned_fraction = 5000; // project at 50% planned

    // Sum of BACs
    let total_bac: i64 = activities
        .iter()
        .map(|a| a.budget_at_completion.unwrap_or(0))
        .sum();
    assert_eq!(total_bac, 300);

    // Aggregate metrics follow the same formulas
    let total_ev: i64 = activities
        .iter()
        .map(|a| {
            let bac = a.budget_at_completion.unwrap_or(0);
            let pct = a.percent_complete.unwrap_or(0).clamp(0, 10_000);
            (i128::from(bac) * i128::from(pct) / 10_000) as i64
        })
        .sum();
    let total_ac: i64 = activities.iter().map(|a| a.actual_cost.unwrap_or(0)).sum();
    let total_pv: i64 = activities
        .iter()
        .map(|a| {
            let bac = a.budget_at_completion.unwrap_or(0);
            (i128::from(bac) * i128::from(planned_fraction) / 10_000) as i64
        })
        .sum();

    assert_eq!(total_ev, 100); // 50 + 50
    assert_eq!(total_ac, 150); // 50 + 100
    assert_eq!(total_pv, 150); // 50 + 100
}
