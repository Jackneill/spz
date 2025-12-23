use pyo3::prelude::*;

#[pymodule]
mod spz {
	use pyo3::prelude::*;

	#[pyfunction]
	fn sum_as_string(a: usize, b: usize) -> PyResult<String> {
		Ok((a + b).to_string())
	}
}
