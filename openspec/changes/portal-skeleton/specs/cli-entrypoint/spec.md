# Spec Delta

## REMOVED Requirements

### Requirement: A verb without an implementation says so
**Reason**: **BREAKING**: `mu client` and `mu server` no longer print `not implemented`. Each verb of `mu` now has an implementation. The `server-process` capability describes `mu server`, and the `terminal-client` capability describes `mu client`.
**Migration**: None. No user relied on the text `not implemented`. The server-process scenarios "A named socket" and "No terminal" check that neither verb prints it. A new verb that exists before its implementation adds its own requirement in its change.
