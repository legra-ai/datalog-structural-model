"""Packaged validation must retain the checkout's selected compiler."""

import importlib.util
import io
import pathlib
import subprocess
import tarfile
import unittest
from unittest.mock import patch


class PackageToolchainTests(unittest.TestCase):
    def test_extracted_archive_retains_active_toolchain(self):
        source = pathlib.Path(__file__).with_name("test-package.py")
        spec = importlib.util.spec_from_file_location("package_validation", source)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        calls = []

        def output(command, **kwargs):
            if command == ["git", "ls-files", "tests/"]:
                return "tests/fixture.rs\n"
            if command == ["rustup", "show", "active-toolchain"]:
                return "1.99.1-test-host (directory override)\n"
            raise AssertionError(command)

        def run(command, **kwargs):
            calls.append((command, kwargs))
            if command[:2] == ["cargo", "package"]:
                target = pathlib.Path(command[command.index("--target-dir") + 1])
                package = module.tomllib.loads((module.pathlib.Path(module.__file__).parents[1] / "Cargo.toml").read_text())["package"]
                stem = f"{package['name']}-{package['version']}"
                archive_path = target / "package" / f"{stem}.crate"
                archive_path.parent.mkdir()
                with tarfile.open(archive_path, "w:gz") as archive:
                    content = b"// fixture"
                    entry = tarfile.TarInfo(f"{stem}/tests/fixture.rs")
                    entry.size = len(content)
                    archive.addfile(entry, io.BytesIO(content))
            return subprocess.CompletedProcess(command, 0)

        with patch.object(module.subprocess, "check_output", side_effect=output), patch.object(module.subprocess, "run", side_effect=run):
            module.main()
        tests = [kwargs for command, kwargs in calls if command[:2] == ["cargo", "test"]]
        self.assertEqual(len(tests), 2)
        for kwargs in tests:
            self.assertEqual(kwargs.get("env", {}).get("RUSTUP_TOOLCHAIN"), "1.99.1-test-host")


if __name__ == "__main__":
    unittest.main()
