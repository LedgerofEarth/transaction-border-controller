# Architecture Overview

## System Architecture

```
User Layer → API Layer → Bridge Layer → Blockchain Layer
```

## Component Responsibilities

### coreprover-service
- Event monitoring
- Timeout processing
- REST API

### coreprover-bridge
- Contract bindings
- Event streaming

### coreprover-contracts
- Escrow logic
- Receipt minting

### coreprover-zk
- Privacy proofs
- Ownership verification
