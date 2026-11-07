"""Fail-closed tests for the stable GitHub release helper; no network access."""

import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
VERSION = (ROOT / "VERSION").read_text().strip()
SHA = "a" * 40

MOCK_CURL = r'''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
out = pathlib.Path(args[args.index('--output') + 1])
url = args[-1]
state = pathlib.Path(os.environ['MOCK_STATE'])
scenario = os.environ['MOCK_SCENARIO']
created = (state / 'created.json').exists()
code, payload = 200, {}
if url.endswith('/releases/latest'):
    if scenario == 'network_error':
        sys.exit(7)
    if scenario == 'latest_error':
        code = 500
    else:
        payload = {'tag_name': 'v9.9.9' if scenario == 'regression' else 'v1.7.1', 'draft': False, 'prerelease': False}
elif '/releases/tags/' in url:
    # GitHub's published-release endpoint never returns drafts.
    code = 404
elif '/git/ref/heads/main' in url:
    counter = state / 'main-count'
    count = int(counter.read_text()) if counter.exists() else 0
    counter.write_text(str(count + 1))
    mismatch = scenario == 'head_mismatch' or (scenario == 'head_changes' and count > 0)
    payload = {'object': {'type': 'commit', 'sha': ('b' * 40 if mismatch else os.environ['GITHUB_SHA'])}}
elif '/git/ref/tags/' in url:
    if scenario == 'tag_exists' or (state / 'ref-created').exists():
        payload = {'object': {'type': 'commit', 'sha': ('b' * 40 if scenario == 'wrong_tag' else os.environ['GITHUB_SHA'])}}
    else:
        code = 404
else:
    code = 500
out.write_text(json.dumps(payload))
print(code, end='')
'''

MOCK_GH = r'''#!/usr/bin/env python3
import hashlib, json, os, pathlib, sys
args = sys.argv[1:]
state = pathlib.Path(os.environ['MOCK_STATE'])
scenario = os.environ['MOCK_SCENARIO']
if args[:2] == ['api', '--paginate']:
    if scenario == 'api_error':
        sys.exit(1)
    if scenario in ('existing', 'draft', 'prerelease'):
        print(json.dumps({'tag_name': 'v' + os.environ['MOCK_VERSION'], 'draft': scenario == 'draft', 'prerelease': scenario == 'prerelease'}))
    elif (state / 'created.json').exists():
        payload = json.loads((state / 'created.json').read_text())
        if scenario == 'bad_digest':
            payload['assets'][0]['digest'] = 'sha256:' + ('0' * 64)
        elif scenario == 'missing_digest':
            payload['assets'][0]['digest'] = None
        elif scenario == 'pending_digest':
            counter = state / 'digest-count'
            count = int(counter.read_text()) if counter.exists() else 0
            counter.write_text(str(count + 1))
            if count < 2:
                payload['assets'][0]['digest'] = None
        print(json.dumps(payload))
    sys.exit(0)
with (state / 'calls').open('a') as calls:
    calls.write(json.dumps(args) + '\n')
if args[:3] == ['api', '--method', 'POST']:
    (state / 'ref-created').write_text('yes')
elif args[:2] == ['release', 'create']:
    if not (state / 'ref-created').exists() or '--verify-tag' not in args:
        sys.exit(2)
    paths = [pathlib.Path(x) for x in args[3:args.index('--repo')]]
    assets = []
    for path in paths:
        with path.open('rb') as f:
            digest = hashlib.file_digest(f, 'sha256').hexdigest()
        assets.append({'name': path.name, 'digest': 'sha256:' + digest, 'state': 'uploaded', 'size': path.stat().st_size})
    (state / 'created.json').write_text(json.dumps({'tag_name': args[2], 'draft': True, 'prerelease': False, 'assets': assets}))
elif args[:2] != ['release', 'edit']:
    sys.exit(2)
'''


class PublishReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="greg-release-test-")
        self.addCleanup(self.temp.cleanup)
        self.base = Path(self.temp.name)
        self.bin = self.base / "bin"
        self.bin.mkdir()
        self.artifacts = self.base / "artifacts"
        self.artifacts.mkdir()
        self.state = self.base / "state"
        self.state.mkdir()
        for name, contents in (("curl", MOCK_CURL), ("gh", MOCK_GH), ("sleep", "#!/bin/sh\nexit 0\n")):
            path = self.bin / name
            path.write_text(contents)
            path.chmod(0o755)
        self.names = [
            f"gregmodmanager-{VERSION}-win-x64-setup.exe",
            f"gregmodmanager-{VERSION}-win-x64.msi",
            f"gregmodmanager-{VERSION}-win-x64.zip",
            f"gregmodmanager-{VERSION}-linux-x64.tar.gz",
            f"gregmodmanager-{VERSION}-linux-x64.deb",
            f"gregmodmanager-{VERSION}-linux-x64.rpm",
            f"gregmodmanager-{VERSION}-linux-x64.pkg.tar.zst",
        ]
        for name in self.names:
            data = name.encode()
            (self.artifacts / name).write_bytes(data)
            (self.artifacts / (name + ".sha256")).write_text(f"{hashlib.sha256(data).hexdigest()}  {name}\n")

    def run_helper(self, scenario="success", version=VERSION):
        env = os.environ.copy()
        env.update({"PATH": str(self.bin) + os.pathsep + env["PATH"], "GH_TOKEN": "test-token", "GITHUB_REPOSITORY": "owner/repo", "GITHUB_SHA": SHA, "MOCK_STATE": str(self.state), "MOCK_SCENARIO": scenario, "MOCK_VERSION": VERSION})
        return subprocess.run(["bash", str(ROOT / "scripts/publish-github-release.sh"), version, str(self.artifacts)], cwd=ROOT, env=env, text=True, capture_output=True)

    def calls(self):
        path = self.state / "calls"
        return [json.loads(line) for line in path.read_text().splitlines()] if path.exists() else []

    def test_success_verifies_then_publishes_only_expected_assets(self):
        result = self.run_helper()
        self.assertEqual(result.returncode, 0, result.stderr)
        ref, create, edit = self.calls()
        self.assertEqual(ref[:3], ['api', '--method', 'POST'])
        self.assertIn("--draft", create)
        self.assertIn("--verify-tag", create)
        self.assertIn("sha=" + SHA, ref)
        names = {Path(value).name for value in create[3:create.index("--repo")]}
        self.assertEqual(names, {name for item in self.names for name in (item, item + ".sha256")})
        self.assertIn("--draft=false", edit)
        self.assertIn("--prerelease=false", edit)

    def test_existing_stable_release_is_left_unchanged(self):
        result = self.run_helper("existing")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(self.calls(), [])

    def test_waits_for_delayed_digest(self):
        result = self.run_helper("pending_digest")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual((self.state / "digest-count").read_text(), "3")

    def test_existing_draft_prerelease_tag_and_api_failure_are_rejected(self):
        for scenario in ("draft", "prerelease", "tag_exists", "api_error", "latest_error", "network_error"):
            with self.subTest(scenario=scenario):
                result = self.run_helper(scenario)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(self.calls(), [])

    def test_invalid_version_and_checksum_are_rejected(self):
        self.assertNotEqual(self.run_helper(version=VERSION + "-rc.1").returncode, 0)
        (self.artifacts / self.names[0]).write_bytes(b"tampered")
        self.assertNotEqual(self.run_helper().returncode, 0)
        self.assertEqual(self.calls(), [])

    def test_version_regression_and_changed_main_are_rejected(self):
        for scenario in ("regression", "head_mismatch"):
            with self.subTest(scenario=scenario):
                self.assertNotEqual(self.run_helper(scenario).returncode, 0)
                self.assertEqual(self.calls(), [])

    def test_wrong_tag_or_bad_asset_stays_draft(self):
        for scenario in ("wrong_tag", "bad_digest", "missing_digest", "head_changes"):
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory(prefix="greg-release-state-") as state:
                self.state = Path(state)
                result = self.run_helper(scenario)
                self.assertNotEqual(result.returncode, 0)
                expected = [["api", "--method"]]
                if scenario != "wrong_tag":
                    expected.append(["release", "create"])
                self.assertEqual([call[:2] for call in self.calls()], expected)


if __name__ == "__main__":
    unittest.main()
