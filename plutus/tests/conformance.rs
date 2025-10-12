use std::path::PathBuf;

#[test]
fn conformance() {
    let mut directories = vec![PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/conformance"
    ))];

    while let Some(dir) = directories.pop() {
        let mut is_dir = false;
        for entry in dir.read_dir().unwrap() {
            let entry = entry.unwrap();
            if entry.path().is_dir() {
                directories.push(entry.path());
                is_dir = true;
            }
        }

        if !is_dir {
            let file_name = dir.file_name().unwrap().to_str().unwrap();
            let program_path = dir.join(file_name).with_extension("uplc");
            if program_path.components().any(|c| c.as_os_str() == "value") {
                // Skip value tests for now
                continue;
            }

            let program = std::fs::read_to_string(&program_path).unwrap();
            let expected_path = program_path.to_string_lossy().to_string() + ".expected";
            let expected_output = std::fs::read_to_string(&expected_path).unwrap();

            eprintln!("{file_name}: {program}");
            if expected_output != "parse error" {
                let output: plutus::Program = program.parse().unwrap();
                println!("{output:#?}");
            }
        }
    }
}
