# Specification notes
This document outlines issues found with the flif16 specification during
implementation. These issues may exist either in the
[spec doc](https://flif.info/spec.html) or in the 
[reference implementation](https://github.com/FLIF-hub/FLIF).

- reference implementation ignores loop counter
- spec and reference are unclear of usage of bits per pixel in second header
    - The implementation uses the max of those values while the spec does not
    mention that constraint
    - furthermore, the implementation for some reason uses 255 and 65535
    instead of 8 and 16 and just takes the log2 of them later (is this some
    weird optimization?)
- range encoder is not documented
    - will there ever be a need for other configs than the 24bit one currently 
    implemented?
- custom bitchances are not implemented and therefore not documented very well
