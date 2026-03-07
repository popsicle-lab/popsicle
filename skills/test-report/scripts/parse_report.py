#!/usr/bin/env python3
"""
Test report parser — normalizes pytest (JUnit XML), cargo test (log),
and Playwright (JSON) reports into a unified JSON format.

Usage:
    python3 parse_report.py --junit reports/acceptance.xml
    python3 parse_report.py --cargo-log reports/boundary.log
    python3 parse_report.py --playwright-json reports/playwright-report/results.json
    python3 parse_report.py --junit reports/acceptance.xml --cargo-log reports/boundary.log
"""

import argparse
import json
import re
import sys
import xml.etree.ElementTree as ET
from datetime import datetime, timezone
from pathlib import Path


def parse_junit_xml(path: str) -> dict:
    tree = ET.parse(path)
    root = tree.getroot()

    suites = root.findall(".//testsuite") if root.tag == "testsuites" else [root]

    total = 0
    pass_count = 0
    fail_count = 0
    skip_count = 0
    xfail_count = 0
    xpass_count = 0
    failures = []
    xpasses = []

    for suite in suites:
        for tc in suite.findall("testcase"):
            total += 1
            classname = tc.get("classname", "")
            name = tc.get("name", "")
            file_attr = tc.get("file", "")

            failure = tc.find("failure")
            error = tc.find("error")
            skipped = tc.find("skipped")

            module = _extract_module(classname, file_attr)

            if skipped is not None:
                msg = skipped.get("message", "")
                if "xfail" in msg.lower():
                    xfail_count += 1
                elif "xpass" in msg.lower() or "XPASS" in msg:
                    xpass_count += 1
                    xpasses.append({
                        "test_name": name,
                        "test_file": file_attr,
                        "module": module,
                        "message": msg,
                    })
                else:
                    skip_count += 1
            elif failure is not None or error is not None:
                fail_count += 1
                elem = failure if failure is not None else error
                failures.append({
                    "test_name": name,
                    "test_file": file_attr,
                    "module": module,
                    "error_message": (elem.get("message", "") or elem.text or "")[:500],
                    "severity_hint": _infer_severity(classname, name),
                })
            else:
                properties = tc.find("properties")
                if properties is not None:
                    for prop in properties.findall("property"):
                        if prop.get("name") == "pytest_mark" and "xpass" in prop.get("value", "").lower():
                            xpass_count += 1
                            xpasses.append({
                                "test_name": name,
                                "test_file": file_attr,
                                "module": module,
                                "message": "XPASS detected via property",
                            })
                            break
                    else:
                        pass_count += 1
                else:
                    pass_count += 1

    return _build_result("pytest", total, pass_count, fail_count, skip_count,
                         xfail_count, xpass_count, failures, xpasses)


def parse_cargo_log(path: str) -> dict:
    text = Path(path).read_text(encoding="utf-8", errors="replace")

    total = 0
    pass_count = 0
    fail_count = 0
    skip_count = 0
    failures = []

    summary_match = re.search(
        r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored",
        text,
    )
    if summary_match:
        pass_count = int(summary_match.group(1))
        fail_count = int(summary_match.group(2))
        skip_count = int(summary_match.group(3))
        total = pass_count + fail_count + skip_count

    fail_pattern = re.compile(r"---- ([\w:]+) stdout ----")
    failure_blocks = fail_pattern.split(text)

    failed_tests_section = re.search(r"failures:\n((?:\s+[\w:]+\n)+)", text)
    failed_names = set()
    if failed_tests_section:
        for line in failed_tests_section.group(1).strip().split("\n"):
            failed_names.add(line.strip())

    for name in failed_names:
        module = _extract_rust_module(name)
        error_msg = ""
        for i in range(1, len(failure_blocks), 2):
            if failure_blocks[i - 1].strip().endswith(name) or failure_blocks[i - 1].strip() == name:
                error_msg = failure_blocks[i][:500]
                break

        failures.append({
            "test_name": name.split("::")[-1] if "::" in name else name,
            "test_file": _guess_test_file(name),
            "module": module,
            "error_message": error_msg.strip(),
            "severity_hint": _infer_severity_rust(name),
        })

    return _build_result("cargo-test", total, pass_count, fail_count, skip_count,
                         0, 0, failures, [])


def parse_playwright_json(path: str) -> dict:
    data = json.loads(Path(path).read_text(encoding="utf-8"))

    total = 0
    pass_count = 0
    fail_count = 0
    skip_count = 0
    failures = []

    suites = data.get("suites", [])
    for suite in suites:
        _walk_playwright_suite(suite, failures,
                               {"total": 0, "pass": 0, "fail": 0, "skip": 0})

    stats = data.get("stats", {})
    if stats:
        total = stats.get("expected", 0) + stats.get("unexpected", 0) + stats.get("skipped", 0)
        pass_count = stats.get("expected", 0)
        fail_count = stats.get("unexpected", 0)
        skip_count = stats.get("skipped", 0)
    else:
        total = pass_count + fail_count + skip_count

    return _build_result("playwright", total, pass_count, fail_count, skip_count,
                         0, 0, failures, [])


