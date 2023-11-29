# didethresolver

## Quick Start

This example code is intended to introduce the [did:eth](https://github.com/veramolabs/did-eth) contract to an engineer.

A simplified registration workflow is as follows

1. store attributes on owner public key 
```
$ (cd profile && cargo run)
```
2. query profile and produce did doc
```
$ (cd profile_doc && cargo run)
```
3. revoke attributes from profile
```
$ (cd revoke && cargo run)
```

## Introduction to DID Specification

The Decentralized Identifiers (DIDs) v1.0 specification, as outlined by W3C, describes a system for creating verifiable, decentralized digital identities. DIDs are unique identifiers that can refer to any subject, such as a person or organization, and are controlled by their creator rather than a centralized authority. This approach aims to decouple identity management from centralized registries and certificate authorities.

Key aspects of DIDs include:

1. **Decentralization**: DIDs eliminate reliance on centralized authorities for identifier management. They offer a way to manage digital identities without a single point of failure【9†source】.

2. **Control and Privacy**: Entities can directly control their digital identifiers and manage the privacy of their information, including what personal data is revealed in different contexts【9†source】.

3. **Interoperability and Portability**: DIDs are designed to be interoperable and can be used across different systems and networks. This promotes a seamless experience across various digital platforms【9†source】.

4. **Architecture**: The architecture of DIDs involves several components, including DID subjects (the entities identified by the DIDs), DID controllers (those who have the capability to make changes to a DID document), and verifiable data registries (systems that record DIDs and DID documents for resolution)【10†source】.

5. **Conformance**: The specification outlines conformance criteria for different components of the DID ecosystem, including DIDs, DID documents, DID resolvers, and DID URL dereferencers. Conformance ensures that these components operate consistently within the DID infrastructure【11†source】.

