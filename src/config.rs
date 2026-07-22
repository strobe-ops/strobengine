use pyo3::prelude::*;
use std::time::Duration;

#[pyclass(from_py_object)]
#[derive(Clone)]
pub struct TestConfig {
    #[pyo3(get, set)]
    pub url: String,
    #[pyo3(get, set)]
    pub concurrency: usize,
    #[pyo3(get, set)]
    pub duration_secs: u64,
    #[pyo3(get, set)]
    pub timeout_secs: u64,
}

#[pymethods]
impl TestConfig {
    #[new]
    #[pyo3(signature = (url, concurrency=10, duration_secs=10, timeout_secs=10))]
    pub fn new(url: String, concurrency: usize, duration_secs: u64, timeout_secs: u64) -> Self {
        Self {
            url,
            concurrency,
            duration_secs,
            timeout_secs,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(from_py_object)]
pub enum LoadProfile {
    Constant {
        concurrency: usize,
        duration_secs: u64,
    },
    Ramp {
        start_concurrency: usize,
        target_concurrency: usize,
        ramp_secs: u64,
        hold_secs: u64,
    },
    Spike {
        baseline_concurrency: usize,
        peak_concurrency: usize,
        pre_spike_secs: u64,
        spike_secs: u64,
        post_spike_secs: u64,
    },
}

#[pymethods]
impl LoadProfile {
    #[staticmethod]
    #[pyo3(signature = (concurrency=10, duration_secs=10))]
    fn constant(concurrency: usize, duration_secs: u64) -> Self {
        LoadProfile::Constant {
            concurrency,
            duration_secs,
        }
    }

    #[staticmethod]
    fn ramp(
        start_concurrency: usize,
        target_concurrency: usize,
        ramp_secs: u64,
        hold_secs: u64,
    ) -> Self {
        LoadProfile::Ramp {
            start_concurrency,
            target_concurrency,
            ramp_secs,
            hold_secs,
        }
    }

    #[staticmethod]
    fn spike(
        baseline_concurrency: usize,
        peak_concurrency: usize,
        pre_spike_secs: u64,
        spike_secs: u64,
        post_spike_secs: u64,
    ) -> Self {
        LoadProfile::Spike {
            baseline_concurrency,
            peak_concurrency,
            pre_spike_secs,
            spike_secs,
            post_spike_secs,
        }
    }

    pub fn total_duration(&self) -> u64 {
        match self {
            LoadProfile::Constant { duration_secs, .. } => *duration_secs,
            LoadProfile::Ramp {
                ramp_secs,
                hold_secs,
                ..
            } => ramp_secs + hold_secs,
            LoadProfile::Spike {
                pre_spike_secs,
                spike_secs,
                post_spike_secs,
                ..
            } => pre_spike_secs + spike_secs + post_spike_secs,
        }
    }

    pub fn max_concurrency(&self) -> usize {
        match self {
            LoadProfile::Constant { concurrency, .. } => *concurrency,
            LoadProfile::Ramp {
                target_concurrency, ..
            } => *target_concurrency,
            LoadProfile::Spike {
                peak_concurrency, ..
            } => *peak_concurrency,
        }
    }

    pub fn target_concurrency(&self, elapsed: Duration) -> usize {
        let t = elapsed.as_secs();
        match self {
            LoadProfile::Constant { concurrency, .. } => *concurrency,
            LoadProfile::Ramp {
                start_concurrency,
                target_concurrency,
                ramp_secs,
                hold_secs: _,
            } => {
                if t < *ramp_secs {
                    if *ramp_secs == 0 {
                        return *target_concurrency;
                    }
                    let progress = t as f64 / *ramp_secs as f64;
                    let range = *target_concurrency as f64 - *start_concurrency as f64;
                    (*start_concurrency as f64 + range * progress).round() as usize
                } else {
                    *target_concurrency
                }
            }
            LoadProfile::Spike {
                baseline_concurrency,
                peak_concurrency,
                pre_spike_secs,
                spike_secs,
                ..
            } => {
                if t < *pre_spike_secs {
                    *baseline_concurrency
                } else if t < pre_spike_secs + spike_secs {
                    *peak_concurrency
                } else {
                    *baseline_concurrency
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_with_defaults() {
        let c = TestConfig::new("http://example.com".into(), 10, 10, 10);
        assert_eq!(c.url, "http://example.com");
        assert_eq!(c.concurrency, 10);
        assert_eq!(c.duration_secs, 10);
        assert_eq!(c.timeout_secs, 10);
    }

    #[test]
    fn new_with_custom_values() {
        let c = TestConfig::new("http://test.io".into(), 50, 30, 5);
        assert_eq!(c.url, "http://test.io");
        assert_eq!(c.concurrency, 50);
        assert_eq!(c.duration_secs, 30);
        assert_eq!(c.timeout_secs, 5);
    }

    #[test]
    fn fields_are_gettable() {
        let c = TestConfig::new("http://a.com".into(), 1, 2, 3);
        assert_eq!(c.url, "http://a.com");
        assert_eq!(c.concurrency, 1);
        assert_eq!(c.duration_secs, 2);
        assert_eq!(c.timeout_secs, 3);
    }

    // LoadProfile tests

    #[test]
    fn constant_total_duration() {
        let p = LoadProfile::Constant {
            concurrency: 10,
            duration_secs: 30,
        };
        assert_eq!(p.total_duration(), 30);
    }

    #[test]
    fn constant_max_concurrency() {
        let p = LoadProfile::Constant {
            concurrency: 50,
            duration_secs: 10,
        };
        assert_eq!(p.max_concurrency(), 50);
    }

    #[test]
    fn constant_target_concurrency() {
        let p = LoadProfile::Constant {
            concurrency: 25,
            duration_secs: 10,
        };
        assert_eq!(p.target_concurrency(Duration::from_secs(0)), 25);
        assert_eq!(p.target_concurrency(Duration::from_secs(5)), 25);
        assert_eq!(p.target_concurrency(Duration::from_secs(10)), 25);
    }

    #[test]
    fn ramp_total_duration() {
        let p = LoadProfile::Ramp {
            start_concurrency: 10,
            target_concurrency: 100,
            ramp_secs: 60,
            hold_secs: 30,
        };
        assert_eq!(p.total_duration(), 90);
    }

    #[test]
    fn ramp_max_concurrency() {
        let p = LoadProfile::Ramp {
            start_concurrency: 10,
            target_concurrency: 200,
            ramp_secs: 60,
            hold_secs: 30,
        };
        assert_eq!(p.max_concurrency(), 200);
    }

    #[test]
    fn ramp_target_concurrency_interpolation() {
        let p = LoadProfile::Ramp {
            start_concurrency: 10,
            target_concurrency: 100,
            ramp_secs: 90,
            hold_secs: 10,
        };
        // At t=0: start
        assert_eq!(p.target_concurrency(Duration::from_secs(0)), 10);
        // At t=45 (halfway): ~55
        assert_eq!(p.target_concurrency(Duration::from_secs(45)), 55);
        // At t=90 (end of ramp): target
        assert_eq!(p.target_concurrency(Duration::from_secs(90)), 100);
        // At t=100 (during hold): target
        assert_eq!(p.target_concurrency(Duration::from_secs(100)), 100);
    }

    #[test]
    fn ramp_zero_duration() {
        let p = LoadProfile::Ramp {
            start_concurrency: 10,
            target_concurrency: 100,
            ramp_secs: 0,
            hold_secs: 10,
        };
        assert_eq!(p.target_concurrency(Duration::from_secs(0)), 100);
    }

    #[test]
    fn spike_total_duration() {
        let p = LoadProfile::Spike {
            baseline_concurrency: 5,
            peak_concurrency: 500,
            pre_spike_secs: 10,
            spike_secs: 20,
            post_spike_secs: 10,
        };
        assert_eq!(p.total_duration(), 40);
    }

    #[test]
    fn spike_max_concurrency() {
        let p = LoadProfile::Spike {
            baseline_concurrency: 5,
            peak_concurrency: 1000,
            pre_spike_secs: 5,
            spike_secs: 10,
            post_spike_secs: 5,
        };
        assert_eq!(p.max_concurrency(), 1000);
    }

    #[test]
    fn spike_target_concurrency_phases() {
        let p = LoadProfile::Spike {
            baseline_concurrency: 5,
            peak_concurrency: 500,
            pre_spike_secs: 10,
            spike_secs: 20,
            post_spike_secs: 10,
        };
        // Pre-spike
        assert_eq!(p.target_concurrency(Duration::from_secs(0)), 5);
        assert_eq!(p.target_concurrency(Duration::from_secs(9)), 5);
        // Spike
        assert_eq!(p.target_concurrency(Duration::from_secs(10)), 500);
        assert_eq!(p.target_concurrency(Duration::from_secs(29)), 500);
        // Post-spike
        assert_eq!(p.target_concurrency(Duration::from_secs(30)), 5);
        assert_eq!(p.target_concurrency(Duration::from_secs(39)), 5);
    }
}
