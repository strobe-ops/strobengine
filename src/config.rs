use pyo3::prelude::*;

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
}
