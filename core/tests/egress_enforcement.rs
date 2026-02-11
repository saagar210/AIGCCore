use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn core_egress_is_restricted_to_policy_egress_module() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let core_src = repo_root.join("core").join("src");
    let allowed = core_src.join("policy").join("egress.rs");

    let mut files = Vec::new();
    walk_rs(&core_src, &mut files);

    let forbidden = [
        "reqwest::",
        "ureq::",
        "surf::",
        "hyper::Client",
        "std::net::TcpStream",
    ];
    for f in files {
        if f == allowed {
            continue;
        }
        let text = fs::read_to_string(&f).unwrap();
        for tok in forbidden {
            assert!(
                !text.contains(tok),
                "forbidden egress token '{}' in {}",
                tok,
                f.display()
            );
        }
    }
}

fn walk_rs(dir: &Path, out: &mut Vec<PathBuf>) {
    for ent in fs::read_dir(dir).unwrap() {
        let ent = ent.unwrap();
        let p = ent.path();
        if p.is_dir() {
            walk_rs(&p, out);
        } else if p.extension().and_then(|x| x.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}