def _walk_playwright_suite(suite: dict, failures: list, counters: dict):
    for spec in suite.get("specs", []):
        for test in spec.get("tests", []):
            for result in test.get("results", []):
                status = result.get("status", "")
                if status == "failed" or status == "timedOut":
                    error_msg = ""
                    if result.get("error", {}).get("message"):
                        error_msg = result["error"]["message"][:500]
                    failures.append({
                        "test_name": spec.get("title", "unknown"),
                        "test_file": spec.get("file", suite.get("title", "")),
                        "module": "ui",
                        "error_message": error_msg,
                        "severity_hint": "critical",
                    })
    for child in suite.get("suites", []):
        _walk_playwright_suite(child, failures, counters)


def _extract_module(classname: str, file_path: str) -> str:
    if file_path:
        match = re.search(r"domain_(\w+)", file_path)
        if match:
            return f"domain_{match.group(1)}"
        if "stories" in file_path:
            return "stories"
    if classname:
        match = re.search(r"domain_(\w+)", classname)
        if match:
            return f"domain_{match.group(1)}"
    return "unknown"


def _extract_rust_module(full_name: str) -> str:
    parts = full_name.split("::")
    if len(parts) >= 2:
        return parts[0]
    return "unknown"


def _guess_test_file(full_name: str) -> str:
    parts = full_name.split("::")
    if len(parts) >= 2:
        crate = parts[0].replace("-", "_").replace("_", "-")
        return f"crates/{crate}/src/..."
    return ""


def _infer_severity(classname: str, name: str) -> str:
    combined = f"{classname}.{name}".lower()
    if any(k in combined for k in ("p0", "critical", "auth", "rbac", "tenant", "isolation")):
        return "critical"
    if any(k in combined for k in ("p1", "quota", "boundary")):
        return "high"
    return "high"


def _infer_severity_rust(name: str) -> str:
    lower = name.lower()
    if any(k in lower for k in ("auth", "tenant", "rbac", "isolation", "encrypt")):
        return "critical"
    if any(k in lower for k in ("engine", "meta", "query", "hlc")):
        return "critical"
    return "high"


def _build_result(framework, total, passed, failed, skipped,
                  xfail, xpass, failures, xpasses) -> dict:
    return {
        "framework": framework,
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "summary": {
            "total": total,
            "pass": passed,
            "fail": failed,
            "skip": skipped,
            "xfail": xfail,
            "xpass": xpass,
        },
        "failures": failures,
        "xpasses": xpasses,
    }


def main():
    parser = argparse.ArgumentParser(description="Unified test report parser")
    parser.add_argument("--junit", action="append", default=[],
                        help="Path to JUnit XML report (repeatable)")
    parser.add_argument("--cargo-log", action="append", default=[],
                        help="Path to cargo test log file (repeatable)")
    parser.add_argument("--playwright-json", action="append", default=[],
                        help="Path to Playwright JSON report (repeatable)")
    parser.add_argument("-o", "--output", help="Output JSON file (default: stdout)")
    args = parser.parse_args()

    results = []

    for path in args.junit:
        if not Path(path).exists():
            print(f"Warning: {path} not found, skipping", file=sys.stderr)
        else:
            results.append(parse_junit_xml(path))

    for path in args.cargo_log:
        if not Path(path).exists():
            print(f"Warning: {path} not found, skipping", file=sys.stderr)
        else:
            results.append(parse_cargo_log(path))

    for path in args.playwright_json:
        if not Path(path).exists():
            print(f"Warning: {path} not found, skipping", file=sys.stderr)
        else:
            results.append(parse_playwright_json(path))

    if not results:
        print("Error: no valid report files provided", file=sys.stderr)
        sys.exit(1)

    merged = {
        "parsed_at": datetime.now(timezone.utc).isoformat(),
        "reports": results,
        "combined_summary": {
            "total": sum(r["summary"]["total"] for r in results),
            "pass": sum(r["summary"]["pass"] for r in results),
            "fail": sum(r["summary"]["fail"] for r in results),
            "skip": sum(r["summary"]["skip"] for r in results),
            "xfail": sum(r["summary"]["xfail"] for r in results),
            "xpass": sum(r["summary"]["xpass"] for r in results),
        },
        "all_failures": [f for r in results for f in r["failures"]],
        "all_xpasses": [x for r in results for x in r["xpasses"]],
    }

    output = json.dumps(merged, indent=2, ensure_ascii=False)
    if args.output:
        Path(args.output).write_text(output, encoding="utf-8")
        print(f"Report written to {args.output}", file=sys.stderr)
    else:
        print(output)


if __name__ == "__main__":
    main()
