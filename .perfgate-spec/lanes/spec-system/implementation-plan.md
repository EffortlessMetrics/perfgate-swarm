# Repo-native spec system implementation plan

Status: active
Owner: repo-architecture
Linked proposal: PERFGATE-PROP-0001
Linked specs: PERFGATE-SPEC-0001
Linked ADRs: PERFGATE-ADR-0001

## End state

Durable proposal/spec/ADR/lane/closeout rails are repo-owned under `.perfgate-spec/` and linked through the index.

## Work items

### Work item: namespace-doctrine

Status: done
Linked proposal: PERFGATE-PROP-0001
Linked spec: PERFGATE-SPEC-0001
Linked ADR: PERFGATE-ADR-0001
Blocks: none
Blocked by: none
Issue: n/a
PR: local

#### Goal

Define ownership boundaries and index conventions.

#### Production delta

Adds namespace README, index, docs guidance.

#### Non-goals

Editing `.spec/` or `.codex/` contents.

#### Acceptance

Contributors can identify durable vs external namespaces.

#### Proof commands

```bash
git diff --check
```

#### Rollback

Revert commit that introduced namespace.

#### Claim boundary

Does not validate artifacts automatically.
