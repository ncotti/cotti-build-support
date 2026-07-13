use cotti_build_support as bsup;

fn main() {
    let paths = ["/usr/lib", "/usr/local/libb"];

    let tar =
        "/home/nicolas.cotti/cotti/rust/cotti-build-support/libftd2xx-linux-x86_64-1.4.35.tgz";
    let lib_install_dir = "/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp_install";
    let header_install_dir = "/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp_install";
    let output_tar_dir = "/home/nicolas.cotti/cotti/rust/cotti-build-support/tmp";

    //let found_lib = bsup::find_ftd2xx(&paths);
    //println!("Found lib? {found_lib:?}");

    // if ! found_lib {
    //     bsup::install_ftd2xx(tar, lib_install_dir, header_install_dir).expect("Install should succeed");
    // }
}
