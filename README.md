# QDFM File Manager
Todo



# Dependencies:

XDG_DESKTOP_PORTAL and an implementation.
libxcb 1.12+

# Winit:

Currently using a forked winit since it doesn't support drag and drop.
event_processor.rs and platform::mod.rs are the only modified files

# Testing: 

cargo test -- --test-threads=1

Since they all act on the same test directory, running them in multiple threads can cause issues


# Attributions
Made with: [Slint](https://github.com/slint-ui/slint)

Icons: 

drive.svg - CC-BY-SA - snwh - https://github.com/snwh/suru-icon-theme/
file.svg & folder.svg - CC-BY - Dazzle UI - https://www.svgrepo.com/author/Dazzle%20UI/
