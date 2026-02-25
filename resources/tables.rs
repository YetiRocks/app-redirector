use yeti_core::prelude::*;

/// Rule: public read-only (redirect rule browser)
/// Writable redirect rules = open-redirect attack vector.
resource!(TableExtender for Rule {
    get => allow_read(),
});

/// Hosts: public read-only (host configuration browser)
resource!(TableExtender for Hosts {
    get => allow_read(),
});

/// Version: public read-only (version history)
resource!(TableExtender for Version {
    get => allow_read(),
});
