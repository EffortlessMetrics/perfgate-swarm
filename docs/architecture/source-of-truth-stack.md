# Source-of-truth stack

Perfgate uses a repo-owned durable control plane under `.perfgate-spec/`.

This preserves full artifact richness (proposal, spec, ADR, lane, proof, closeout) while separating durable repository truth from external tool/session state.
