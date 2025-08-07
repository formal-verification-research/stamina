use std::process::Command;

fn main() {
	if Command::new("which").arg("z3").output().is_ok() {
		println!("Z3 is already installed. If you find an error, please ensure that the z3 library is correctly linked in your project.");
		println!("If you encounter this error, a common fix is to install the z3 library using your package manager, e.g., `apt install libz3-dev` or `yum install z3-libs`.");
		return;
	} else {
		println!("Z3 is not installed. Attempting to install using package manager or build from source.");
	}

	// Determine the package manager and install the package accordingly
	let output = if Command::new("which").arg("apt").output().is_ok() {
		// For Debian/Ubuntu
		Command::new("apt")
			.args(&["install", "-y", "libz3-dev"])
			.output()
	} else if Command::new("which").arg("yum").output().is_ok() {
		// For CentOS/RHEL
		Command::new("yum")
			.args(&["install", "-y", "z3-libs"])
			.output()
	} else {
		println!("No supported package manager found. Building z3 from source.");
		// Fallback to building from source
		let status = Command::new("git")
			.args(&["clone", "https://github.com/Z3Prover/z3.git"])
			.status()
			.expect("Failed to clone Z3 repository");
		if !status.success() {
			panic!("Failed to clone Z3 repository");
		}

		let status = Command::new("git")
			.args(&["checkout", "tags/z3-4.13.0"])
			.current_dir("z3")
			.status()
			.expect("Failed to checkout Z3 tag");
		if !status.success() {
			panic!("Failed to checkout Z3 tag");
		}

		let status = Command::new("mkdir")
			.arg("build")
			.current_dir("z3")
			.status()
			.expect("Failed to create build directory");
		if !status.success() {
			panic!("Failed to create build directory");
		}

		let status = Command::new("cmake")
			.args(&[
				"-G",
				"Unix Makefiles",
				"-DCMAKE_BUILD_TYPE=Release",
				"-j4",
				"..",
			])
			.current_dir("z3/build")
			.status()
			.expect("Failed to run cmake");
		if !status.success() {
			panic!("Failed to run cmake");
		}

		Command::new("make")
			.arg("install")
			.current_dir("z3/build")
			.output()
	};

	// Check if the command was successful
	let output = match output {
		Ok(o) => o,
		Err(e) => panic!("Failed to execute package manager: {}", e),
	};

	if !output.status.success() {
		panic!(
			"Package installation failed: {}",
			String::from_utf8_lossy(&output.stderr)
		);
	}
}
