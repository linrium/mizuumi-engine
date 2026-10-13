# Mizuumi EAB demo spine v2, for your review

Linh, this replaces the earlier spine. Read it fully; the questions are at the end.

The demos are the proof of two messages

- One: the backbone is real, it runs today, and it is enough to start the pilot.  
- Two: every critical part is built so that the next capability mounts on it, so the roadmap is a sequence of attachments to a stable core, flexible to what the group needs.

Each demo therefore has two halves: what *runs* *now*, and what *mounts next*, with the pilot as the time it gets built.

## Audience and tone

All CIOs and CTOs of the group, Tommy, and the owner of the internal AI platform. Cost and data-center power are a concern at the top of the group. They are modernizing fast: two new T24 R26 cores, a CRM, a coming loyalty solution, an AI platform nine months old. Mizuumi arrives with the new cores and treats the AI platform as its first consumer. Slide text is flat English for translation. You narrate the demos in Vietnamese; Rikki frames each act before the demo and lands the closing line after it. On screen we use roles (catalog, object store, witnessed log, sandbox, control plane); product names appear once on the architecture slide and in the speaker notes. Synthetic data is stated once, in Act 0: no real personal data is touched in this session, which is itself a point under the new data protection regime.

## The customer

Trần Minh Anh, 34, HCMC, synthetic. VietJet passenger, HDBank customer, Vikki user. Vikki: digital account, the default wallet adopted, wallet transactions over ninety days. HDBank: a consumer loan with three repayments. CRM: one contact with an older phone number and the name without diacritics, plus one campaign interaction. VietJet: three bookings in the last year, one paid with the Vikki wallet. National ID, phone, and email agree across Vikki and HDBank; the CRM carries the drift. A synthetic population of a few hundred thousand surrounds her so that pipelines and telemetry have volume.

## The spine

**Five demos**, **one chain**: *land, share, secure, measure, act*.

Each demo starts from the artifact the previous one left behind, on the same customer and the same data. The room sees one system doing one thing after another, each step standing on the last.

## Timing

120 minutes. Content 89, reserve 31: ten at the open for arrivals, Tommy’s opening, and introductions; twenty-one floating for questions and the discussion after the ask. Each act’s time includes one or two questions.

Act 0 Why, 7\. Act 1 Land, 14\. Act 2 Share, 14\. Act 3 Secure, 12\. Act 4 Measure, 9\. Act 5 Act, 12\. Act 7 The AI platform together, 6\. Act 6 Adoption and operating model, 7\. Act 8 The ask, 8\. Open discussion follows.

Demos at roughly 20, 38, 55, 68, and 85 minutes. None longer than five minutes. Cut order if behind: first the Splink half of Demo 1 moves to a recording, then Demo 4 moves to its recording, then Act 6 compresses to one slide. Demos 2, 3, and 5 are never cut.

## Demo rules

One screen per half. A demo ends on a slide split in two columns: what you just saw, what mounts here in the pilot. Every demo has a recording. If a live run fails, you switch to the recording without comment and Rikki keeps narrating; the room never hears the word “failed”. Full timed run-through on Thursday, twice.

### Act 0, Why (7 min, slides only)

**Purpose**: give the room the frame and the sentence that governs the session.

**The estate today**: independent warehouses and partially built lakes, with the two new cores, the CRM, and loyalty landing fresh.

**What the group needs**: one customer across subsidiaries, sovereignty by jurisdiction and by regulation, agentic use without exposure, cost and power visible before hardware is bought. Mizuumi in one picture.

**How to read the deck**: the maturity labels, and the governing sentence: built intentionally to the core, stopped here for the group’s direction. This act plants the message that the mesh returns each subsidiary’s focus to its own business; Act 6 closes it. Synthetic data stated here, once.

### Act 1, Land (14 min, Demo 1\)

**Purpose**: show that all the data is in one place, the mesh, in source-aligned domains with lineage, today; then show how master data assembles on that.

**Business moment**: Minh Anh books a VietJet flight and pays with her Vikki wallet. Three subsidiaries hold a piece of her. Today nobody holds her whole.

**Now, shown**: the two T24 generators stream through the message bus and the streaming engine into the Vikki and HDBank source domains as managed tables in the object store under the catalog; the CRM poll lands in its own domain. The UI shows the mesh with three domains populated and Minh Anh’s three records with lineage back to each source. The drift in the CRM record is visible.

