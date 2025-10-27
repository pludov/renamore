fn main() {
    if supported() {
        println!("cargo:rustc-cfg=linker");
    }
}

#[cfg(all(feature = "always-supported", not(target_os = "android")))]
fn supported() -> bool {
    true
}

#[cfg(all(feature = "always-fallback", not(target_os = "android")))]
fn supported() -> bool {
    false
}

#[cfg(not(any(
    feature = "always-supported",
    feature = "always-fallback",
    target_os = "android",
)))]
fn supported() -> bool {
    use std::process::Command;
    let target = std::env::var("TARGET").unwrap();

    if target.contains("android") {
        // For android, we assume syscall is available, but not wrapped.
        // Running a test compile will not work most of the time (cross compile).
        cc::Build::new()
            .file("src/linux-musl.c")
            .compile("linux-musl");
        return true;
    }

    let dir = tempfile::tempdir().unwrap();
    let test_c = dir.path().join("test.c");

    let compiler = cc::Build::new().cargo_metadata(false).get_compiler();
    let compiler_path = compiler.path();

    // It might be better to #include the relevant headers and check that the
    // argument types are as expected.

    if target.contains("linux") {
        std::fs::write(
            &test_c,
            b"
            void renameat2();
            void statfs();

            int main() {
                renameat2();
                statfs();
            }",
        )
        .unwrap();

        let status = Command::new(compiler_path)
            .current_dir(dir.path())
            .arg("test.c")
            .status()
            .unwrap();

        if status.success() {
            return true;
        }

        // musl doesn't expose a wrapper around the renameat2 syscall but it
        // does have the syscall number definition. So we're providing our own
        // wrapper. Although, the syscall might not exist and we'd get an error
        // instead of using the fallback in that case.
        if target.contains("musl") {
            cc::Build::new()
                .file("src/linux-musl.c")
                .compile("linux-musl");
            return true;
        }
    }

    false
}
