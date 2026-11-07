#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Pattern, Tuple


@dataclass
class Finding:
    rule_id: str
    message: str
    severity: str
    file_path: str
    start_line: int
    description: str


@dataclass(frozen=True)
class ScanPatterns:
    secret: Pattern[str]
    secret_assignment: Pattern[str]
    network: Pattern[str]
    system: Pattern[str]
    obfuscation: Pattern[str]


SCAN_EXTENSIONS = {
    ".cs", ".csx", ".ps1", ".psm1", ".py", ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs",
    ".sh", ".bash", ".zsh", ".cmd", ".bat", ".yml", ".yaml", ".json", ".toml", ".ini", ".env",
    ".sql", ".rb", ".php", ".java", ".kt", ".go", ".rs",
}


def run_git(args: List[str]) -> str:
    validate_git_args(args)
    result = subprocess.run(
        ["git", *args],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"Git command failed: {result.stderr.strip()}")
    return result.stdout or ""


def validate_git_args(args: List[str]) -> None:
    if not args:
        raise ValueError("At least one git argument is required.")

    for arg in args:
        if not arg or "\x00" in arg or "\n" in arg or "\r" in arg:
            raise ValueError("Invalid git argument.")


def git_changed_files(days: int) -> List[str]:
    out = run_git([
        "log",
        f"--since={days} days ago",
        "--name-only",
        "--pretty=format:",
    ])
    files = sorted({line.strip() for line in out.splitlines() if line.strip()})
    return [f for f in files if Path(f).is_file()]


def is_scannable(path: Path) -> bool:
    suffix = path.suffix.lower()
    if suffix not in SCAN_EXTENSIONS:
        return False
    if any(part.startswith(".") and part not in {".github"} for part in path.parts):
        return False
    return True


def git_added_lines(days: int) -> Dict[str, List[Tuple[int, str]]]:
    out = run_git([
        "log",
        f"--since={days} days ago",
        "--patch",
        "--unified=0",
        "--pretty=format:",
    ])

    file_lines: Dict[str, List[Tuple[int, str]]] = {}
    current_file: str = ""
    current_new_line: int = 0

    for raw in out.splitlines():
        current_file, current_new_line = parse_diff_line(raw, current_file, current_new_line, file_lines)

    return {k: v for k, v in file_lines.items() if v}


def parse_diff_line(raw: str, current_file: str, current_new_line: int, file_lines: Dict[str, List[Tuple[int, str]]]) -> Tuple[str, int]:
    line = raw.rstrip("\n")
    if line.startswith("+++ b/"):
        return select_current_file(line, file_lines), current_new_line

    if line.startswith("@@"):
        return current_file, read_hunk_line_number(line, current_new_line)

    if not current_file:
        return current_file, current_new_line

    if line.startswith("+") and not line.startswith("+++"):
        file_lines[current_file].append((current_new_line, line[1:]))
        return current_file, current_new_line + 1

    if line.startswith("-"):
        return current_file, current_new_line

    return current_file, current_new_line + 1


def select_current_file(line: str, file_lines: Dict[str, List[Tuple[int, str]]]) -> str:
    candidate = line[6:]
    path = Path(candidate)
    if path.is_file() and is_scannable(path):
        file_lines.setdefault(candidate, [])
        return candidate
    return ""


def read_hunk_line_number(line: str, fallback: int) -> int:
    match = re.search(r"\+(\d+)(?:,(\d+))?", line)
    return int(match.group(1)) if match else fallback


def git_recent_commits(days: int) -> List[str]:
    out = run_git([
        "log",
        f"--since={days} days ago",
        "--pretty=format:%h - %an, %ar : %s",
    ])
    return [line for line in out.splitlines() if line.strip()]


def first_match_line(added_lines: List[Tuple[int, str]], pattern: Pattern[str]) -> int:
    for line_number, text in added_lines:
        if pattern.search(text):
            return line_number
    return 1


def map_severity(score: int) -> str:
    if score >= 7:
        return "error"
    if score >= 3:
        return "warning"
    return "note"


