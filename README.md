# QDFM File Manager
A lightweight file manager written in Rust with the Slint GUI library.
It is still a work in progress. Currently only works on X11, wayland support planned.

# Dependencies:

XDG_DESKTOP_PORTAL and an implementation.
libxcb 1.12+

# Winit:

Currently using a forked winit since it doesn't support drag and drop.
event_processor.rs and platform::mod.rs are the only modified files

2025-04-16: It should now be implemented by winit, move to their version and remove our fork when adding wayland support.

# Testing: 

cargo test -- --test-threads=1

Since they all act on the same test directory, running them in multiple threads can cause issues.
Eventually might be worth changing this to different names to run them faster.


# Attributions
Made with: [Slint](https://github.com/slint-ui/slint)
