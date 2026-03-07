# Domain Analysis Writing Guide

## Purpose

A domain model document defines the system's core boundaries, entities, and interactions. A reader should understand the system's domain structure within 5 minutes.

## Section Standards

### Bounded Contexts

Each bounded context must answer:
- What is its core responsibility?
- Where is the boundary with adjacent contexts?
- How does data cross boundaries (events, shared kernel, API)?

**Good:**
> **Order Context** manages the full order lifecycle from creation to fulfillment. It publishes `OrderPlaced` and `OrderShipped` events consumed by Payment and Shipping contexts. It never directly accesses payment or inventory data.

**Bad:**
> Order Context handles orders.

### Entity Relationships

Describe how core entities relate. Use "has-a", "belongs-to", "references". Clarify which relationships cross context boundaries.

### Aggregate Roots

Identify which entities serve as aggregate roots — entry points for consistency boundaries. Each aggregate should be independently transactable.

### Ubiquitous Language

Build a glossary of domain terms. Every term should have one unambiguous definition shared between dev and domain experts.

## Common Mistakes

- **One big context**: putting everything into a single bounded context defeats the purpose
- **Entity vs Value Object confusion**: if it has no identity and is immutable, it's a value object
- **Missing domain events**: if one context needs to react to something in another, there should be an event
- **Premature technical decisions**: domain models should be technology-agnostic
