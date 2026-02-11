import fs from "node:fs";
import path from "node:path";

const repo = process.cwd();
const coreSrc = path.join(repo, "core", "src");
const allowFile = path.join(coreSrc, "policy", "egress.rs");

const forbiddenPatterns = [
  "reqwest::",
  "ureq::",
  "surf::",
  "hyper::Client",
  "std::net::TcpStream"
];

function walk(dir, out = []) {
  for (const ent of fs.readdirSync(dir, { withFileTypes: true })) {
    const p = path.join(dir, ent.name);
    if (ent.isDirectory()) walk(p, out);
    else if (ent.isFile() && p.endsWith(".rs")) out.push(p);
  }
  return out;
}

const files = walk(coreSrc).filter((f) => f !== allowFile);
const violations = [];
for (const f of files) {
  const text = fs.readFileSync(f, "utf8");
  for (const pat of forbiddenPatterns) {
    if (text.includes(pat)) {
      violations.push(`${path.relative(repo, f)} contains forbidden egress token "${pat}"`);
    }
  }
}

// Dependency-level check in core/Cargo.toml
const cargoToml = fs.readFileSync(path.join(repo, "core", "Cargo.toml"), "utf8");
for (const dep of ["reqwest", "ureq", "surf", "hyper"]) {
  const re = new RegExp(`^\\s*${dep}\\s*=`, "m");
  if (re.test(cargoToml)) {
    violations.push(`core/Cargo.toml declares forbidden direct dependency "${dep}"`);
  }
}

if (violations.length > 0) {
  console.error("EGRESS_ENFORCEMENT FAIL");
  for (const v of violations) console.error(`- ${v}`);
  process.exit(1);
}

console.log("EGRESS_ENFORCEMENT PASS");

