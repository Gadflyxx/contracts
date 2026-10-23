"""Deployment orchestration tests. The CLI is stubbed; no RPC is contacted."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[1]


class DeploymentTests(unittest.TestCase):
    def run_deploy(self, network="testnet", component="core", missing=None, fail=False, toolchain="stable"):
        with tempfile.TemporaryDirectory() as directory:
            tmp = Path(directory)
            cli = tmp / "stellar"
            cli.write_text("""#!/usr/bin/env python3
import json, os, sys
with open(os.environ['CLI_LOG'], 'a') as f:
    f.write(json.dumps(sys.argv[1:]) + '\\n')
if sys.argv[1:3] == ['contract', 'build']:
    with open(os.environ['BUILD_TOOLCHAIN_LOG'], 'w') as f:
        f.write(os.environ.get('RUSTUP_TOOLCHAIN', ''))
if os.environ.get('FAIL_BUILD') and sys.argv[1:3] == ['contract', 'build']:
    sys.exit(1)
if sys.argv[1:3] == ['contract', 'deploy']:
    print('C_TEST_CONTRACT_ID')
""")
            cli.chmod(0o755)
            env = {**os.environ, "STELLAR_CLI": str(cli), "CLI_LOG": str(tmp / "calls"),
                   "SOURCE_ACCOUNT": "deployer", "ADMIN": "admin",
                   "BUILD_TOOLCHAIN_LOG": str(tmp / "toolchain"),
                   "RUSTUP_TOOLCHAIN": toolchain,
                   "ACCOUNT_CONFIG": '{"threshold":1}', "ROOT_HASH": "00" * 32,
                   "TOKEN": "asset", "FUNDER": "funder", "OWNER": "owner",
                   "MAX_SUPPLY": "10", "METADATA": '{"name":"RemitPulse"}'}
            if missing:
                env.pop(missing, None)
            if fail:
                env["FAIL_BUILD"] = "1"
            before = set((ROOT / "deployments").glob("*.env"))
            result = subprocess.run([str(ROOT / "deploy.sh"), network, component],
                                    env=env, text=True, capture_output=True)
            calls = [json.loads(line) for line in (tmp / "calls").read_text().splitlines()] if (tmp / "calls").exists() else []
            if calls and calls[0] == ["contract", "build"]:
                expected = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
                self.assertEqual((tmp / "toolchain").read_text(), expected)
            records = set((ROOT / "deployments").glob("*.env")) - before
            saved = [p.read_text() for p in records]
            for p in records:
                p.unlink()
            return result, calls, saved

    def test_build_entry_points_override_inherited_toolchain(self):
        expected = tomllib.loads((ROOT / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        with tempfile.TemporaryDirectory() as directory:
            tmp = Path(directory)
            cli = tmp / "stellar"
            cli.write_text("#!/bin/sh\nprintf '%s' \"$RUSTUP_TOOLCHAIN\" > \"$BUILD_TOOLCHAIN_LOG\"\n")
            cli.chmod(0o755)
            for inherited in ("stable", "1.85.0"):
                env = {**os.environ, "STELLAR_CLI": str(cli), "RUSTUP_TOOLCHAIN": inherited,
                       "BUILD_TOOLCHAIN_LOG": str(tmp / "toolchain")}
                for command in ([str(ROOT / "scripts/build.sh")],
                                ["make", "-C", str(ROOT), "build"]):
                    result = subprocess.run(command, cwd=tmp, env=env, text=True, capture_output=True)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual((tmp / "toolchain").read_text(), expected)

    def test_core_constructor_arguments_and_network(self):
        result, calls, records = self.run_deploy("mainnet")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(calls[0], ["contract", "build"])
        self.assertEqual(len(calls), 5)
        for call in calls[1:]:
            self.assertEqual(call[call.index("--network") + 1], "mainnet")
        self.assertEqual(calls[3][-3:], ["--", "--config", '{"threshold":1}'])
        self.assertEqual(calls[4][-3:], ["--", "--admin", "admin"])
        self.assertIn("REMITPULSE_ACCOUNT=C_TEST_CONTRACT_ID", records[0])

    def test_preflight_and_build_failures_never_deploy(self):
        result, calls, _ = self.run_deploy(missing="ACCOUNT_CONFIG")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])
        result, calls, _ = self.run_deploy(fail=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [["contract", "build"]])
        result, calls, _ = self.run_deploy(network="unknown")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(calls, [])

    def test_optional_contract_constructors(self):
        for component, arguments in [("airdrop", ["--root_hash", "--token", "--admin", "--funder"]),
                                     ("nft", ["--owner", "--max_supply", "--metadata"])]:
            result, calls, _ = self.run_deploy(component=component)
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(len(calls), 2)
            for arg in arguments:
                self.assertIn(arg, calls[1])


if __name__ == "__main__":
    unittest.main()