**Next, mounted**: a pipeline runs the probabilistic matcher over the three domains against the synthetic population. One screen: Minh Anh’s pairs scored, the Vikki and HDBank pair above the upper threshold, the CRM pair inside the band because of the phone drift, the two threshold lines drawn. You say: this is stage two of four. The pilot adds stage one (deterministic keys), stage three (a model judgment on the uncertain band only), the steward’s queue, and survivorship, on this same pipeline.

**Slides**: claim; Minh Anh in three systems; the four stages, three signals, and the human in the loop; the golden record as a gold product under contract with lineage, the week-six gate; loyalty and insurance as the next in-ports.

**Closing line**: the data is in one place, governed per source, and the golden record is assembled from here.

**Labels**: ingestion and lineage Partial; matcher Partial as a pipeline, master data Planned; golden record Planned, week-six gate.

**Artifact left behind**: three governed domains and a scored pair table.

**Your role**: run the landing, show lineage, trigger the matcher pipeline, read the scores. Keep the matcher screen to one table and the two lines; executives need the lines, not the weights.

### Act 2, Share (14 min, Demo 2\)

**Purpose**: show that sharing inside the mesh is live, contract-governed through the catalog under the control plane; then show that derived domains and cross-mesh transfer are built on this same mechanism.

**Business moment**: the VietJet domain wants to know which passengers are also HDBank customers, to design an offer. A legitimate question that today needs an email, a spreadsheet, and a risk.

**Now, shown**: the VietJet domain requests the HDBank outport. The contract opens in the UI: ODCS, with its residency terms and its quality rules. The control plane governs the grant through the catalog. The consumer queries Vikki and HDBank together and sees Minh Anh across two subsidiaries in one result, with lineage. The request, the contract, and the grant appear as events.

**Next, mounted**: that cross-domain result is the Group Customer derived domain in embryo. Derived domains, with their own owner and steward, are built on this share mechanism: aggregation and correlation producing higher-order products. Transfer across meshes is the same contract carried across control planes through an enclave; Phase B delivers one simulated transfer. Regulation: the data protection law in force since January 2026 and its implementing decree require documented impact assessments for processing and for cross-border transfer; a contract with residency terms and a witnessed grant are that documentation produced by the platform. We say “compliance-ready by structure”; counsel confirms the mapping before Friday.

**Slides**: claim; one control plane is one mesh is one jurisdiction, with regulation able to require a separate mesh for a line of business; domains, ownership, products, in-ports and outports to contracts, Vikki as its own source domain, Group Customer as the derived domain; the contract; stewards as gates; the enclave figure; migration (parallel run, consumer-by-consumer cutover under contract, history with lineage, retirement gated on blast radius).

**Closing line**: sharing inside the mesh, transfer across it, the contract decides.

**Labels**: sharing and contracts Partial (quality rules operator-triggered today); derived domain Planned, Phase B; enclave Planned, week-twelve gate.

**Artifact left behind**: a live share under contract, with a vended credential beneath it.

**Your role**: drive the request and grant from the UI, open the contract, run the query. The VietJet CTO should recognise his domain as the requester.

### Act 3, Secure (12 min, Demo 3\)

**Purpose**: lift the veil on the share just made and show that safety is structural: no standing credential, every event witnessed.

**Business moment**: the HDBank CIO asks what the VietJet consumer actually holds now that it has been granted access. The answer is nothing standing.

**Now, shown**: beneath the Act 2 share, the catalog vended a short-lived object-store credential, the secrets authority minted the material, and the engine holds nothing standing. The credential expires and the same query fails, by design. The grant, the vend, the queries, and the expiry appear in the witnessed log with a signed checkpoint, and the log verifies.

**Next, mounted**: independent, legally accountable witnesses co-signing checkpoints, and the HSM-backed signing key with an external HSM as the regulator-demanded alternative, both pilot items. Per-session delegated credentials for agents, which Act 5 will use.

**Slides**: claim; isolation stack (workspace for collaboration, domain and mesh for isolation); the secrets authority, BYOK, HSM; the witnessed log, tamper-evident with independent witnesses; six adversaries with one control each.

**Closing line**: nothing is held, everything is witnessed.

**Labels**: credential vending Partial; secrets authority Partial; witnessed log Partial (witness assumption open).

**Artifact left behind**: a verified log of everything done so far.

**Your role**: show the credential’s lifetime, the expiry failure, and the log verification. The failure is the point; say “by design” in the first sentence.

### Act 4, Measure (9 min, Demo 4\)

**Purpose**: show that every engine’s work is visible in telemetry, which is the basis of cost control, tier routing, and the hardware decision the group will make with its own numbers.

