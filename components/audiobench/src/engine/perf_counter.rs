use crate::util::*;
use std::time::{Duration, Instant};

pub struct PerfCountSection {
    index: usize,
    name: &'static str,
}

pub mod sections {
    use super::PerfCountSection;

    pub const GLOBAL_SETUP: PerfCountSection = PerfCountSection {
        index: 0,
        name: "Global Setup",
    };
    pub const NOTE_SETUP: PerfCountSection = PerfCountSection {
        index: 1,
        name: "Note Setup",
    };
    pub const NODESPEAK_EXEC: PerfCountSection = PerfCountSection {
        index: 2,
        name: "Nodespeak Exec",
    };
    pub const NOTE_FINALIZE: PerfCountSection = PerfCountSection {
        index: 3,
        name: "Note Finalize",
    };
    pub const GLOBAL_FINALIZE: PerfCountSection = PerfCountSection {
        index: 4,
        name: "Global Finalize",
    };
    pub const GENERATE_CODE: PerfCountSection = PerfCountSection {
        index: 8,
        name: "Generate Code",
    };
    pub const COMPILE_CODE: PerfCountSection = PerfCountSection {
        index: 5,
        name: "Compile Code",
    };
    pub const COLLECT_AUTOCON_DATA: PerfCountSection = PerfCountSection {
        index: 6,
        name: "Collect Autocon Data",
    };
    pub const COLLECT_STATICON_DATA: PerfCountSection = PerfCountSection {
        index: 7,
        name: "Collect Staticon Data",
    };

    pub const NUM_SECTIONS: usize = 9;
    pub const ALL_SECTIONS: [&'static PerfCountSection; NUM_SECTIONS] = [
        &GENERATE_CODE,
        &COMPILE_CODE,
        &COLLECT_AUTOCON_DATA,
        &COLLECT_STATICON_DATA,
        &GLOBAL_SETUP,
        &NOTE_SETUP,
        &NODESPEAK_EXEC,
        &NOTE_FINALIZE,
        &GLOBAL_FINALIZE,
    ];
}

use sections::NUM_SECTIONS;

pub trait PerfCounter {
    fn new() -> Self;
    fn begin_section(&mut self, section: &PerfCountSection);
    fn end_section(&mut self, section: &PerfCountSection);
    fn report(&self) -> String;
}

/// Does nothing.
pub struct NoopPerfCounter;

impl PerfCounter for NoopPerfCounter {
    fn new() -> Self {
        Self
    }

    fn begin_section(&mut self, _section: &PerfCountSection) {}
    fn end_section(&mut self, _section: &PerfCountSection) {}
    fn report(&self) -> String {
        "No report available (NoopPerfCounter)".to_owned()
    }
}

/// Limited statistics, but fast enough to run in production builds without
/// screwing with anything.
pub struct SimplePerfCounter {
    num_invocations: [u32; NUM_SECTIONS],
    cumulative_time: [Duration; NUM_SECTIONS],
    current_section: Option<usize>,
    section_start_time: Instant,
}

impl PerfCounter for SimplePerfCounter {
    fn new() -> Self {
        Self {
            num_invocations: [0; NUM_SECTIONS],
            cumulative_time: [Duration::from_secs(0); NUM_SECTIONS],
            current_section: None,
            section_start_time: Instant::now(),
        }
    }

    fn begin_section(&mut self, section: &PerfCountSection) {
        assert!(
            self.current_section.is_none(),
            "ERROR: A section named {} was begun without closing the previous section.",
            section.name
        );
        self.current_section = Some(section.index);
        // We do this last to make the timing statistics as accurate as possible.
        self.section_start_time = Instant::now();
    }

    fn end_section(&mut self, section: &PerfCountSection) {
        // We do this first to make the timing statistics as accurate as possible.
        self.cumulative_time[section.index] += self.section_start_time.elapsed();
        self.num_invocations[section.index] += 1;
        assert!(
            self.current_section.is_some(),
            "ERROR: Tried to end a section named {} but the section was not started.",
            section.name
        );
        assert!(
            self.current_section == Some(section.index),
            "ERROR: Tried to end a section named {} while in the middle of a different section.",
            section.name
        );
        self.current_section = None;
    }

    fn report(&self) -> String {
        let mut report = String::new();
        report += &format!(
            "SECTION NAME                   | TOTAL TIME | SAMPLES | TIME PER SAMPLE \n"
        );
        let mut everything_time = 0.0;
        for section in &sections::ALL_SECTIONS {
            let invocations = self.num_invocations[section.index];
            let total_time = self.cumulative_time[section.index].as_secs_f64();
            everything_time += total_time;
            let average_time = total_time / (invocations as f64);
            report += &format!(
                "{:<30} | {:>10} | {:>7} | {:>15} \n",
                section.name,
                total_time.format_metric(6, "s"),
                invocations,
                average_time.format_metric(6, "s")
            );
        }
        report += &format!(
            "                                 {:>10}",
            everything_time.format_metric(6, "s")
        );
        report
    }
}
