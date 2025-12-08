pub fn degree_for_dim(dim: u8) -> u8 {
	if dim < 3 {
		0
	} else if dim < 8 {
		1
	} else if dim < 15 {
		2
	} else {
		3
	}
}

pub fn dim_for_degree(degree: u8) -> u8 {
	match degree {
		0 => 0,
		1 => 3,
		2 => 8,
		3 => 15,
		_ => 0,
	}
}
