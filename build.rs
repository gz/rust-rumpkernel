use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use git2::Repository;
use num_cpus;

fn artefacts_built(build_dir: &Path) -> bool {
    build_dir.join("rump/lib/librumpdev_bpf.so").exists()
        && build_dir.join("rump/lib/librumpnet_config.so").exists()
        && build_dir.join("rump/lib/librumpnet_netinet.so").exists()
        && build_dir.join("rump/lib/librumpnet_net.so").exists()
        && build_dir.join("rump/lib/librumpnet.so").exists()
        && build_dir.join("rump/lib/librumpfs_tmpfs.so").exists()
        && build_dir.join("rump/lib/librumpvfs.so").exists()
        && build_dir.join("rump/lib/librumpdev_pci.so").exists()
        && build_dir.join("rump/lib/librumpdev_pci_if_iwn.so").exists()
}

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir_path = PathBuf::from(out_dir.clone());

    println!("OUT_DIR {:?}", out_dir);
    let libs_built = artefacts_built(out_dir_path.as_path());

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

        let cpus = format!("{}", num_cpus::get());
        let mut options = vec![
            "-k",
            "-j",
            cpus.as_str(),
            "-F",
            r#"CFLAGS=-Wimplicit-fallthrough=0"#,
        ];

        let target_os = env::var("CARGO_CFG_TARGET_OS");
        match target_os.as_ref().map(|x| &**x) {
            Ok("none") => {
                options.push("-V");
                options.push(r#"RUMP_KERNEL_IS_LIBC=1"#)
            }
            _ => {}
        }

        // For options see also:
        // https://github.com/rumpkernel/wiki/wiki/Performance:-compile-options
        // https://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/sys/rump/README.compileopts
        Command::new("./buildrump.sh")
            .args(options.as_slice())
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();

        println!("BUILD PCI {:?}", out_dir);
        println!("OUT_DIR {:?}", out_dir);

        let rump_top = out_dir_path.join("src/sys/rump/");
        let mkconfig = out_dir_path.join("obj/tooldir/mk.conf");
        let toolflags = out_dir_path.join("obj/tooldir/toolchain-conf.mk");
        assert!(rump_top.exists());
        assert!(mkconfig.exists());
        assert!(toolflags.exists());

        env::set_var("TOPRUMP", rump_top.as_path());
        env::set_var("RUMPRUN_MKCONF", mkconfig.as_path());
        env::set_var("BUILDRUMP_TOOLFLAGS", toolflags.as_path());

        let rumpmake = out_dir_path.join("obj/tooldir/rumpmake");
        assert!(rumpmake.exists());

        Command::new(rumpmake.as_path())
            .args(&["obj", "dependall", "install"])
            .current_dir(&Path::new("./pci_build"))
            .status()
            .unwrap();
    }

    assert!(artefacts_built(out_dir_path.as_path()));
    println!("cargo:rustc-link-search=native={}/rump/lib", out_dir);

    //println!("cargo:rustc-link-lib=static=rump");
    //println!("cargo:rustc-link-lib=static=rumpdev_bpf");
    //println!("cargo:rustc-link-lib=static=rumpnet_config");
    //println!("cargo:rustc-link-lib=static=rumpnet_netinet");
    //println!("cargo:rustc-link-lib=static=rumpnet_net");
    //println!("cargo:rustc-link-lib=static=rumpnet");
    //println!("cargo:rustc-link-lib=static=rumpvfs");
    //println!("cargo:rustc-link-lib=static=rumpfs_tmpfs");
}
