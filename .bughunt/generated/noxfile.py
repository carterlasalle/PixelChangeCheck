from __future__ import annotations

import os
import secrets
from pathlib import Path
import nox

PYTHONS = ['3.11', '3.12', '3.13', '3.14']
TEST_PATHS = ['tests']

# trace:exempt reason=generated-by-bughunt-configure-do-not-hand-edit
@nox.session(python=PYTHONS, venv_backend="uv|virtualenv")
def tests(session):
    # Nox invokes this file from .bughunt/generated/; anchor every
    # session at the repository root so relative installs and test
    # paths resolve against the project, not the generated file.
    session.chdir(Path(__file__).resolve().parent.parent.parent)
    session.install(
        ".",
        "--group",
        "dev",
        "-r",
        "pyproject.toml",
    )
    seed = str(secrets.randbelow(2**31 - 2) + 1)
    env = {"PYTHONHASHSEED": seed, "PYTHONASYNCIODEBUG": "1"}
    cmd = ["pytest", "-q", "--timeout=300", f"--randomly-seed={seed}", *TEST_PATHS]
    # Free-threaded interpreters deserve an extra concurrent pass.  Do not make
    # ordinary interpreter sessions pay this cost.
    if str(session.python).endswith("t"):
        session.install("pytest-run-parallel")
        cmd[1:1] = ["--parallel-threads=auto", "--iterations=2"]
    session.run(*cmd, env=env)