def compile_patterns() -> ScanPatterns:
    return ScanPatterns(
        secret=re.compile(r"(secret|token|password|apikey|api[_-]?key|private[_-]?key)", re.IGNORECASE),
        secret_assignment=re.compile(r"(secret|token|password|apikey|api[_-]?key|private[_-]?key)\s*[:=]\s*['\"][^'\"]{8,}", re.IGNORECASE),
        network=re.compile(r"(curl\s|wget\s|http[s]?://|requests\.|fetch\(|http\.get|HttpClient|Invoke-RestMethod)", re.IGNORECASE),
        system=re.compile(r"(Process\.Start|cmd\.exe|powershell\.exe|/bin/sh|subprocess\.|Runtime\.getRuntime\(\)\.exec)", re.IGNORECASE),
        obfuscation=re.compile(r"([A-Za-z0-9+/]{120,}={0,2}|\\x[0-9a-fA-F]{2}\\x[0-9a-fA-F]{2}\\x[0-9a-fA-F]{2,})"),
    )


def analyze_files(added_by_file: Dict[str, List[Tuple[int, str]]]) -> List[Finding]:
    patterns = compile_patterns()
    findings: List[Finding] = []

    for file_path, added_lines in added_by_file.items():
        findings.extend(analyze_added_lines(file_path, added_lines, patterns))

    return deduplicate_findings(findings)


def analyze_added_lines(file_path: str, added_lines: List[Tuple[int, str]], patterns: ScanPatterns) -> List[Finding]:
    added_text = "\n".join(text for _, text in added_lines)
    findings: List[Finding] = []

    has_secret = bool(patterns.secret.search(added_text))
    has_secret_assignment = bool(patterns.secret_assignment.search(added_text))
    has_network = bool(patterns.network.search(added_text))

    if has_secret_assignment and has_network:
        findings.append(secret_exfiltration_finding(file_path, added_lines, patterns))
    if has_network and not has_secret:
        findings.append(network_finding(file_path, added_lines, patterns.network))
    if patterns.system.search(added_text):
        findings.append(system_access_finding(file_path, added_lines, patterns.system))
    if patterns.obfuscation.search(added_text):
        findings.append(obfuscation_finding(file_path, added_lines, patterns.obfuscation))

    return findings


def secret_exfiltration_finding(file_path: str, added_lines: List[Tuple[int, str]], patterns: ScanPatterns) -> Finding:
    return Finding(
        rule_id="malicious-code-scanner/secret-exfiltration",
        message="Potential secret exfiltration pattern detected",
        severity=map_severity(9),
        file_path=file_path,
        start_line=min(first_match_line(added_lines, patterns.secret_assignment), first_match_line(added_lines, patterns.network)),
        description=(
            "Threat score: 9/10. This file contains both secret-related terms and network transfer patterns. "
            "Review if any sensitive values are read and transmitted externally."
        ),
    )


def network_finding(file_path: str, added_lines: List[Tuple[int, str]], pattern: Pattern[str]) -> Finding:
    return Finding(
        rule_id="malicious-code-scanner/suspicious-network",
        message="Unusual network activity pattern in recent changes",
        severity=map_severity(5),
        file_path=file_path,
        start_line=first_match_line(added_lines, pattern),
        description=(
            "Threat score: 5/10. Network-related operations were introduced in recent changes. "
            "Verify destination domains and business justification."
        ),
    )


def system_access_finding(file_path: str, added_lines: List[Tuple[int, str]], pattern: Pattern[str]) -> Finding:
    return Finding(
        rule_id="malicious-code-scanner/system-access",
        message="Suspicious system/process execution pattern",
        severity=map_severity(6),
        file_path=file_path,
        start_line=first_match_line(added_lines, pattern),
        description=(
            "Threat score: 6/10. Process or shell execution pattern detected. "
            "Validate command safety and ensure no user-controlled command injection path exists."
        ),
    )


def obfuscation_finding(file_path: str, added_lines: List[Tuple[int, str]], pattern: Pattern[str]) -> Finding:
    return Finding(
        rule_id="malicious-code-scanner/obfuscation",
        message="Possible obfuscation or encoded payload pattern",
        severity=map_severity(4),
        file_path=file_path,
        start_line=first_match_line(added_lines, pattern),
        description=(
            "Threat score: 4/10. Long encoded/obfuscated-looking strings detected. "
            "Confirm these are expected assets/config blobs and not hidden payloads."
        ),
    )


