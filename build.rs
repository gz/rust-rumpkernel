use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use git2::Repository;
use num_cpus;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir_path = PathBuf::from(out_dir.clone());

    println!("OUT_DIR {:?}", out_dir);

    let libs_built = out_dir_path
        .join("obj/dest.stage/usr/lib/librumpdev_bpf.so")
        .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpnet_config.so")
            .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpnet_netinet.so")
            .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpnet_net.so")
            .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpnet.so")
            .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpfs_tmpfs.so")
            .exists()
        && out_dir_path
            .join("obj/dest.stage/usr/lib/librumpvfs.so")
            .exists();

    if !libs_built {
        println!("RMDIR {:?}", out_dir);
        Command::new(format!("rm",))
            .args(&["-rf", out_dir.as_str()])
            .status()
            .unwrap();

        println!("MKDIR {:?}", out_dir);
        Command::new(format!("mkdir",))
            .args(&["-p", out_dir.as_str()])
            .status()
            .unwrap();

        println!("CLONE {:?}", out_dir);
        let url = "https://github.com/rumpkernel/buildrump.sh.git";
        match Repository::clone(url, out_dir.clone()) {
            Ok(_) => (),
            Err(e) => panic!("failed to clone: {}", e),
        };

        println!("BUILD {:?}", out_dir);
        env::set_var("TARGET", "x86_64-netbsd");
        env::set_var("MKSTATICLIB", "yes");

        // For options see also:
        // https://github.com/rumpkernel/wiki/wiki/Performance:-compile-options
        // https://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/sys/rump/README.compileopts
        Command::new("./buildrump.sh")
            .args(&[
                "-k",
                "-j",
                format!("{}", num_cpus::get()).as_str(),
                "-V",
                r#"RUMP_KERNEL_IS_LIBC=1"#,
                "-F",
                r#"CFLAGS=-Wimplicit-fallthrough=0"#,
            ])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();
    }

    println!(
        "cargo:rustc-link-search=native={}/obj/dest.stage/usr/lib/",
        out_dir
    );

    println!("cargo:rustc-link-lib=static=rump");
    println!("cargo:rustc-link-lib=static=rumpdev_bpf");
    println!("cargo:rustc-link-lib=static=rumpnet_config");
    println!("cargo:rustc-link-lib=static=rumpnet_netinet");
    println!("cargo:rustc-link-lib=static=rumpnet_net");
    println!("cargo:rustc-link-lib=static=rumpnet");
    println!("cargo:rustc-link-lib=static=rumpvfs");
    println!("cargo:rustc-link-lib=static=rumpfs_tmpfs");
    //println!("cargo:rustc-link-lib=static=rumpfs_kernfs");
    //println!("cargo:rustc-link-lib=static=rumpfs_null");
}
