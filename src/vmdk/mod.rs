mod descriptor;

#[derive(Copy, Clone, FromPrimitive, ToPrimitive, Eq, PartialEq)]
pub enum VmdkKind {
    TwoGbSparse,      // VMware Workstation multi-extent dynamic disk
    TwoGbFlat,        // VMware Workstation multi-extent pre-allocated disk
    MonolithicSparse, // VMware Workstation single-file dynamic disk
    MonolithicFlat,   // VMware Workstation single-file pre-allocated disk.
    VmfsSparse,       // ESX Dynamic Disk
    Vmfs,             // ESX pre-allocated disk
}