def deduplicate_findings(findings: List[Finding]) -> List[Finding]:
    dedup = {}
    for f in findings:
        key = (f.rule_id, f.file_path, f.start_line)
        if key not in dedup:
            dedup[key] = f
    return list(dedup.values())


def make_sarif(findings: List[Finding]) -> dict:
    return {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {"driver": make_sarif_driver(findings)},
            "results": [make_sarif_result(f) for f in findings],
        }],
    }


def make_sarif_driver(findings: List[Finding]) -> dict:
    rules = {}
    for f in findings:
        if f.rule_id not in rules:
            rules[f.rule_id] = {
                "id": f.rule_id,
                "name": f.rule_id.split("/")[-1],
                "shortDescription": {"text": f.message},
                "help": {"text": f.description},
                "properties": {"tags": ["security", "malicious-code-scan"]},
            }
    return {"name": "Malicious Code Scanner", "informationUri": "https://github.com", "rules": list(rules.values())}


def make_sarif_result(finding: Finding) -> dict:
    return {
        "ruleId": finding.rule_id,
        "level": finding.severity,
        "message": {"text": finding.message},
        "locations": [{
            "physicalLocation": {
                "artifactLocation": {"uri": finding.file_path.replace('\\', '/')},
                "region": {"startLine": max(1, finding.start_line)},
            }
        }],
    }


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--days", type=int, default=3)
    parser.add_argument("--output", required=True)
    parser.add_argument("--summary", required=True)
    return parser.parse_args()


def run_scan(days: int) -> Tuple[List[str], List[str], List[Finding], dict]:
    commits = git_recent_commits(days)
    files = [f for f in git_changed_files(days) if is_scannable(Path(f))]
    findings = analyze_files(git_added_lines(days))
    return commits, files, findings, make_sarif(findings)


def write_scan_outputs(output: str, summary: str, days: int, commits: List[str], files: List[str], findings: List[Finding], sarif: dict) -> None:
    Path(output).write_text(json.dumps(sarif, indent=2), encoding="utf-8")
    summary_text = build_summary(days, commits, files, findings)
    Path(summary).write_text(summary_text + "\n", encoding="utf-8")
    print(summary_text)


def build_summary(days: int, commits: List[str], files: List[str], findings: List[Finding]) -> str:
    summary_lines = [
        "Daily malicious code scan completed.",
        f"Analysis window: last {days} days",
        f"Commits reviewed: {len(commits)}",
        f"Files analyzed: {len(files)}",
        f"Findings: {len(findings)}",
        "Patterns checked: secret-exfiltration, suspicious-network, system-access, obfuscation",
    ]
    append_commit_summary(summary_lines, commits)
    append_finding_summary(summary_lines, findings)
    return "\n".join(summary_lines)


def append_commit_summary(summary_lines: List[str], commits: List[str]) -> None:
    if commits:
        summary_lines.append("")
        summary_lines.append("Recent commits:")
        summary_lines.extend([f"- {c}" for c in commits[:30]])


def append_finding_summary(summary_lines: List[str], findings: List[Finding]) -> None:
    summary_lines.append("")
    if not findings:
        summary_lines.append("No suspicious patterns detected.")
        return

    summary_lines.append("Findings:")
    for f in findings:
        summary_lines.append(f"- [{f.severity}] {f.rule_id} :: {f.file_path}:{f.start_line}")


def make_error_sarif(message: str) -> dict:
    return {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {"driver": {"name": "Malicious Code Scanner"}},
            "results": [{
                "ruleId": "malicious-code-scanner/system-access",
                "level": "warning",
                "message": {"text": message},
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": {"uri": ".github/scripts/malicious_scan.py"},
                        "region": {"startLine": 1},
                    }
                }],
            }],
        }],
    }


def main() -> int:
    args = parse_args()
    try:
        commits, files, findings, sarif = run_scan(args.days)
        write_scan_outputs(args.output, args.summary, args.days, commits, files, findings, sarif)
        return 0
    except Exception as ex:
        message = f"Scanner execution error: {ex}"
        Path(args.output).write_text(json.dumps(make_error_sarif(message), indent=2), encoding="utf-8")
        Path(args.summary).write_text(f"Scanner failed: {ex}\n", encoding="utf-8")
        print(f"Scanner failed: {ex}")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())
