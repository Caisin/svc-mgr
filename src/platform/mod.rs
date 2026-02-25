cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        pub mod launchd;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        pub mod systemd;
        pub mod openrc;
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))] {
        pub mod rcd;
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        pub mod sc;
        pub mod winsw;
    }
}
