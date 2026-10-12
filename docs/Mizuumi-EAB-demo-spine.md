# Mizuumi EAB demo spine, for your review

Linh, this is the demo plan for Friday. Please read the whole thing and answer the questions at the end. Nothing here is final until you confirm what the build can sustain.

## Timing

The session is 120 minutes. We plan content for \~ 90 minutes and hold \~30 minutes in reserve. Ten minutes at the start absorb late arrivals, Tommy’s opening, and introductions. 20 minutes float across the session for questions and for the closing open Q\&A. Each act’s allocation already includes one or two questions from the room.

1. Act 0 Why, 7 min.   
2. Act 1 Integrated and fast, 14\.   
3. Act 2 Sovereign across the group, 14\.  
4. Act 3 Safe by structure, 11\.  
5. Act 4 Efficient at scale, 9\.  
6. Act 5 Agentic, safely, 11\.  
7. Act 7 The AI platform, together, 6, slides only.  
8. Act 6 Adoption and the operating model, 7\.  
9. Act 8 The ask, 8\.

Open discussion follows the ask. Content is 87 minutes, reserve is 33\.

Every act then has the same rhythm:

- one **claim** slide,  
- one **mechanism** slide,  
- a **demo beat** on that customer,  
- one **Q\&A** question seeded in the notes, and   
- a **maturity label**.

The person in the room follows a story across two hours and the architecture arrives as what the story needed. Acts 3 and 4 have no natural customer beat, so those two take an adversary scenario and a tier-versus-SLA scenario on the same data instead.

Five demos, none longer than five minutes, at roughly 20, 35, 50, 60, and 72 minutes into the session. If we fall behind, the cut order is fixed in advance: first the Act 4 demo becomes a thirty-second recording, then the witnessed-log half of Act 3 becomes a recording, then Act 6 compresses to one slide. Acts 1, 2, 5, and 8 are never cut. Every demo has a recording as fallback already; the recordings also serve as the time fallback.

## The customer

One synthetic person, consistent across every generator. Working name Trần Minh Anh, 34, HCMC. National ID, phone, and email must agree across the Vikki and HDBank generators. The CRM record carries one deliberate drift: an older phone number and the name with diacritics stripped. This drift is what the matcher must resolve in Act 1\.

Minh Anh’s history, which the generators must produce:

* Vikki: digital account opened, the default digital wallet adopted, a small set of wallet transactions over the last ninety days.  
* HDBank: a consumer loan originated, three repayments made.  
* CRM: one contact record with the drift above, one campaign interaction.  
* VietJet: three bookings over the last year, one with a Vikki wallet payment.

The transactions and product adoptions exist so that Act 2 has signals to converge and Act 4 has a population to route. Volume stays small for the customer and larger for the synthetic population around her.

## Demo beats

### Act 1, Integrated and fast. 

The two T24 generators stream through Redpanda, the CRM poll lands. Three Minh Anh records appear in the UI. The matcher scores the CRM pair as uncertain because of the phone drift. The steward resolves it. The golden record materializes with lineage to all three sources, the wallet adoption and the loan visible on it, and the steward’s decision recorded. Closing line on screen: one customer, three systems, one governed record, under contract.

### Act 2, Sovereign across the group.

The mesh view shows Vikki, HDBank, and CRM as source domains inside the Vietnam mesh, each with an outport under an ODCS contract. VietJet joins as a fourth source domain. Minh Anh’s booking signals converge into the Group Customer derived domain, with its own owner and steward, and the room sees the derived record form from the Act 1 golden record plus the VietJet signals.

Then the transfer. We use the terms as the document defines them: one control plane is one mesh is one jurisdiction; movement inside a mesh is sharing; movement between meshes is transfer, and the enclave is the transfer mechanism, with the data contract as the carrier. Where regulation requires it, a line of business such as banking gets its own mesh and control plane regardless of jurisdiction. On a laptop we do not run two full meshes. Two options, you choose:

Option A: a second Rust control plane runs as a minimal process with no data planes beneath it. The Vietnam mesh transfers one field set under a contract with residency terms; the second control plane records receipt; a second contract with a field lacking residency clearance is refused before anything moves. Both the transfer and the refusal appear in the witnessed log.

Option B: the receiving side is a stub endpoint. We show the refusal path fully and the accepted path up to “transfer recorded, awaiting receiving mesh.” Less complete, far lighter.

Closing line on screen: sharing inside the mesh, transfer across it, the contract decides.

### Act 3, Safe by structure.

No customer beat; one adversary scenario, the mistaken cross-domain grant. A steward grants a consumer access to the HDBank outport. Unity Catalog vends a short-lived RustFS credential. The consumer queries. The credential expires. The same query fails. Then the witnessed log: the grant and the vend appear as entries, a checkpoint is signed, a second witness co-signs, a tampered tile fails verification. Closing line on screen: tamper-evident, independently witnessed.

### Act 4, Efficient at scale.

The same query shape twice. Over Minh Anh’s domain it runs on DuckDB in milliseconds. Over the full synthetic population it routes to Daft. SigNoz shows the route decision, the cost, and the completion notification. Closing line on screen: cheapest engine that meets the SLA, escalation only when required.

### Act 5, Agentic, safely.

Two sessions, same question, “summarize Minh Anh’s relationship with the group.” Session A is opened by the control plane for a principal with the HDBank grant; Pi runs in the OpenShell sandbox; the supervisor injects the vended credential; the DuckDB MCP tool answers. Inside the sandbox we show that the harness itself holds nothing: no environment variable, no file, no token. Session B is opened for a principal without the grant; the supervisor has no credential for that endpoint; the identical MCP call is refused at the gateway. Then, in Session A, the agent attempts an external endpoint and the supervisor denies the connection. Closing line on screen: the agent holds no credential and dies with its sandbox. Consent is kept off this act entirely; it is a Planned item and appears on the Act 8 residue slide only.

### *Act 7, The AI platform, together*.

Six minutes, three slides, no demo and no open discussion at this point. The slides show where the session API sits in the architecture, what joint ownership means in practice, and the named seat on the task force. The last slide carries three questions for the AI platform owner, to be taken in the open discussion after the ask. The session API is labeled Planned.

Acts 0, 6, and 8 have no demo. 

Act 8 shows the §15 tracker once, as the honest list of what remains.

## Questions for you, Linh

1. Act 1: does the matcher exist at a level that can surface an uncertain pair and let a steward resolve it in the UI, or is the resolution manual outside the UI today? Either is fine; the slide label must say which.  
2. Act 1: can the generators produce the wallet adoption, the loan with repayments, and the CRM drift as specified, and keep the identifiers consistent across the three?  
3. Act 2: do we have a VietJet booking generator, or does one need to be written this week?  
4. Act 2: Option A or Option B for the transfer, given the laptop?  
5. Act 3: can a second witness identity be simulated, and can we show a tampered tile failing verification live, or only as a recording?  
6. Act 4: is the tier (Daft vs. Spark vs. DuckDb) routing decision visible in SigNoz today, or is routing manual? If manual, the slide says Partial and the demo shows the two engines without the automatic decision.  
7. Act 5: can Session B’s refusal be shown at the gateway with a clear message, and can we show the inside of the sandbox with no credential present?  
8. Which of the five demos are you least confident running live? Those get the recording as the primary path and the live run as the bonus.

