use std::env;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

//use git2::Repository;
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

fn usermtree() -> io::Result<ExitStatus> {
    let include_dirs = &[
        "adosfs",
        "altq",
        "arpa",
        "c++",
        "c++/experimental",
        "c++/ext",
        "crypto",
        "dev",
        "filecorefs",
        "fs",
        "i386",
        "isofs",
        "miscfs",
        "msdosfs",
        "net",
        "net80211",
        "netatalk",
        "netbt",
        "netinet",
        "netinet6",
        "netipsec",
        "netisdn",
        "netkey",
        "netmpls",
        "netnatm",
        "netsmb",
        "nfs",
        "ntfs",
        "openssl",
        "pcap",
        "ppath",
        "prop",
        "protocols",
        "rpc",
        "rpcsvc",
        "rumprun",
        "ssp",
        "sys",
        "ufs",
        "uvm ",
        "x86",
    ];

    for dir in include_dirs {
        let cmd = Command::new("mkdir");
        let out_dir = "{}/include/{}";
        cmd.args(&["-p"])
    }
}

/// Builds a 'user-space' library of the rumpkernel
/// i.e. things like pthread, libc etc.
fn makeuserlib(make_path: &Path, lib: &Path) -> io::Result<ExitStatus> {
    let mut make = Command::new(make_path);
    let cpus = format!("{}", num_cpus::get()).as_str();

    eprintln!("makeuserlib: {:?} obj", lib);
    make.args(&["obj"]).current_dir(lib).status()?;

    eprintln!("makeuserlib: {:?} dependall", lib);
    make.args(&[
        "MKMAN=no",
        "MKLINT=no",
        "MKPROFILE=no",
        "MKYP=no",
        "MKNLS=no",
        "NOGCCERROR=1",
        "HAVE_LIBGCC_EH=yes",
        "-j4",
        "dependall",
    ])
    .current_dir(lib)
    .status()?;

    eprintln!("makeuserlib: {:?} install", lib);
    make.args(&[
        "MKMAN=no",
        "MKLINT=no",
        "MKPROFILE=no",
        "MKYP=no",
        "-j4",
        "install",
    ])
    .current_dir(lib)
    .status()
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
        Command::new("git")
            .args(&["clone", "--depth=1", url, out_dir.as_str()])
            .status()
            .unwrap();

        println!("BUILD {:?}", out_dir);
        env::set_var("TARGET", "x86_64-netbsd");
        env::set_var("MKSTATICLIB", "yes");

        let cpus = format!("{}", num_cpus::get());
        // -w disables all GCC warnings; a drastic method to ensure that the rump code-base
        // compiles with newer compilers
        //
        // -k: means we don't build user-space libraries like libc
        let buildrump_arguments = &[
            "-j",
            cpus.as_str(),
            "-F",
            r#"CFLAGS=-w"#,
            "-V",
            r#"RUMP_KERNEL_IS_LIBC=1"#,
            "checkout",
            "fullbuild",
        ];

        // For options see also:
        // https://github.com/rumpkernel/wiki/wiki/Performance:-compile-options
        // https://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/sys/rump/README.compileopts
        Command::new("./buildrump.sh")
            .args(buildrump_arguments)
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

        env::set_var("BUILDRUMP_MACHINE", "amd64");
        env::set_var("BUILDRUMP_MACHINE_GNU_ARCH", "x86_64");
        env::set_var("MACHINE", "amd64");
        env::set_var("MACHINE_GNU_ARCH", "x86_64");

        let rumpmake = out_dir_path.join("obj/tooldir/rumpmake");
        assert!(rumpmake.exists());

        Command::new(rumpmake.as_path())
            .args(&["obj", "dependall", "install"])
            .current_dir(&Path::new("./pci_build"))
            .status()
            .unwrap();

        let user_libs = &[
            "src/lib/libc",
            "src/lib/libcrypt",
            "src/lib/libipsec",
            "src/lib/libkvm",
            "src/lib/libm",
            "src/lib/libnpf",
            "src/lib/libprop",
            "src/lib/libpthread",
            "src/lib/librmt",
            "src/lib/libutil",
            "src/lib/liby",
            "src/lib/libz",
            "src/external/bsd/flex",
            "src/external/bsd/libpcap/lib",
            "src/external/bsd/libc++",
        ];

        // build the relevant 'user-space' libraries
        for lib in user_libs {
            let make = rumpmake.as_path();
            makeuserlib(make, out_dir_path.join(lib).as_path())
                .expect(format!("Can't build {}", lib).as_str());
        }
    }

    assert!(artefacts_built(out_dir_path.as_path()));
    println!("cargo:rustc-link-search=native={}/rump/lib", out_dir);
}
