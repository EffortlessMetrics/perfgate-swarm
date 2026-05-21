# PERFGATE-ADR-0001: Rails framework footprint lives under .rails/

Status: accepted
Date: 2026-05-21
Owner: docs-platform
Linked proposal: PERFGATE-PROP-0001
Linked specs: PERFGATE-SPEC-0001

## Decision

Long-term proposal/spec/ADR/lane/closeout/support/policy Rails artifacts live in `.rails/`.

## Context

The repo needs one portable, brandable framework root so downstream automation can operate consistently across repositories.

## Consequences

- Durable knowledge becomes tool-agnostic and repo-portable.
- External agent/session namespaces remain awareness-only.
- Validators and portal/email/F1 tooling can consume one canonical graph.

## Alternatives considered

- Repo-specific hidden directory naming (rejected: weak portability).
- Using `.codex/` or `.spec/` as durable home (rejected: wrong ownership).

## Follow-up specs / plans

Implement and enforce artifact graph contracts in validators and lane workflows.
