#!/usr/bin/env python3
"""PreToolUse guard: refuse the built-in generic agents.

cgap CLAUDE.md law: only agents defined in .claude/agents/ plus read-only
Explore/Plan. Enforced as a deny-by-default allowlist (fail-closed): any
subagent_type not on the ALLOWED list exits 2 (block) with a typed refusal
naming the allowed agent types.
"""
import json
import sys

# Project agents (.claude/agents/) + read-only built-ins per CLAUDE.md.
ALLOWED = frozenset({
    "lane-runner", "court-auditor", "gate-runner", "reviewer",
    "Explore", "Plan",
})

# Known generic/built-in types, for a precise refusal reason.
GENERICS = frozenset({
    "general-purpose", "claude", "code-reviewer", "security-review",
    "output-style-setup", "statusline-setup", "architect", "worker",
})

# Tool name carrying subagent dispatch (new + classic names).
AGENT_TOOLS = frozenset({"Agent", "Task"})


def refusal(subagent: str) -> str:
    kind = "generic built-in agent" if subagent in GENERICS else "unlisted agent type"
    return (
        f"GENERIC_AGENT_REFUSED: subagent_type '{subagent}' is a {kind} "
        f"and is banned in cgap. Allowed: {', '.join(sorted(ALLOWED))} "
        "(.claude/agents/ + read-only Explore/Plan per CLAUDE.md). "
        "Re-dispatch with a project agent type."
    )


def main() -> int:
    try:
        payload = json.load(sys.stdin)
    except Exception:
        print(json.dumps({"decision": "block",
                          "reason": "HOOK_INPUT_INVALID_JSON: "
                                    "ban_generic_agents could not parse stdin"}))
        return 2  # fail closed: an unparseable dispatch is not provably lawful

    tool = payload.get("tool_name", "")
    if tool not in AGENT_TOOLS:
        return 0

    subagent = payload.get("tool_input", {}).get("subagent_type", "")
    if subagent in ALLOWED:
        return 0

    print(json.dumps({"decision": "block", "reason": refusal(subagent)}))
    return 2


if __name__ == "__main__":
    sys.exit(main())
