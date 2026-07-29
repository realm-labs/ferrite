# Playable topology conformance

`G01-P4-B4` binds the C2 gameplay spine to the Region remoting boundary. It runs one deterministic
scenario through direct-local and Lattice-backed routing, then requires the canonical committed
state and the encoded Java 26.2 clientbound trace to be identical.

## Shared scenario

The scenario uses two adjacent V1 Regions and the production `PlayerRegionLogic`,
`LocalRegionRunner`, player session, chunk session, block transaction logic, replay projection, and
Java 26.2 clientbound codec:

1. admit a semantic join in Region `(0, 0)` and stream the initial chunk controls and terrain;
2. commit `player_loaded`, same-Region movement, and a movement across the x Region boundary;
3. commit the generation-fenced player transfer into Region `(1, 0)` and recenter chunk interest;
4. place a block, reject a second placement with two authoritative corrections, and emit the
   cumulative prediction acknowledgement;
5. commit matching start/stop destroy operations and replicate the resulting air state.

The final state projection scans every loaded voxel, omits only the known air default, includes the
encoded authoritative player component, and uses the canonical replay world hash. Activation
generation and transport metadata remain fencing data and do not enter the gameplay hash.

The locked seven-tick state digest is:

```text
1e7c50dbf4463c858fcd779f4db59a08418e54cab7ae0e502821bba95ad0a858
```

## Routing modes

| Mode | Semantic admission path |
|---|---|
| Local | `RegionCommand` and `EntityTransfer` are admitted directly to the Region runner. |
| Lattice in-process | Both types are encoded into stable bounded semantic payloads, wrapped in a Ferrite remote Region envelope and a pinned Lattice `EntityTell` frame, decoded, generation-checked, and only then admitted. |
| Process-isolated | Three child executions independently run the same Lattice path and return structured evidence over the coordinator pipe. |

The process-isolated repetitions detect process/environment drift in this scenario; they are not
presented as a replacement for the three-partition process topology proof in
[Region topology and fault conformance](topology-conformance.md). That earlier proof owns
multi-node partition routing, barriers, generation fencing, recovery, and message-fault outcomes.
This batch adds real C2 gameplay semantics to the same remoting envelope contract.

`ferrite-region-runtime::lattice::semantic` owns the stable command and transfer payload formats.
They preserve every semantic field, validate bounded resource and state payloads, reject unknown
source/role tags, wrong message kinds, truncation, and trailing bytes, and keep Lattice frame types
inside the runtime adapter.

## Packet compatibility

Every projected clientbound packet is encoded by the Java 26.2 Play codec. The trace records its
wire ID, exact body length, and BLAKE3 body digest in order. A domain-separated digest also covers
the complete ordered packet bodies, so equal packet names with different fields do not pass.

The locked 16-packet trace digest is:

```text
8328cdaa1bf165640fc44b8db0be6727c5445c302983dff9bcbfa36e16fcf95e
```

The golden additionally requires a full chunk-with-light packet, three prediction
acknowledgements, and the authoritative correction/committed block-update packets.

Run the proof with:

```text
cargo run -p ferrite-cluster -- verify-playable
```

The repository test suite also runs the local/Lattice golden and invokes the cluster command across
the child-process boundary.

## Scope

This proves topology-independent C2 semantic admission, committed state, and packet projection for
the locked scenario. It does not claim that the child processes partition the gameplay Regions;
the existing Phase 2 topology suite owns that substrate proof. Adverse latency, batching,
malformed/backpressure cases, cross-Region client acceptance, and the unmodified-client C2 smoke
remain `G01-P4-B5`.
