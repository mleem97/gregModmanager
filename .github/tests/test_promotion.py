"""Promotion contract regressions (Python 3.11+ + PyYAML, no network or real gates)."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import yaml

ROOT = Path(__file__).resolve().parents[2]
WORKFLOW = yaml.safe_load((ROOT / '.github/workflows/greg-promote.yml').read_text())
EVAL = next(s['run'] for s in WORKFLOW['jobs']['gates']['steps'] if s.get('id') == 'eval')

class PromotionTests(unittest.TestCase):
    def evaluate(self, body, expected):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            subprocess.run(['git', 'init', '-q', tmp], check=True)
            subprocess.run(['git', '-c', 'user.name=Test', '-c', 'user.email=test@example.com',
                            'commit', '-qm', 'ci: fixture', '--allow-empty'], cwd=tmp, check=True)
            sha = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=tmp, text=True).strip()
            script = root / '.github/scripts/greg-gates.sh'
            if body is not None:
                script.parent.mkdir(parents=True)
                script.write_text(body)
            # Stale success from a previous runner job must never be accepted.
            (root / 'gate-report.txt').write_text('OVERALL: PASS\n')
            output = root / 'output'
            output.touch()
            env = dict(os.environ, GITHUB_WORKSPACE=tmp, GITHUB_OUTPUT=str(output))
            run = subprocess.run(['bash', '-e', '-c', EVAL], cwd=tmp, env=env, capture_output=True, text=True)
            self.assertEqual(run.returncode == 0, expected == 'pass', run.stderr)
            self.assertEqual(output.read_text(), f'verdict={expected}\nsha={sha}\n')
            if expected == 'fail':
                self.assertIn('OVERALL: FAIL', (root / 'gate-report.txt').read_text())

    def test_evaluator_contract(self):
        success = '''echo 'OVERALL: PASS' > gate-report.txt
printf 'verdict=pass\nsha=%s\n' "$(git rev-parse HEAD)" >> "$GITHUB_OUTPUT"
'''
        cases = [
            (None, 'fail'), ('exit 127', 'fail'), ('exit 0', 'fail'),
            (success, 'pass'), (success + 'exit 127\n', 'fail'),
            (success.replace('verdict=pass', 'verdict=fail'), 'fail'),
            (success.replace('$(git rev-parse HEAD)', 'wrong'), 'fail'),
            (success + 'rm gate-report.txt\n', 'fail'),
            (success + 'echo verdict=pass >> "$GITHUB_OUTPUT"\n', 'fail'),
            (success + 'echo "GATE build: FAIL" >> gate-report.txt\n', 'fail'),
        ]
        for body, verdict in cases:
            with self.subTest(body=body):
                self.evaluate(body, verdict)

    def test_real_gate_with_stubbed_tools(self):
        # Exercise the actual gate, including worktree baseline and tool failures.
        for scenario in ['pass', 'coverage-error', 'baseline-error', 'low-coverage', 'tests-error', 'secrets-error', 'audit-error']:
            with self.subTest(scenario=scenario), tempfile.TemporaryDirectory() as tmp:
                root = Path(tmp)
                repo = root / 'repo'
                repo.mkdir()
                def git(*args):
                    return subprocess.run(['git', *args], cwd=repo, check=True, capture_output=True, text=True)
                git('init', '-q')
                (repo / 'Cargo.toml').write_text('[workspace.package]\nversion = "1.0.0"\n')
                (repo / 'Cargo.lock').write_text('version = 4\n')
                (repo / 'VERSION').write_text('1.0.0\n')
                git('add', '.')
                git('-c', 'user.name=Test', '-c', 'user.email=test@example.com', 'commit', '-qm', 'ci: fixture')
                subprocess.run(['git', 'clone', '-q', '--bare', str(repo), str(root / 'remote')], check=True)
                git('remote', 'add', 'origin', str(root / 'remote'))
                git('push', '-q', 'origin', 'HEAD:refs/heads/staging')
                bindir = root / 'bin'
                bindir.mkdir()
                cargo = bindir / 'cargo'
                cargo.write_text('''#!/bin/bash
case "$1:$SCENARIO" in
  test:tests-error|audit:audit-error|tarpaulin:coverage-error) exit 2 ;;
esac
if [ "$1" = tarpaulin ]; then
  if [ "$SCENARIO" = baseline-error ] && [ "$PWD" != "$DEV_ROOT" ]; then exit 2; fi
  if [ "$SCENARIO" = low-coverage ]; then echo '80.00% coverage'; else echo '95.00% coverage'; fi
fi
exit 0
''')
                leaks = bindir / 'gitleaks'
                leaks.write_text('#!/bin/bash\n[ "$SCENARIO" != secrets-error ]\n')
                for tool in [cargo, leaks]:
                    tool.chmod(0o755)
                (bindir / 'cargo-tarpaulin').symlink_to(cargo)
                output = root / 'output'
                output.touch()
                env = dict(os.environ, PATH=str(bindir) + os.pathsep + os.environ['PATH'],
                           GITHUB_OUTPUT=str(output), STAGE='staging', KIND='rust',
                           SCENARIO=scenario, DEV_ROOT=str(repo))
                run = subprocess.run(['bash', str(ROOT / '.github/scripts/greg-gates.sh')], cwd=repo,
                                     env=env, capture_output=True, text=True)
                expected = 'pass' if scenario == 'pass' else 'fail'
                self.assertIn('verdict=' + expected, output.read_text(), run.stdout + run.stderr)

    def test_workflow_handoff_and_restrictions(self):
        jobs = WORKFLOW['jobs']
        condition = jobs['promote']['if']
        for required in ["needs.gates.result == 'success'", "needs.gates.outputs.verdict == 'pass'",
                         "github.actor == 'mleem97'", 'inputs.dry_run == false']:
            self.assertIn(required, condition)
        checkout = jobs['gates']['steps'][0]['with']
        self.assertIn('github.sha', checkout['ref'])
        self.assertIn("'dev'", checkout['ref'])
        upload = next(s for s in jobs['gates']['steps'] if 'upload-artifact' in s.get('uses', ''))
        download = next(s for s in jobs['sync-issue']['steps'] if 'download-artifact' in s.get('uses', ''))
        self.assertEqual(upload['with']['name'], download['with']['name'])
        self.assertEqual(upload['if'], 'always()')
        self.assertEqual(upload['with']['if-no-files-found'], 'error')
        self.assertIn('github.run_attempt', upload['with']['name'])
        issue = jobs['sync-issue']['steps'][-1]['run']
        self.assertIn('"$REPORT_OUTCOME" = success', issue)
        self.assertIn('"$PROMOTE_RESULT" = success', issue)
        self.assertIn('gh issue edit "$OPEN" --body-file', issue)
        self.assertNotIn('no report', issue)
        for job in jobs.values():
            for step in job['steps']:
                if 'run' in step:
                    subprocess.run(['bash', '-n'], input=step['run'], text=True, check=True)

    def test_preserved_gates(self):
        script = (ROOT / '.github/scripts/greg-gates.sh').read_text()
        for gate in ['build', 'tests', 'coverage', 'security-secrets', 'security-deps',
                     'security-warnings', 'conventional']:
            self.assertIn('gfail ' + gate, script)
        self.assertIn('GATE consistency: FAIL', script)
        self.assertIn('no passing tests to measure (fail closed)', script)
        self.assertIn('measurement or baseline failed', script)
        subprocess.run(['bash', '-n', str(ROOT / '.github/scripts/greg-gates.sh')], check=True)

if __name__ == '__main__':
    unittest.main()
