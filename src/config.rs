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
