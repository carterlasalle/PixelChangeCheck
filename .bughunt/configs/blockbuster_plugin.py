from __future__ import annotations

import pytest
from blockbuster import blockbuster_ctx

# trace:exempt reason=generated-by-bughunt-configure-do-not-hand-edit
@pytest.fixture(autouse=True)
def _bughunt_blockbuster():
    with blockbuster_ctx():
        yield