**Business moment**: the group asks what this session’s work cost in compute. We show it.

**Now, shown**: telemetry for the work already done: the matcher pipeline on the distributed engine, the share query on the interactive engine, each with engine, duration, and resource footprint, and the evidence projection of the same events. The same query shape on two engines with their footprints side by side.

**Next, mounted**: automatic tier routing (the cheapest engine that meets the SLA) and chargeback, both built on exactly these numbers. Power and sizing: no hardware numbers until the group’s own dev-kit stress tests and per-subsidiary proofs of concept; this telemetry is how those numbers will be produced.

**Slides**: claim; tiers against SLA; telemetry and the evidence projection; servicer team and chargeback roadmap; power and sizing discipline.

**Closing line**: every engine’s cost is visible, so the cheapest engine that meets the SLA is a policy.

**Labels**: telemetry Partial; tier routing Partial (manual today); chargeback Stub.

**Artifact left behind**: the cost view of the session so far.

**Your role**: one dashboard, two engines, one comparison. Resist adding panels.

### Act 5, Act (12 min, Demo 5\)

**Purpose**: show that an agent can work on governed data while holding no credential, and that the same governance decides what it may do.

**Business moment**: a relationship manager asks an assistant, in plain language, to summarize Minh Anh’s relationship with the group before a call.

**Now, shown**: Session A, a principal with the Act 2 grant. The control plane opens the session; the harness runs inside the sandbox; the supervisor outside the sandbox injects the vended credential; the interactive-engine tool answers the question over the shared data. The sandbox interior is shown: no credential anywhere. Session B, a principal without the grant: the identical call is refused. Session A attempts an external endpoint and the supervisor denies the connection.

**Next**, **mounted**: the model route per data class with self-hosted models as the only route for governed data; the tool catalog beyond this one tool with action-bound approval; the session API, which the AI platform will consume.

**Slides**: claim; the session as a control-plane-initiated governed object; harness in the sandbox with the supervisor holding every credential; model route; tool catalog and action-bound approval.

**Closing line**: the agent holds no credential and dies with its sandbox.

**Labels**: sandbox and supervisor Partial; tool catalog Partial (one tool); model route and session API Planned.

**Your role**: two sessions, one question, one refusal, one denied connection. The sandbox-interior moment is the only terminal in the session; keep it under twenty seconds.

### Act 7, The AI platform, together (6 min, slides only)

**Purpose**: position the group’s AI platform as the first consumer of the session API, collaborative, with a named seat.

**Three slides**: 

1. where the session API sits;  
2. joint ownership in practice;  
3. the named seat on the task force and three questions for the AI platform owner, taken in the discussion after the ask.

No demo, no integration claim.

### Act 6, Adoption and the operating model (7 min, slides only)

**Purpose**: show that adoption is a structure the subsidiaries own.

- Maturity per domain;  
- the steward model (one accountable executive per subsidiary, executing stewards per line of business, business-side);  
- team composition;  
- what the subsidiaries gain, closing the Act 0 message.

### Act 8, The ask (8 min, slides only)

**Purpose**: the ask, framed as the group taking the wheel. Built intentionally to the core, stopped here for the group’s direction. The remaining-work tracker shown once. The twelve-week pilot, Phase A and Phase B, gated at week six.

**Four items**: 

- Mizuumi as the group’s candidate standard pending the pilot;  
- the AI platform integration through the session API under joint ownership;  
- a budget envelope for the pilot and the servicer team seed;  
- the task force.

**Closing**; thanks to Tommy spoken. Open discussion follows.

## Questions for you

- Does the full landing path run end to end today without intervention, for both T24 generators and the CRM poll, and does the UI show lineage per record?  
- Can the generators produce Minh Anh’s history and the CRM drift as specified, with consistent identifiers, and a synthetic population large enough for telemetry to be interesting?  
- Can a pipeline run the probabilistic matcher over the three domains and write a scored pair table with the two thresholds by Thursday, shown in one table in the UI?  
- Can the share request, contract view, and grant be driven from the UI, with the VietJet domain as requester, and the cross-domain query shown with lineage?  
- Can the vended credential’s lifetime be set short enough to expire live, and does log verification with a signed checkpoint display cleanly?  
- Does the telemetry capture the matcher pipeline and the share query with resource figures, and can the same query shape be shown on two engines?  
- For Demo 5: can the refusal for an ungranted principal be shown with a clear message, and can the sandbox interior be shown without a long terminal session?  
- Which of the five would you run from the recording as the primary path?

Anything above that you would phrase differently in Vietnamese for this room?
