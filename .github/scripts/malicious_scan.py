#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Tuple


@dataclass
class Finding:
    rule_id: str
    message: str
    severity: str
    file_path: str
    start_line: int
    description: str


SCAN_EXTENSIONS = {
    ".cs", ".csx", ".ps1", ".psm1", ".py", ".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs",
    ".sh", ".bash", ".zsh", ".cmd", ".bat", ".yml", ".yaml", ".json", ".toml", ".ini", ".env",
    ".sql", ".rb", ".php", ".java", ".kt", ".go", ".rs",
}


def run(cmd: List[str]) -> str:
    result = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"Command failed ({' '.join(cmd)}): {result.stderr.strip()}")
    return result.stdout or ""


def git_changed_files(days: int) -> List[str]:
    out = run([
        "git",
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
    out = run([
        "git",
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
        line = raw.rstrip("\n")

        if line.startswith("+++ b/"):
            candidate = line[6:]
            path = Path(candidate)
            if path.is_file() and is_scannable(path):
                current_file = candidate
                file_lines.setdefault(current_file, [])
            else:
                current_file = ""
            continue

        if line.startswith("@@"):
            match = re.search(r"\+(\d+)(?:,(\d+))?", line)
            if match:
                current_new_line = int(match.group(1))
            continue

        if not current_file:
            continue

        if line.startswith("+") and not line.startswith("+++"):
            file_lines[current_file].append((current_new_line, line[1:]))
            current_new_line += 1
        elif line.startswith("-"):
            continue
        else:
            current_new_line += 1

    return {k: v for k, v in file_lines.items() if v}


def git_recent_commits(days: int) -> List[str]:
    out = run([
        "git",
        "log",
        f"--since={days} days ago",
        "--pretty=format:%h - %an, %ar : %s",
    ])
    return [line for line in out.splitlines() if line.strip()]


def first_match_line(added_lines: List[Tuple[int, str]], pattern: re.Pattern) -> int:
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


def analyze_files(added_by_file: Dict[str, List[Tuple[int, str]]]) -> List[Finding]:
    findings: List[Finding] = []

    secret_re = re.compile(r"(secret|token|password|apikey|api[_-]?key|private[_-]?key)", re.IGNORECASE)
    secret_assignment_re = re.compile(r"(secret|token|password|apikey|api[_-]?key|private[_-]?key)\s*[:=]\s*['\"][^'\"]{8,}", re.IGNORECASE)
    network_re = re.compile(r"(curl\s|wget\s|http[s]?://|requests\.|fetch\(|http\.get|HttpClient|Invoke-RestMethod)", re.IGNORECASE)
    system_re = re.compile(r"(Process\.Start|cmd\.exe|powershell\.exe|/bin/sh|subprocess\.|Runtime\.getRuntime\(\)\.exec)", re.IGNORECASE)
    obfuscation_re = re.compile(r"([A-Za-z0-9+/]{120,}={0,2}|\\x[0-9a-fA-F]{2}\\x[0-9a-fA-F]{2}\\x[0-9a-fA-F]{2,})")

    for file_path, added_lines in added_by_file.items():
        added_text = "\n".join(text for _, text in added_lines)

        has_secret = bool(secret_re.search(added_text))
        has_secret_assignment = bool(secret_assignment_re.search(added_text))
        has_network = bool(network_re.search(added_text))

        if has_secret_assignment and has_network:
            findings.append(Finding(
                rule_id="malicious-code-scanner/secret-exfiltration",
                message="Potential secret exfiltration pattern detected",
                severity=map_severity(9),
                file_path=file_path,
                start_line=min(first_match_line(added_lines, secret_assignment_re), first_match_line(added_lines, network_re)),
                description=(
                    "Threat score: 9/10. This file contains both secret-related terms and network transfer patterns. "
                    "Review if any sensitive values are read and transmitted externally."
                ),
            ))

        if has_network and not has_secret:
            findings.append(Finding(
                rule_id="malicious-code-scanner/suspicious-network",
                message="Unusual network activity pattern in recent changes",
                severity=map_severity(5),
                file_path=file_path,
                start_line=first_match_line(added_lines, network_re),
                description=(
                    "Threat score: 5/10. Network-related operations were introduced in recent changes. "
                    "Verify destination domains and business justification."
                ),
            ))

        if system_re.search(added_text):
            findings.append(Finding(
                rule_id="malicious-code-scanner/system-access",
                message="Suspicious system/process execution pattern",
                severity=map_severity(6),
                file_path=file_path,
                start_line=first_match_line(added_lines, system_re),
                description=(
                    "Threat score: 6/10. Process or shell execution pattern detected. "
                    "Validate command safety and ensure no user-controlled command injection path exists."
                ),
            ))

        if obfuscation_re.search(added_text):
            findings.append(Finding(
                rule_id="malicious-code-scanner/obfuscation",
                message="Possible obfuscation or encoded payload pattern",
                severity=map_severity(4),
                file_path=file_path,
                start_line=first_match_line(added_lines, obfuscation_re),
                description=(
                    "Threat score: 4/10. Long encoded/obfuscated-looking strings detected. "
                    "Confirm these are expected assets/config blobs and not hidden payloads."
                ),
            ))

    dedup = {}
    for f in findings:
        key = (f.rule_id, f.file_path, f.start_line)
        if key not in dedup:
            dedup[key] = f
    return list(dedup.values())


def make_sarif(findings: List[Finding]) -> dict:
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

    results = []
    for f in findings:
        results.append({
            "ruleId": f.rule_id,
            "level": f.severity,
            "message": {"text": f.message},
            "locations": [{
                "physicalLocation": {
                    "artifactLocation": {"uri": f.file_path.replace('\\\\', '/')},
                    "region": {"startLine": max(1, f.start_line)},
                }
            }],
        })

    return {
        "version": "2.1.0",
        "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
        "runs": [{
            "tool": {
                "driver": {
                    "name": "Malicious Code Scanner",
                    "informationUri": "https://github.com",
                    "rules": list(rules.values()),
                }
            },
            "results": results,
        }],
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--days", type=int, default=3)
    parser.add_argument("--output", required=True)
    parser.add_argument("--summary", required=True)
    args = parser.parse_args()

    try:
        commits = git_recent_commits(args.days)
        files = [f for f in git_changed_files(args.days) if is_scannable(Path(f))]
        added_lines = git_added_lines(args.days)
        findings = analyze_files(added_lines)
        sarif = make_sarif(findings)

        Path(args.output).write_text(json.dumps(sarif, indent=2), encoding="utf-8")

        summary_lines = [
            "Daily malicious code scan completed.",
            f"Analysis window: last {args.days} days",
            f"Commits reviewed: {len(commits)}",
            f"Files analyzed: {len(files)}",
            f"Findings: {len(findings)}",
            "Patterns checked: secret-exfiltration, suspicious-network, system-access, obfuscation",
        ]

        if commits:
            summary_lines.append("")
            summary_lines.append("Recent commits:")
            summary_lines.extend([f"- {c}" for c in commits[:30]])

        if findings:
            summary_lines.append("")
            summary_lines.append("Findings:")
            for f in findings:
                summary_lines.append(f"- [{f.severity}] {f.rule_id} :: {f.file_path}:{f.start_line}")
        else:
            summary_lines.append("")
            summary_lines.append("✅ No suspicious patterns detected.")

        Path(args.summary).write_text("\n".join(summary_lines) + "\n", encoding="utf-8")
        print("\n".join(summary_lines))
        return 0
    except Exception as ex:
        fallback = {
            "version": "2.1.0",
            "$schema": "https://json.schemastore.org/sarif-2.1.0.json",
            "runs": [{
                "tool": {"driver": {"name": "Malicious Code Scanner"}},
                "results": [{
                    "ruleId": "malicious-code-scanner/system-access",
                    "level": "warning",
                    "message": {"text": f"Scanner execution error: {ex}"},
                    "locations": [{
                        "physicalLocation": {
                            "artifactLocation": {"uri": ".github/scripts/malicious_scan.py"},
                            "region": {"startLine": 1},
                        }
                    }],
                }],
            }],
        }
        Path(args.output).write_text(json.dumps(fallback, indent=2), encoding="utf-8")
        Path(args.summary).write_text(f"Scanner failed: {ex}\n", encoding="utf-8")
        print(f"Scanner failed: {ex}")
        return 0


if __name__ == "__main__":
    raise SystemExit(main())

