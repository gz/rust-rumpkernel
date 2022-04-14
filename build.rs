use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use num_cpus;

/// Verify that necessary files have been built.
fn artefacts_built(build_dir: &Path) -> bool {
    let libs = vec![
        "libc.a",
        "libpthread.a",
        "librump.a",
        "librumpdev.a",
        "librumpdev_audio.a",
        "librumpdev_audio_ac97.a",
        "librumpdev_bpf.a",
        "librumpdev_cgd.a",
        "librumpdev_disk.a",
        "librumpdev_dm.a",
        "librumpdev_drvctl.a",
        "librumpdev_fss.a",
        "librumpdev_hdaudio_hdafg.a",
        "librumpdev_md.a",
        "librumpdev_miiphy.a",
        "librumpdev_netsmb.a",
        "librumpdev_opencrypto.a",
        "librumpdev_pad.a",
        "librumpdev_pci.a",
        "librumpdev_pci_auich.a",
        "librumpdev_pci_eap.a",
        "librumpdev_pci_hdaudio.a",
        "librumpdev_pci_if_iwn.a",
        "librumpdev_pci_if_pcn.a",
        "librumpdev_pci_if_wm.a",
        "librumpdev_pci_usbhc.a",
        "librumpdev_pci_virtio.a",
        "librumpdev_pud.a",
        "librumpdev_putter.a",
        "librumpdev_raidframe.a",
        "librumpdev_rnd.a",
        "librumpdev_scsipi.a",
        "librumpdev_sysmon.a",
        "librumpdev_ubt.a",
        "librumpdev_ucom.a",
        "librumpdev_ugenhc.a",
        "librumpdev_ulpt.a",
        "librumpdev_umass.a",
        "librumpdev_usb.a",
        "librumpdev_virtio_if_vioif.a",
        "librumpdev_virtio_ld.a",
        "librumpdev_virtio_viornd.a",
        "librumpdev_virtio_vioscsi.a",
        "librumpdev_vnd.a",
        "librumpdev_wscons.a",
        "librumpfs_cd9660.a",
        "librumpfs_efs.a",
        "librumpfs_ext2fs.a",
        "librumpfs_fdesc.a",
        "librumpfs_ffs.a",
        "librumpfs_hfs.a",
        "librumpfs_kernfs.a",
        "librumpfs_lfs.a",
        "librumpfs_mfs.a",
        "librumpfs_msdos.a",
        "librumpfs_nfs.a",
        "librumpfs_nfsserver.a",
        "librumpfs_nilfs.a",
        "librumpfs_ntfs.a",
        "librumpfs_null.a",
        "librumpfs_ptyfs.a",
        "librumpfs_smbfs.a",
        "librumpfs_syspuffs.a",
        "librumpfs_sysvbfs.a",
        "librumpfs_tmpfs.a",
        "librumpfs_udf.a",
        "librumpfs_umap.a",
        "librumpfs_union.a",
        "librumpfs_v7fs.a",
        "librumpkern_crypto.a",
        "librumpkern_sljit.a",
        "librumpkern_sys_linux.a",
        "librumpkern_sysproxy.a",
        "librumpkern_tty.a",
        "librumpkern_z.a",
        "librumpnet.a",
        "librumpnet_agr.a",
        "librumpnet_bpfjit.a",
        "librumpnet_bridge.a",
        "librumpnet_config.a",
        "librumpnet_gif.a",
        "librumpnet_local.a",
        "librumpnet_net80211.a",
        "librumpnet_net.a",
        "librumpnet_netbt.a",
        "librumpnet_netinet6.a",
        "librumpnet_netinet.a",
        "librumpnet_netmpls.a",
        "librumpnet_npf.a",
        "librumpnet_pppoe.a",
        "librumpnet_shmif.a",
        "librumpnet_sockin.a",
        "librumpnet_tap.a",
        "librumpvfs.a",
        "librumpvfs_aio.a",
        "librumpvfs_fifofs.a",
        "librumpvfs_layerfs.a",
        "librumpkern_mman.a",
    ];

    let rump_libs_folder =
        build_dir.join("obj-amd64-nrk/dest.stage/rumprun-x86_64/lib/rumprun-nrk/");

    // Check that all files exist now
    for lib in libs.iter() {
        if !rump_libs_folder.join(lib).exists() {
            eprintln!("{:?} was not built", lib);
            return false;
        }
    }
    return true;
}

/// Clones rumprun repo and builds the rumpkernel libraries.
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
        let url = "https://github.com/gz/rumprun.git";
        Command::new("git")
            .args(&["clone", url, out_dir.as_str()])
            .status()
            .unwrap();

        println!("CHECKOUT netbsd-8 {:?}", out_dir);
        Command::new("git")
            .args(&["checkout", "e06a2b0f2e18f1c1dd0776ad306145efa82bfae4"])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();

        println!("INIT SUBMODULES {:?}", out_dir);
        Command::new("git")
            .args(&["submodule", "update", "--init", "--depth", "1"])
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();

        println!("BUILD {:?}", out_dir);
        // CFLAGS=-w: disables all GCC warnings; a drastic method to ensure that the rump
        // code-base compiles also with newer compilers
        //
        // CFLAGS=-fcommon: ignores forgotten extern declarations (lots of them in NetBSD)
        // no longer the default with GCC>=10
        //
        // For more possible options/configurations see also:
        // https://github.com/rumpkernel/wiki/wiki/Performance:-compile-options
        // https://ftp.netbsd.org/pub/NetBSD/NetBSD-current/src/sys/rump/README.compileopts
        let cpus = format!("{}", num_cpus::get());
        let build_args = &["-j", cpus.as_str(), "nrk", "--", "-F", r#"CFLAGS=-w -fcommon"#];
        Command::new("./build-rr.sh")
            .args(build_args)
            .current_dir(&Path::new(&out_dir))
            .status()
            .unwrap();

        println!("OUT_DIR {:?}", out_dir);
    }

    assert!(artefacts_built(out_dir_path.as_path()));

    let rump_libs_folder =
        out_dir_path.join("obj-amd64-nrk/dest.stage/rumprun-x86_64/lib/rumprun-nrk/");

    // Add folder to the sarch path
    println!(
        "cargo:rustc-link-search=native={}",
        rump_libs_folder.as_path().display()
    );

    // Pass bin path via cargo env variable
    let bin_path = out_dir_path.join("rumprun/bin");

    println!("cargo:bin_target={}", bin_path.as_path().display());
}
