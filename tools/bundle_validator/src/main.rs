use aigc_core::policy::types::PolicyMode;
use aigc_core::validator::BundleValidator;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "usage: bundle_validator <STRICT|BALANCED|DRAFT_ONLY> <path/to/evidence_bundle_*.zip>"
        );
        std::process::exit(2);
    }
    let policy = match args[1].as_str() {
        "STRICT" => PolicyMode::STRICT,
        "BALANCED" => PolicyMode::BALANCED,
        "DRAFT_ONLY" => PolicyMode::DRAFT_ONLY,
        other => {
            eprintln!("invalid policy: {}", other);
            std::process::exit(2);
        }
    };
    let path = std::path::Path::new(&args[2]);

    let v = BundleValidator::new_v3();
    match v.validate_zip(path, policy) {
        Ok(summary) => {
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            if summary.overall == "PASS" {
                std::process::exit(0);
            } else {
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("validator error: {}", e);
            std::process::exit(1);
        }
    }
}
