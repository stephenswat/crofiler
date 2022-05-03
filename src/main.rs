//! Easier C++ build profiling

#![deny(missing_docs)]

use clang_time_trace::{ActivityTrace, ClangTrace};
use num_traits::ToPrimitive;
use std::{fmt::Display, iter::Sum, ops::Add};

fn main() {
    let trace =
        ClangTrace::from_file("2020-05-25_CombinatorialKalmanFilterTests.cpp.json").unwrap();

    println!("Profile from {}", trace.process_name());

    println!("\nGlobal statistics: {:#?}", trace.global_stats());

    // Flat profile prototype
    const SELF_CUTOFF: f32 = 0.01;
    println!(
        "\nSelf-time flat profile with {} % cutoff:",
        SELF_CUTOFF * 100.0
    );
    let root_duration = trace
        .root_activities()
        .map(|root| root.duration())
        .sum::<f64>();
    display_flat_profile(
        &trace,
        |a| a.self_duration(),
        "µs",
        Some(root_duration),
        Some(SELF_CUTOFF),
    );
    //
    const CHILD_CUTOFF: f32 = 0.01;
    println!(
        "\nDirect children flat profile with {} % cutoff:",
        CHILD_CUTOFF * 100.0
    );
    let num_activities = trace.all_activities().count();
    display_flat_profile(
        &trace,
        |a| a.direct_children().count(),
        "children",
        Some(num_activities),
        Some(CHILD_CUTOFF),
    );

    // Hierarchical profile prototype
    // (TODO: Make this more hierarchical and display using termtree)
    println!("\nTree roots:");
    for root in trace.root_activities() {
        println!("- {root:#?}");
    }
}

/// Display a flat profile of clang activities according to some metric
///
/// - `trace` is the clang execution trace to be profiled
/// - `metric` is the sorting criterion to be used for profiling, higher is
///   more overhead. For flat profiles, you should pick metrics that are not
///   aggregated across children in the activity hierarchy (e.g. prefer
///   Activity::self_duration over Activity::duration).
/// - `unit` is the unit suffix to be used when displaying metric values
/// - `sum` allows the sum of the metric across all activities to be
///   externally provided in situations where it can be more efficiently or
///   precisely computed than through naive summation.
/// - `threshold` is the minimal relative share of the metric sum that an
///   activity must exhibit in order to be featured in the profile. For example,
///   if this is 0.01, then metric(activity) must be more than 1% of the sum of
///   the metric across all activities.
fn display_flat_profile<Metric>(
    trace: &ClangTrace,
    metric: impl Fn(&ActivityTrace) -> Metric,
    unit: &'static str,
    sum: Option<Metric>,
    threshold: Option<f32>,
) where
    Metric: Add + Display + PartialOrd + Sum + ToPrimitive,
{
    // Determine retention threshold, check its validity if provided
    let threshold = threshold.unwrap_or(0.0);
    assert!(threshold.is_normal() && threshold >= 0.0);

    // Determine metric sum and associated normalization factor
    let sum = sum.unwrap_or_else(|| trace.all_activities().map(|a| metric(&a)).sum::<Metric>());
    let norm = 1.0 / sum.to_f32().unwrap();

    // Collect activities passing retention threshold
    let mut activities = trace
        .all_activities()
        .filter(|a| metric(&a).to_f32().unwrap() * norm >= threshold)
        .collect::<Box<[_]>>();

    // Sort them by decreasing metric value
    activities.sort_unstable_by(|a1, a2| metric(&a2).partial_cmp(&metric(&a1)).unwrap());

    // Display the resulting profile
    for activity_trace in activities.iter() {
        let activity = activity_trace.activity();
        let metric = metric(activity_trace);
        let percent = metric.to_f32().unwrap() * norm * 100.0;
        println!("- {activity:?} ({metric} {unit}, {percent:.2} %)");
    }
}
