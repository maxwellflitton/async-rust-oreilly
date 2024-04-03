

fn main() {
    prost_build::Config::new()
        .out_dir("src/")
        .compile_protos(&["src/data.proto"], &["src/"]).unwrap();
}
