# ADR-055: Mamba-2 physical motion foundation profile

| Field | Value |
|---|---|
| ID | ADR-055 |
| Status | Superseded |
| Version | 1.1 |
| Decision date | 2026-08-09 |
| Last verified | 2026-08-10 |
| Normative dependencies | [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |
| Supersedes | none |
| Superseded by | fully [ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md) |

## Historical summary

ADR-055 proposed Mamba-2 as the intended learned foundation for a bounded
morphology family, with explicit recurrent cache, fixed-PD position/velocity
actions, portable standard-op export and procedural fallback.

[ADR-057](057-hierarchical-learnable-motor-system-and-policy-family-architecture.md)
rejects Mamba as the target low-level foundation. MLP is the first humanoid
low-level comparator; TCN/GRU are the first adaptation comparators. Mamba may
return only through an equal-budget Proposed experiment for long-history
adaptation, motion generation or temporal planning and gains no default status
from export availability.

This file is a historical pointer and is not current authority.
