# Auto-generated BugHunt Hypothesis profile loader.
from __future__ import annotations

import os
from hypothesis import settings

settings.register_profile(
    "bughunt",
    max_examples=2000,
    deadline=None,
    derandomize=False,
    print_blob=True,
    report_multiple_bugs=True,
    stateful_step_count=100,
)
if os.getenv("HYPOTHESIS_PROFILE") == "bughunt":
    settings.load_profile("bughunt")
